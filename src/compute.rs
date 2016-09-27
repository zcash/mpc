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

use bn::*;
use rand::{SeedableRng, Rng};
use std::io::{Read, Write, self};
use std::thread;
use std::time::Duration;
use std::fs::{self, File};
use std::process::Command;
use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::{encode_into, decode_from};

pub const THREADS: usize = 8;
pub const DIRECTORY_PREFIX: &'static str = "/";

fn main() {
    prompt("Press [ENTER] when you're ready to perform diagnostics of the DVD drive.");
    disable_modloop_unmount();
    perform_diagnostics();
    prompt("Diagnostics complete. Press [ENTER] when you're ready to begin the ceremony.");

    println!("Constructing personal keypair...");
    let (privkey, pubkey, comm) = {
        // ChaCha20 seed
        let mut seed: [u32; 8];
        {
            // Obtain 256 bits of random data from the kernel.
            // 
            // TODO CHANGE THIS TO RANDOM NOT URANDOM
            // 
            let mut linux_rng = rand::read::ReadRng::new(File::open("/dev/urandom").unwrap());
            seed = linux_rng.gen();
        }
        let mut chacha_rng = rand::chacha::ChaChaRng::from_seed(&seed);

        let privkey = PrivateKey::new(&mut chacha_rng);
        let pubkey = privkey.pubkey(&mut chacha_rng);
        let comm = pubkey.hash();

        (privkey, pubkey, comm)
    };

    let mut stage1: Stage1Contents = read_disc(
        "A",
        &format!("Commitment: {}\n\n\
                  Please type the above commitment into the networked machine.\n\n\
                  The networked machine should produce disc 'A'.\n\n\
                  When disc 'A' is in the DVD drive, press [ENTER].", comm.to_string()),
        |f| {
            decode_from(f, Infinite)
        }
    );

    reset();
    println!("Please wait while disc 'B' is computed... This could take 1 or 2 hours.");
    stage1.transform(&privkey);

    let mut stage2: Stage2Contents = exchange_disc(
        "B",
        "C",
        |f| {
            try!(encode_into(&pubkey, f, Infinite));
            encode_into(&stage1, f, Infinite)
        },
        |f| {
            decode_from(f, Infinite)
        }
    );

    drop(stage1);

    reset();
    println!("Please wait while disc 'D' is computed... This could take 1 or 2 hours.");
    stage2.transform(&privkey);

    let mut stage3: Stage3Contents = exchange_disc(
        "D",
        "E",
        |f| {
            encode_into(&stage2, f, Infinite)
        },
        |f| {
            decode_from(f, Infinite)
        }
    );

    drop(stage2);

    reset();
    println!("Please wait while disc 'F' is computed...");
    stage3.transform(&privkey);

    write_disc(
        "F",
        |f| {
            encode_into(&stage3, f, Infinite)
        },
    );
}
