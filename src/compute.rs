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
pub const DIRECTORY_PREFIX: &'static str = "/";
pub const ASK_USER_TO_RECORD_HASHES: bool = true;

fn entropy_from_kernel(seed: &mut [u8; 32]) {
    println!("Please wait...");

    let mut linux_rng = rand::read::ReadRng::new(File::open("/dev/random").unwrap());

    for i in 0..32 {
        let n: u8 = linux_rng.gen();
        seed[i] = n;
    }
}

fn entropy_from_user(seed: &mut [u8; 32]) {
    use blake2_rfc::blake2s::blake2s;

    let entropy = prompt("Please type a random string of text and then press [ENTER] to provide additional entropy.");

    let hash = blake2s(32, &[], entropy.as_bytes());

    seed.copy_from_slice(hash.as_bytes());
}

fn get_entropy() -> [u32; 8] {
    use blake2_rfc::blake2s::blake2s;

    let mut v = vec![];

    {
        let mut seed1: [u8; 32] = [0; 32];
        entropy_from_kernel(&mut seed1);
        let mut seed2: [u8; 32] = [0; 32];
        entropy_from_user(&mut seed2);
        let mut seed3: [u8; 32] = [0; 32];
        entropy_from_kernel(&mut seed3);

        v.extend_from_slice(&seed1);
        v.extend_from_slice(&seed2);
        v.extend_from_slice(&seed3);
    }

    assert_eq!(v.len(), 32*3);

    let mut seed: [u32; 8] = [0; 8];

    let hash = blake2s(32, &[], &v);
    let hash = hash.as_bytes();

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

    let (privkey, pubkey, comm) = {
        let seed = get_entropy();

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
