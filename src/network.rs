#![allow(non_snake_case, dead_code)]

extern crate bn;
extern crate rand;
extern crate crossbeam;
extern crate rustc_serialize;
extern crate blake2_rfc;
extern crate bincode;
extern crate byteorder;

mod protocol;
use self::protocol::*;
mod consts;
use self::consts::*;
mod dvd;
use self::dvd::*;

use rand::Rng;
use std::io::{Read,Write};
use std::net::{TcpStream};
use std::thread;
use std::time::Duration;
use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::{encode_into, decode_from};
use rustc_serialize::{Decodable, Encodable};

pub const THREADS: usize = 8;
pub const DIRECTORY_PREFIX: &'static str = "/";
pub const ASK_USER_TO_RECORD_HASHES: bool = false;
const COORDINATOR_ADDR: &'static str = "mpc847619423.z.cash:65530";

struct ConnectionHandler {
    peerid: [u8; 8],
    s: TcpStream,
    msgid: u8
}

impl ConnectionHandler {
    fn new() -> ConnectionHandler {
        let peerid = rand::thread_rng().gen();

        let mut tmp = ConnectionHandler {
            peerid: peerid,
            s: TcpStream::connect(COORDINATOR_ADDR).unwrap(),
            msgid: 0
        };

        tmp.handshake().expect("could not handshake with coordinator");

        tmp
    }

    fn handshake(&mut self) -> Option<u8> {
        if self.s.set_read_timeout(Some(Duration::from_secs(5))).is_err() {
            return None;
        }
        if self.s.set_write_timeout(Some(Duration::from_secs(5))).is_err() {
            return None;
        }
        if self.s.write_all(&NETWORK_MAGIC).is_err() {
            return None;
        }
        if self.s.write_all(&self.peerid).is_err() {
            return None;
        }
        if self.s.write_all(&[self.msgid]).is_err() {
            return None;
        }
        if self.s.flush().is_err() {
            return None;
        }
        let _ = self.s.set_read_timeout(Some(Duration::from_secs(NETWORK_TIMEOUT)));
        let _ = self.s.set_write_timeout(Some(Duration::from_secs(NETWORK_TIMEOUT)));

        let mut buf: [u8; 8] = [0; 8];
        if self.s.read_exact(&mut buf).is_err() {
            return None;
        }

        if buf != COORDINATOR_MAGIC {
            return None;
        }

        let mut buf: [u8; 1] = [0];
        match self.s.read_exact(&mut buf) {
            Ok(_) => Some(buf[0]),
            Err(_) => None
        }
    }

    fn do_with_stream<T, E, F: Fn(&mut TcpStream, u8) -> Result<T, E>>(&mut self, cb: F) -> T
    {
        let mut their_msgid = 0;

        loop {
            let val = cb(&mut self.s, their_msgid);

            match val {
                Ok(s) => {
                    return s;
                },
                Err(_) => {
                    let mut failed = false;

                    loop {
                        match TcpStream::connect(COORDINATOR_ADDR) {
                            Ok(s) => {
                                self.s = s;
                                match self.handshake() {
                                    Some(id) => {
                                        their_msgid = id;
                                        if failed {
                                            println!("Reconnected to coordinator.");
                                        }
                                        break;
                                    },
                                    None => {
                                        thread::sleep(Duration::from_secs(2));
                                    }
                                }
                            },
                            Err(_) => {
                                failed = true;
                                println!("Failed to connect to coordinator, trying again...");
                                thread::sleep(Duration::from_secs(2));
                            }
                        }
                    }
                }
            }
        }
    }

    fn read<T: Decodable>(&mut self) -> T {
        let msg = self.do_with_stream(|s, _| {
            decode_from(s, Infinite)
        });

        self.msgid += 1;

        let _ = self.s.write_all(&NETWORK_ACK);
        let _ = self.s.flush();

        msg
    }

    fn write<T: Encodable>(&mut self, obj: &T) {
        self.msgid += 1;

        let msgid = self.msgid;

        self.do_with_stream(|s, theirid| {
            if theirid >= msgid {
                // They have the message we're sending already.
                return Ok(());
            }

            if encode_into(obj, s, Infinite).is_err() {
                // Couldn't write to the socket, this error will trigger
                // a reconnect.
                return Err("couldn't send data".to_string())
            }

            if s.flush().is_err() {
                // Couldn't flush the buffer, assume the connection failed.
                return Err("couldn't flush buffer".to_string())
            }

            // We expect an ACK now.
            let mut ack: [u8; 4] = [0; 4];
            let _ = s.read_exact(&mut ack);

            if ack != NETWORK_ACK {
                // Bad ACK, this error will trigger reconnect.
                return Err("bad or no ack".to_string())
            }

            // All good.
            Ok(())
        })
    }
}

