use bn::*;
use rand::Rng;
use super::digest::Digest512;

#[derive(PartialEq, Eq, Clone, RustcEncodable, RustcDecodable)]
pub struct Nizk<G: Group> {
    r: G,
    u: Fr
}

impl<G: Group> Nizk<G> {
    /// Constructing the non-interactive schnorr proof for knowledge of log
    /// of s*f in base f, i.e., knowledge of s
    pub fn new<R: Rng>(rng: &mut R, f: G, s: Fr) -> Nizk<G> {
        let a = Fr::random(rng);
        let r = f * a;
        let c = Digest512::from(&r).expect("group element should never fail to encode").interpret();
        Nizk {
            r: r,
            u: a + c * s
        }
    }

    /// Verify the Nizk
    pub fn verify(&self, f: G, fs: G) -> bool {
        let c = Digest512::from(&self.r).expect("group element should never fail to encode").interpret();
        
        (f * self.u) == (self.r + fs * c)
    }
}

#[test]
fn nizk_test() {
    fn nizk_test_group<G: Group>() {
        let rng = &mut ::rand::thread_rng();
        for _  in 0..50 {
            let f = G::random(rng);
            let s = Fr::random(rng);
            let fs = f * s;

            let proof = Nizk::new(rng, f, s);
            assert!(proof.verify(f, fs));
            assert!(!proof.verify(f, f * Fr::random(rng)));
            assert!(!proof.verify(f * Fr::random(rng), fs));
        }
    }

    nizk_test_group::<G1>();
    nizk_test_group::<G2>();
}
