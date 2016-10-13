#![allow(non_snake_case, dead_code)]

extern crate bn;
extern crate rand;
extern crate snark;
extern crate crossbeam;
extern crate rustc_serialize;
extern crate blake2_rfc;
extern crate bincode;
extern crate byteorder;

#[macro_use]
mod protocol;

mod consts;
use self::consts::*;

use std::fs::File;
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

    // Hash of all the commitments.
    let hash_of_commitments = Digest512::from(&commitments).unwrap();

    // Hash of the last message
    let mut last_message_hash = Digest256::from(&commitments).unwrap();

    let mut stage1 = Stage1Contents::new(&cs);

    for i in 0..num_players {
        let expected_ihash = {
            let h = digest256_from_parts!(
                hash_of_commitments,
                stage1,
                last_message_hash
            );
            println!("Player {} hash of disk A: {}", i+1, h.to_string());
            h
        };
        let pubkey: PublicKey = decode_from(&mut f, Infinite).unwrap();

        if pubkey.hash() != commitments[i] {
            panic!("Invalid commitment from player {}", i);
        }

        let nizks: PublicKeyNizks = decode_from(&mut f, Infinite).unwrap();

        if !nizks.is_valid(&pubkey, &hash_of_commitments) {
            panic!("Invalid nizks from player {}", i);
        }

        let new_stage: Stage1Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage1, &pubkey) {
            panic!("Invalid stage1 transformation from player {}", i);
        }

        let ihash: Digest256 = decode_from(&mut f, Infinite).unwrap();
        assert!(ihash == expected_ihash);

        {
            last_message_hash = digest256_from_parts!(
                pubkey,
                nizks,
                new_stage,
                ihash
            );
            println!("Player {} hash of disk B: {}", i+1, last_message_hash.to_string());
        }

        stage1 = new_stage;
        pubkeys.push(pubkey);
    }

    let mut stage2 = Stage2Contents::new(&cs, &stage1);

    for i in 0..num_players {
        let expected_ihash = {
            let h = digest256_from_parts!(
                stage2,
                last_message_hash
            );
            println!("Player {} hash of disk C: {}", i+1, h.to_string());

            h
        };

        let new_stage: Stage2Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage2, &pubkeys[i]) {
            panic!("Invalid stage2 transformation from player {}", i);
        }

        let ihash: Digest256 = decode_from(&mut f, Infinite).unwrap();
        assert!(ihash == expected_ihash);

        {
            last_message_hash = digest256_from_parts!(
                new_stage,
                ihash
            );

            println!("Player {} hash of disk D: {}", i+1, last_message_hash.to_string());
        }

        stage2 = new_stage;
    }

    let mut stage3 = Stage3Contents::new(&cs, &stage2);

    for i in 0..num_players {
        let expected_ihash = {
            let h = digest256_from_parts!(
                stage3,
                last_message_hash
            );
            println!("Player {} hash of disk E: {}", i+1, h.to_string());

            h
        };

        let new_stage: Stage3Contents = decode_from(&mut f, Infinite).unwrap();
        if !new_stage.verify_transform(&stage3, &pubkeys[i]) {
            panic!("Invalid stage3 transformation from player {}", i);
        }

        let ihash: Digest256 = decode_from(&mut f, Infinite).unwrap();

        assert!(expected_ihash == ihash);

        {
            last_message_hash = digest256_from_parts!(
                new_stage,
                ihash
            );
            println!("Player {} hash of disk F: {}", i+1, last_message_hash.to_string());
        }

        stage3 = new_stage;
    }

    let kp = keypair(&cs, &stage1, &stage2, &stage3);
    kp.write_to_disk();
}
