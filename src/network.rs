extern crate bn;
extern crate rand;
extern crate crossbeam;
extern crate rustc_serialize;
extern crate blake2_rfc;
extern crate bincode;
// TODO: remove
extern crate snark;

mod protocol;
use self::protocol::*;

mod dvd;
use self::dvd::*;

use snark::*;
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
pub const DIRECTORY_PREFIX: &'static str = "/home/sean/mpc_trialrun/network/";

fn main() {
    let cs = CS::dummy();

    let stage1_before = Stage1Contents::new(&cs);

    let (pubkey, stage1_after): (PublicKey, Stage1Contents) = exchange_disc(
        "A",
        "B",
        |f| -> Result<(), bincode::rustc_serialize::EncodingError> {
            encode_into(&stage1_before, f, Infinite)
        },
        |f| -> Result<(PublicKey, Stage1Contents), bincode::rustc_serialize::DecodingError> {
            let pubkey: PublicKey = try!(decode_from(f, Infinite));
            let stage: Stage1Contents = try!(decode_from(f, Infinite));

            Ok((pubkey, stage))
        }
    );

    assert!(stage1_after.verify_transform(&stage1_before, &pubkey));

    let stage2_before = Stage2Contents::new(&cs, &stage1_after);

    let stage2_after: Stage2Contents = exchange_disc(
        "C",
        "D",
        |f| {
            encode_into(&stage2_before, f, Infinite)
        },
        |f| {
            decode_from(f, Infinite)
        }
    );

    assert!(stage2_after.verify_transform(&stage2_before, &pubkey));

    let stage3_before = Stage3Contents::new(&cs, &stage2_after);

    let stage3_after: Stage3Contents = exchange_disc(
        "E",
        "F",
        |f| {
            encode_into(&stage3_before, f, Infinite)
        },
        |f| {
            decode_from(f, Infinite)
        }
    );

    assert!(stage3_after.verify_transform(&stage3_before, &pubkey));
}
