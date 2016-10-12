use bn::*;
use rand::Rng;
use super::digest::Digest512;

#[derive(PartialEq, Eq, Clone, RustcEncodable, RustcDecodable)]
pub struct Nizk<G: Group> {
    r: G,
    u: Fr
}

#[derive(RustcEncodable)]
pub struct NizkChallengePreimage<'a, G> {
    r: G,
    f: G,
    fs: G,
    extra: &'a Digest512
}

impl<G: Group> Nizk<G> {
    /// Constructing the non-interactive schnorr proof for knowledge of log
    /// of s*f in base f, i.e., knowledge of s
    pub fn new<R: Rng>(rng: &mut R, f: G, s: Fr, extra: &Digest512) -> Nizk<G> {
        let a = Fr::random(rng);
        let r = f * a;
        let c = Digest512::from(&NizkChallengePreimage {
            r: r,
            f: f,
            fs: f * s,
            extra: extra
        }).expect("nizk challenge preimage should not fail to encode").interpret();
        Nizk {
            r: r,
            u: a + c * s
        }
    }

    /// Verify the Nizk
    pub fn verify(&self, f: G, fs: G, extra: &Digest512) -> bool {
        let c = Digest512::from(&NizkChallengePreimage{
            r: self.r,
            f: f,
            fs: fs,
            extra: extra
        }).expect("group element should never fail to encode").interpret();
        
        (f * self.u) == (self.r + fs * c)
    }
}

#[test]
fn nizk_test() {
    fn nizk_test_group<G: Group>() {
        let rng = &mut ::rand::thread_rng();
        let correct_extra = Digest512::from(&"test").unwrap();
        let incorrect_extra = Digest512::from(&"tesst").unwrap();
        for _  in 0..50 {
            let f = G::random(rng);
            let s = Fr::random(rng);
            let fs = f * s;

            let proof = Nizk::new(rng, f, s, &correct_extra);
            assert!(proof.verify(f, fs, &correct_extra));
            {
                let r = Fr::random(rng);
                assert!(!proof.verify(f * r, fs * r, &correct_extra));
            }
            assert!(!proof.verify(f, fs, &incorrect_extra));
            assert!(!proof.verify(f, f * Fr::random(rng), &correct_extra));
            assert!(!proof.verify(f * Fr::random(rng), fs, &correct_extra));
        }
    }

    nizk_test_group::<G1>();
    nizk_test_group::<G2>();
}
