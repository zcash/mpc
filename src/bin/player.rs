extern crate mpc;
extern crate rustc_serialize;
extern crate rand;
extern crate bincode;
extern crate bn;

use mpc::*;
use bn::*;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write, self};
use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::{encode_into, decode_from};

mod constants;
use self::constants::*;

fn main() {
    let rng = &mut ::rand::thread_rng();

    println!("Computing secrets and spairs...");
    let secrets = Secrets::new(rng);
    let spairs = secrets.spairs(rng);

    let mut stream = TcpStream::connect("127.0.0.1:65530").unwrap();
    let stream = &mut stream;

    stream.write(&NETWORK_MAGIC).unwrap();

    // Round 1: Commitment to spairs
    {
        encode_into(&spairs.hash(), stream, Infinite).unwrap();
    }

    // Round 2: Powers of tau
    {
        println!("Receiving current tau powers...");

        let mut cur_g1: Vec<G1> = decode_from(stream, Infinite).unwrap();
        let mut cur_g2: Vec<G2> = decode_from(stream, Infinite).unwrap();

        println!("Calculating new tau powers...");

        secrets.taupowers(&mut cur_g1, &mut cur_g2);

        println!("Sending new tau powers...");

        // Send spairs, new g1 / g2
        encode_into(&spairs, stream, Infinite).unwrap();
        encode_into(&cur_g1, stream, Infinite).unwrap();
        encode_into(&cur_g2, stream, Infinite).unwrap();
    }

    // Round 3: Random coeffs, part 1.
    {
        println!("Receiving current random coeffs (stage1)...");
        let mut cur: Stage1Values = decode_from(stream, Infinite).unwrap();

        println!("Calculating new random coeffs (stage1)...");

        secrets.stage1(&mut cur);

        println!("Sending new random coeffs (stage1)...");

        encode_into(&cur, stream, Infinite).unwrap();
    }

    // Round 4: Random coeffs, part 2.
    {
        println!("Receiving current random coeffs (stage2)...");
        let mut cur: Stage2Values = decode_from(stream, Infinite).unwrap();

        println!("Calculating new random coeffs (stage2)...");

        secrets.stage2(&mut cur);

        println!("Sending new random coeffs (stage2)...");

        encode_into(&cur, stream, Infinite).unwrap();
    }

    // Done!
}