fn main() {
    prompt("Press [ENTER] when you're ready to perform diagnostics of the DVD drive.");
    perform_diagnostics();
    prompt("Diagnostics complete. Press [ENTER] when you're ready to begin the ceremony.");

    let mut handler = ConnectionHandler::new();

    let comm;
    {
        let mut entered_wrong = false;
        loop {
            let msg = prompt(&format!("Please enter the commitment from the compute machine. It contains a checksum,\n\
                              so don't worry (much) about entering it in wrong. We'll let you keep trying.{}\n\n",
                              if entered_wrong { "\n\nInvalid, try again!"} else { "" }));

            if let Some(c) = Digest256::from_string(&msg) {
                comm = c;
                break;
            } else {
                entered_wrong = true;
            }
        }
    }

    handler.write(&comm);

    println!("Waiting to receive disc 'A' from coordinator server...");
    let hash_of_commitments = handler.read::<Digest512>();
    let stage1_before = handler.read::<Stage1Contents>();
    let prev_msg_hash = handler.read::<Digest256>();

    let (pubkey, nizks, stage1_after, ihash): (PublicKey, PublicKeyNizks, Stage1Contents, Digest256) = exchange_disc(
        "A",
        "B",
        |f| -> Result<(), bincode::rustc_serialize::EncodingError> {
            try!(encode_into(&hash_of_commitments, f, Infinite));
            try!(encode_into(&stage1_before, f, Infinite));

            encode_into(&prev_msg_hash, f, Infinite)
        },
        |f, _| -> Result<(PublicKey, PublicKeyNizks, Stage1Contents, Digest256), bincode::rustc_serialize::DecodingError> {
            let pubkey: PublicKey = try!(decode_from(f, Infinite));
            let nizks: PublicKeyNizks = try!(decode_from(f, Infinite));
            let stage: Stage1Contents = try!(decode_from(f, Infinite));
            let ihash: Digest256 = try!(decode_from(f, Infinite));

            Ok((pubkey, nizks, stage, ihash))
        }
    );

    println!("Sending disc 'B' to the coordinator server...");
    handler.write(&pubkey);
    handler.write(&nizks);
    handler.write(&stage1_after);
    handler.write(&ihash);

    drop(stage1_before);
    drop(stage1_after);

    println!("Waiting to receive disc 'C' from coordinator server...");
    let stage2_before = handler.read::<Stage2Contents>();
    let prev_msg_hash = handler.read::<Digest256>();

    let (stage2_after, ihash): (Stage2Contents, Digest256) = exchange_disc(
        "C",
        "D",
        |f| {
            try!(encode_into(&stage2_before, f, Infinite));

            encode_into(&prev_msg_hash, f, Infinite)
        },
        |f, _| -> Result<(Stage2Contents, Digest256), bincode::rustc_serialize::DecodingError> {
            let stage2_after: Stage2Contents = try!(decode_from(f, Infinite));
            let ihash: Digest256 = try!(decode_from(f, Infinite));

            Ok((stage2_after, ihash))
        }
    );

    println!("Sending disc 'D' to the coordinator server...");
    handler.write(&stage2_after);
    handler.write(&ihash);

    drop(stage2_before);
    drop(stage2_after);

    println!("Waiting to receive disc 'E' from coordinator server...");
    let stage3_before = handler.read::<Stage3Contents>();
    let prev_msg_hash = handler.read::<Digest256>();

    let (stage3_after, ihash): (Stage3Contents, Digest256) = exchange_disc(
        "E",
        "F",
        |f| {
            try!(encode_into(&stage3_before, f, Infinite));

            encode_into(&prev_msg_hash, f, Infinite)
        },
        |f, _| -> Result<(Stage3Contents, Digest256), bincode::rustc_serialize::DecodingError> {
            let stage3_after: Stage3Contents = try!(decode_from(f, Infinite));
            let ihash: Digest256 = try!(decode_from(f, Infinite));

            Ok((stage3_after, ihash))
        }
    );

    println!("Sending disc 'F' to the coordinator server...");
    handler.write(&stage3_after);
    handler.write(&ihash);

    drop(stage3_before);
    drop(stage3_after);

    eject();

    loop {
        prompt("Done! Both machines can be shut down.\n\
                Do not destroy any DVDs, and ensure there are no DVDs still\n\
                inside of either machine. Place them all in a safe and secure\n\
                place.");
    }
}
