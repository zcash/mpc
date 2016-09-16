#![allow(non_snake_case, dead_code)]

extern crate bn;
extern crate rand;
#[cfg(feature = "snark")]
extern crate snark;
extern crate crossbeam;
extern crate rustc_serialize;
extern crate bincode;

mod protocol;
use self::protocol::*;

use rand::Rng;
use std::net::{TcpStream};
use std::io::{Read, Write};
use rustc_serialize::{Decodable, Encodable};
use std::thread;
use std::time::Duration;
use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::{encode_into, decode_from};

const COORDINATOR_ADDR: &'static str = "127.0.0.1:65530";
const NETWORK_MAGIC: [u8; 8] = [0xff, 0xff, 0x1f, 0xbb, 0x1c, 0xee, 0x00, 0x19];
pub const THREADS: usize = 8;

struct ConnectionHandler {
    peerid: [u8; 8],
    s: TcpStream
}

impl ConnectionHandler {
    fn new() -> ConnectionHandler {
        let peerid = rand::thread_rng().gen();

        let mut tmp = ConnectionHandler {
            peerid: peerid,
            s: TcpStream::connect(COORDINATOR_ADDR).unwrap()
        };

        tmp.handshake();

        tmp
    }

    fn handshake(&mut self) {
        self.s.set_read_timeout(Some(Duration::from_secs(60)));
        self.s.set_write_timeout(Some(Duration::from_secs(60)));
        self.s.write(&NETWORK_MAGIC);
        self.s.write(&self.peerid);
        self.s.flush();
    }

    fn do_with_stream<T, E, F: Fn(&mut TcpStream) -> Result<T, E>>(&mut self, cb: F) -> T
    {
        let mut failed = false;

        loop {
            let val = cb(&mut self.s);

            self.s.flush();

            match val {
                Ok(s) => {
                    return s;
                },
                Err(_) => {
                    match TcpStream::connect(COORDINATOR_ADDR) {
                        Ok(s) => {
                            if failed {
                                failed = false;
                                println!("Reconnected to coordinator.");
                            }
                            self.s = s;
                            self.handshake();

                            thread::sleep(Duration::from_secs(2));
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

    fn read<T: Decodable>(&mut self) -> T {
        self.do_with_stream(|s| {
            decode_from(s, Infinite)
        })
    }

    fn write<T: Encodable>(&mut self, obj: &T) {
        self.do_with_stream(|s| {
            encode_into(obj, s, Infinite)
        })
    }
}

fn main() {
    let rng = &mut ::rand::thread_rng();

    let mut handler = ConnectionHandler::new();

    let privkey = PrivateKey::new(rng);
    let pubkey = privkey.pubkey(rng);

    let commitment = pubkey.hash();
    handler.write(&commitment);

    // Get powers of tau.
    {
        println!("Waiting to receive stage1 from coordinator.");
        let mut stage1 = handler.read::<Stage1Contents>();
        println!("Received stage1, transforming.");
        stage1.transform(&privkey);
        println!("Sending new stage1 to coordinator.");
        handler.write(&pubkey);
        handler.write(&stage1);
    }

    // Random coeffs part 1
    {
        println!("Waiting to receive stage2 from coordinator.");
        let mut stage2 = handler.read::<Stage2Contents>();
        println!("Received stage2, transforming.");
        stage2.transform(&privkey);
        println!("Sending new stage2 to coordinator.");
        handler.write(&stage2);
    }

    // Random coeffs part 2
    {
        println!("Waiting to receive stage3 from coordinator.");
        let mut stage3 = handler.read::<Stage3Contents>();
        println!("Received stage3, transforming.");
        stage3.transform(&privkey);
        println!("Sending new stage3 to coordinator.");
        handler.write(&stage3);
    }
}
