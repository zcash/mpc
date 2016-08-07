extern crate snark;
extern crate crossbeam;

mod util;
mod lagrange;
mod protocol;
mod spair;

use snark::*;
use protocol::*;
use lagrange::*;

fn main() {
    initialize();

    // Get the constraint system
    let cs = getcs();

    // Initialize the players
    const NUM_PLAYERS: usize = 3;
    let players = (0..NUM_PLAYERS).map(|_| Player::new(&cs)).collect::<Vec<_>>();

    // Phase 1 & 2 produces s-pairs for each player
    let spairs = players.iter().map(|p| p.spairs()).collect::<Vec<_>>();

    // Phase 3 produces random powers of tau in G1/G2
    let (taupowers_g1, taupowers_g2) = {
        let mut transcript = vec![];

        for (i, p) in players.iter().enumerate() {
            if i == 0 {
                transcript.push(p.randompowers_start().unwrap());
            } else {
                let v = p.randompowers(&transcript[i-1].0, &transcript[i-1].1).unwrap();
                transcript.push(v);
            }

            // Verification
            assert!(verify_randompowers(&transcript[i],
                                        if i == 0 { None } else { Some(&transcript[i-1]) },
                                        &spairs[i]));
        }

        transcript[NUM_PLAYERS-1].clone()
    };


}
