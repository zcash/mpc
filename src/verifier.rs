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

pub const THREADS: usize = 128;

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
        println!("Player {} commitment: {}", i+1, comm.to_string());
    }

    let mut stage1 = Stage1Contents::new(&cs);

    for i in 0..num_players {
        {
            let mut diskA = vec![];
            encode_into(&stage1, &mut diskA, Infinite).unwrap();
            let hash = Digest256::from_reader(&mut (&diskA[..]));
            println!("Player {} hash of disk A: {}", i+1, hash.to_string());
        }
        let pubkey: PublicKey = decode_from(&mut f, Infinite).unwrap();

        if pubkey.hash() != commitments[i] {
            panic!("Invalid commitment from player {}", i);
        }

        let new_stage: Stage1Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage1, &pubkey) {
            panic!("Invalid stage1 transformation from player {}", i);
        }

        {
            let mut diskB = vec![];
            encode_into(&pubkey, &mut diskB, Infinite).unwrap();
            encode_into(&new_stage, &mut diskB, Infinite).unwrap();
            let hash = Digest256::from_reader(&mut (&diskB[..]));
            println!("Player {} hash of disk B: {}", i+1, hash.to_string());
        }

        stage1 = new_stage;
        pubkeys.push(pubkey);
    }

    let mut stage2 = Stage2Contents::new(&cs, &stage1);

    for i in 0..num_players {
        {
            let mut diskC = vec![];
            encode_into(&stage2, &mut diskC, Infinite).unwrap();
            let hash = Digest256::from_reader(&mut (&diskC[..]));
            println!("Player {} hash of disk C: {}", i+1, hash.to_string());
        }

        let new_stage: Stage2Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage2, &pubkeys[i]) {
            panic!("Invalid stage2 transformation from player {}", i);
        }

        {
            let mut diskD = vec![];
            encode_into(&new_stage, &mut diskD, Infinite).unwrap();
            let hash = Digest256::from_reader(&mut (&diskD[..]));
            println!("Player {} hash of disk D: {}", i+1, hash.to_string());
        }

        stage2 = new_stage;
    }

    let mut stage3 = Stage3Contents::new(&cs, &stage2);

    for i in 0..num_players {
        {
            let mut diskE = vec![];
            encode_into(&stage3, &mut diskE, Infinite).unwrap();
            let hash = Digest256::from_reader(&mut (&diskE[..]));
            println!("Player {} hash of disk E: {}", i+1, hash.to_string());
        }

        let new_stage: Stage3Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage3, &pubkeys[i]) {
            panic!("Invalid stage3 transformation from player {}", i);
        }

        {
            let mut diskF = vec![];
            encode_into(&new_stage, &mut diskF, Infinite).unwrap();
            let hash = Digest256::from_reader(&mut (&diskF[..]));
            println!("Player {} hash of disk F: {}", i+1, hash.to_string());
        }

        stage3 = new_stage;
    }

    let kp = keypair(&cs, &stage1, &stage2, &stage3);
    kp.write_to_disk();
}
