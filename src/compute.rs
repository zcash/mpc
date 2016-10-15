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

mod dvd;
use self::dvd::*;

use rand::{SeedableRng, Rng};
use std::fs::{File};
use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::{encode_into, decode_from};

pub const THREADS: usize = 8;
pub const DIRECTORY_PREFIX: &'static str = "/home/compute/";
pub const ASK_USER_TO_RECORD_HASHES: bool = true;

fn get_entropy() -> [u32; 8] {
    use blake2_rfc::blake2s::blake2s;

    let mut v: Vec<u8> = vec![];

    {
        let input_from_user = prompt(
            "Please type a random string of text and then press [ENTER] to provide additional entropy."
        );

        let hash = blake2s(32, &[], input_from_user.as_bytes());

        v.extend_from_slice(hash.as_bytes());
    }

    println!("Please wait while Linux fills up its entropy pool...");
    
    {
        let mut linux_rng = rand::read::ReadRng::new(File::open("/dev/random").unwrap());

        for _ in 0..32 {
            v.push(linux_rng.gen());
        }
    }

    assert_eq!(v.len(), 64);

    let hash = blake2s(32, &[], &v);
    let hash = hash.as_bytes();

    let mut seed: [u32; 8] = [0; 8];

    for i in 0..8 {
        use byteorder::{ByteOrder, LittleEndian};

        seed[i] = LittleEndian::read_u32(&hash[(i*4)..]);
    }

    seed
}

fn main() {
    prompt("Press [ENTER] when you're ready to perform diagnostics of the DVD drive.");
    perform_diagnostics();
    prompt("Diagnostics complete. Press [ENTER] when you're ready to begin the ceremony.");

    let mut chacha_rng = rand::chacha::ChaChaRng::from_seed(&get_entropy());

    let privkey = PrivateKey::new(&mut chacha_rng);
    let pubkey = privkey.pubkey(&mut chacha_rng);
    let comm = pubkey.hash();

    let (hash_of_commitments, mut stage1, prev_msg_hash): (Digest512, Stage1Contents, Digest256) = read_disc(
        "A",
        &format!("Commitment: {}\n\n\
                  Please type the above commitment into the networked machine.\n\n\
                  Also, write the string down on a piece of paper.\n\n\
                  The networked machine should produce disc 'A'.\n\n\
                  When disc 'A' is in the DVD drive, press [ENTER].", comm.to_string()),
        |f, p| -> Result<_, bincode::rustc_serialize::DecodingError> {
            let hash_of_commitments: Digest512 = try!(decode_from(f, Infinite));
            let stage: Stage1Contents = try!(decode_from(f, Infinite));

            Ok((hash_of_commitments, stage, p.unwrap()))
        }
    );

    let nizks = pubkey.nizks(&mut chacha_rng, &privkey, &hash_of_commitments);

    reset();
    println!("Please wait while disc 'B' is computed... This should take 30 minutes to an hour.");
    stage1.transform(&privkey);

    let (mut stage2, prev_msg_hash): (Stage2Contents, Digest256) = exchange_disc(
        "B",
        "C",
        |f| {
            try!(encode_into(&pubkey, f, Infinite));
            try!(encode_into(&nizks, f, Infinite));
            try!(encode_into(&stage1, f, Infinite));

            encode_into(&prev_msg_hash, f, Infinite)
        },
        |f, p| -> Result<(Stage2Contents, Digest256), bincode::rustc_serialize::DecodingError> {
            let stage2 = try!(decode_from(f, Infinite));

            Ok((stage2, p.unwrap()))
        }
    );

    drop(stage1);

    reset();
    println!("Please wait while disc 'D' is computed... This should take 45 to 90 minutes.");
    stage2.transform(&privkey);

    let (mut stage3, prev_msg_hash): (Stage3Contents, Digest256) = exchange_disc(
        "D",
        "E",
        |f| {
            try!(encode_into(&stage2, f, Infinite));

            encode_into(&prev_msg_hash, f, Infinite)
        },
        |f, p| -> Result<(Stage3Contents, Digest256), bincode::rustc_serialize::DecodingError> {
            let stage3 = try!(decode_from(f, Infinite));

            Ok((stage3, p.unwrap()))
        }
    );

    drop(stage2);

    reset();
    println!("Please wait while disc 'F' is computed... This should take 15-30 minutes.");
    stage3.transform(&privkey);

    write_disc(
        "F",
        |f| {
            try!(encode_into(&stage3, f, Infinite));

            encode_into(&prev_msg_hash, f, Infinite)
        },
    );
}
