extern crate bn;
extern crate rand;
extern crate snark;
extern crate crossbeam;
extern crate rustc_serialize;
extern crate blake2_rfc;
extern crate bincode;

mod protocol;
use protocol::*;
use snark::*;

pub const THREADS: usize = 128;

fn main() {
    let rng = &mut ::rand::thread_rng();

    let privkeys: Vec<_> = (0..3).map(|_| PrivateKey::new(rng)).collect();
    let pubkeys: Vec<_> = privkeys.iter().map(|p| p.pubkey(rng)).collect();

    let cs = CS::from_file();

    // Stage 1
    let mut stage1 = Stage1Contents::new(&cs);

    for (private, public) in privkeys.iter().zip(pubkeys.iter()) {
        let prev = stage1.clone();
        stage1.transform(private);
        assert!(stage1.verify_transform(&prev, public));
    }

    // Stage 2
    let mut stage2 = Stage2Contents::new(&cs, &stage1);
    for (private, public) in privkeys.iter().zip(pubkeys.iter()) {
        let prev = stage2.clone();
        stage2.transform(private);
        assert!(stage2.verify_transform(&prev, public));
    }

    // Stage 3
    let mut stage3 = Stage3Contents::new(&cs, &stage2);
    for (private, public) in privkeys.iter().zip(pubkeys.iter()) {
        let prev = stage3.clone();
        stage3.transform(private);
        assert!(stage3.verify_transform(&prev, public));
    }

    let kp = keypair(&cs, &stage1, &stage2, &stage3);

    // Compare to libsnark

    let mut acc = PrivateKey::new_blank();
    let mut doneone = false;
    for private in privkeys.iter() {
        if doneone {
            assert!(kp != acc.libsnark_keypair(&cs));
        } else {
            doneone = true;
        }
        acc.multiply(private);
    }

    assert!(kp == acc.libsnark_keypair(&cs));
}
