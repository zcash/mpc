use bn::*;
use rand::Rng;

/// Hash a group element with BLAKE2b and interpret it as an
/// element of Fr.
fn hash_group_to_fr<G: Group>(r: &G) -> Fr {
    use bincode::SizeLimit::Infinite;
    use bincode::rustc_serialize::encode;
    use blake2_rfc::blake2b::blake2b;

    let serialized = encode(r, Infinite).unwrap();

    let mut hash = [0; 64];
    hash.copy_from_slice(blake2b(64, &[], &serialized).as_bytes());

    Fr::interpret(&hash)
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Nizk<G: Group> {
    r: G,
    u: Fr
}

impl<G: Group> Nizk<G> {
    /// Constructing the non-interactive schnorr proof for knowledge of log
    /// of s*f in base f, i.e., knowledge of s
    fn new<R: Rng>(f: G, s: Fr, rng: &mut R) -> Nizk<G> {
        let a = Fr::random(rng);
        let r = f * a;
        let c = hash_group_to_fr(&r);
        Nizk {
            r: r,
            u: a + c * s
        }
    }

    /// Verify the Nizk
    fn verify(&self, f: G, fs: G) -> bool {
        let c = hash_group_to_fr(&self.r);
        
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

            let proof = Nizk::new(f, s, rng);
            assert!(proof.verify(f, fs));
            assert!(!proof.verify(f, f * Fr::random(rng)));
            assert!(!proof.verify(f * Fr::random(rng), fs));
        }
    }

    nizk_test_group::<G1>();
    nizk_test_group::<G2>();
}
