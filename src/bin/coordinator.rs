extern crate mpc;
extern crate rustc_serialize;
extern crate rand;
extern crate snark;
extern crate bincode;

use snark::*;
use mpc::*;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write, self};
use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::{encode_into, decode_from};

mod constants;
use self::constants::*;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:65530").unwrap();

    let mut player_connections = vec![];

    println!("Waiting for {} players to connect...", NUM_PLAYERS);

    for stream in listener.incoming().take(NUM_PLAYERS) {
        match stream {
            Ok(mut stream) => {
                let mut magic = [0; 8];
                stream.read_exact(&mut magic);

                if magic != NETWORK_MAGIC {
                    println!("Remote peer sent wrong network magic!");
                } else {
                    player_connections.push(stream);
                    println!("\tPlayer {} connected.", player_connections.len());
                }
            },
            Err(e) => {}
        }
    }

    println!("\tAll players have connected!");
    println!("Constructing constraint system and performing QAP reduction...");
    let cs = CS::from_file();
    println!("\tDone.");

    let rng = &mut ::rand::thread_rng();
    let mut transcript = Transcript::new(rng, &cs);

    println!("Round 1: Receiving commitments from players to their s-pairs...");
    for stream in &mut player_connections {
        let commitment: BlakeHash = decode_from(stream, Infinite).unwrap();
        transcript.take(commitment);

        println!("\tReceived commitment from player.");
    }

    let mut transcript = transcript.next();

    println!("Round 2: Powers of tau...");
    for stream in &mut player_connections {
        println!("\tSending current g1/g2 to player...");
        {
            let (cur_g1, cur_g2) = transcript.current();
            encode_into(cur_g1, stream, Infinite).unwrap();
            encode_into(cur_g2, stream, Infinite).unwrap();
        }

        println!("\tWaiting for spairs, new g1/g2 from player...");

        // Receive new g1 / g2 and spairs
        let spairs: Spairs = decode_from(stream, Infinite).unwrap();
        let new_g1 = decode_from(stream, Infinite).unwrap();
        let new_g2 = decode_from(stream, Infinite).unwrap();

        println!("\tVerifying...");

        // Verify that it's correct.
        assert!(transcript.take(spairs, new_g1, new_g2));
    }

    let mut transcript = transcript.next();

    println!("Round 3: Random coeffs, part 1.");
    for stream in &mut player_connections {
        println!("\tSending current coeffs...");
        // Send user current round data
        encode_into(transcript.current(), stream, Infinite).unwrap();

        println!("\tWaiting for new coeffs...");

        // Get new round data
        let new_data = decode_from(stream, Infinite).unwrap();

        println!("\tVerifying...");

        // Verify that it's correct.
        assert!(transcript.take(new_data));
    }

    let mut transcript = transcript.next();

    println!("Round 4: Random coeffs, part 2.");
    for stream in &mut player_connections {
        println!("\tSending current coeffs...");
        // Send user current round data
        encode_into(transcript.current(), stream, Infinite).unwrap();

        println!("\tWaiting for new coeffs...");

        // Get new round data
        let new_data = decode_from(stream, Infinite).unwrap();

        println!("\tVerifying...");

        // Verify that it's correct.
        assert!(transcript.take(new_data));
    }

    println!("Constructing final keypair.");

    let keypair = transcript.keypair();

    keypair.write_to_disk();
}
