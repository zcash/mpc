extern crate bn;
extern crate rand;
extern crate snark;
extern crate crossbeam;
extern crate rustc_serialize;
extern crate blake2_rfc;
extern crate bincode;
extern crate byteorder;

mod protocol;

mod consts;
use self::consts::*;

use std::fs::File;
use std::io::{Read, Write};
use protocol::*;
use snark::*;

use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::{encode_into, decode_from};

pub const THREADS: usize = 8;

fn main() {
    let mut f = File::open("transcript").unwrap();

    let cs = {
        if USE_DUMMY_CS {
            CS::dummy()
        } else {
            CS::from_file()
        }
    };

    let num_players: usize = decode_from(&mut f, Infinite).unwrap();
    println!("Number of players: {}", num_players);

    let mut commitments = vec![];
    let mut pubkeys = vec![];
    for i in 0..num_players {
        let comm: Digest256 = decode_from(&mut f, Infinite).unwrap();
        commitments.push(comm);
    }

    let mut stage1 = Stage1Contents::new(&cs);

    for i in 0..num_players {
        let pubkey: PublicKey = decode_from(&mut f, Infinite).unwrap();

        if pubkey.hash() != commitments[i] {
            panic!("Invalid commitment from player {}", i);
        }

        let new_stage: Stage1Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage1, &pubkey) {
            panic!("Invalid stage1 transformation from player {}", i);
        }

        stage1 = new_stage;
        pubkeys.push(pubkey);
    }

    let mut stage2 = Stage2Contents::new(&cs, &stage1);

    for i in 0..num_players {
        let new_stage: Stage2Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage2, &pubkeys[i]) {
            panic!("Invalid stage2 transformation from player {}", i);
        }

        stage2 = new_stage;
    }

    let mut stage3 = Stage3Contents::new(&cs, &stage2);

    for i in 0..num_players {
        let new_stage: Stage3Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage3, &pubkeys[i]) {
            panic!("Invalid stage3 transformation from player {}", i);
        }

        stage3 = new_stage;
    }

    let kp = keypair(&cs, &stage1, &stage2, &stage3);
    kp.write_to_disk();
}
