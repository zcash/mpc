use bn::*;
use rand::Rng;
use super::spair::{Spair, same_power};
use super::nizk::Nizk;
use super::digest::{Digest512,Digest256};
#[cfg(feature = "snark")]
use snark::*;
use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey(PublicKeyInner);

#[derive(Clone, PartialEq, Eq, RustcEncodable, RustcDecodable)]
struct PublicKeyInner {
    f1: G2, // f1
    f1_rho_a: G2, // f1 * rho_a
    f1_rho_a_alpha_a: G2, // f1 * rho_a * alpha_a
    f1_rho_a_rho_b: G2, // f1 * rho_a * rho_b
    f1_rho_a_rho_b_alpha_c: G2, // f1 * rho_a * rho_b * alpha_c
    f1_rho_a_rho_b_alpha_b: G2, // f1 * rho_a * rho_b * alpha_b
    f2: G2, // f2
    f2_beta: G2, // f2 * beta
    f2_beta_gamma: G2, // f2 * beta * gamma

    f3_tau: Spair<G2>, // (f3, f3 * tau)
    f4_alpha_a: Spair<G1>, // (f4, f4 * alpha_a)
    f5_alpha_c: Spair<G1>, // (f5, f5 * alpha_c)
    f6_rho_b: Spair<G1>, // (f6, f6 * rho_b)
    f7_rho_a_rho_b: Spair<G1>, // (f7, f7 * rho_a * rho_b)
    f8_gamma: Spair<G1> // (f8, f8 * gamma)
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct PublicKeyNizks {
    tau: Nizk<G2>,
    alpha_a: Nizk<G1>,
    alpha_b: Nizk<G2>,
    alpha_c: Nizk<G1>,
    rho_a: Nizk<G2>,
    rho_b: Nizk<G1>,
    beta: Nizk<G2>,
    gamma: Nizk<G1>
}

impl PublicKeyNizks {
    pub fn is_valid(&self, pubkey: &PublicKey, extra: &Digest512) -> bool {
        pubkey.tau_g2().verify_nizk(&self.tau, extra) &&
        pubkey.alpha_a_g1().verify_nizk(&self.alpha_a, extra) &&
        pubkey.alpha_b_g2().verify_nizk(&self.alpha_b, extra) &&
        pubkey.alpha_c_g1().verify_nizk(&self.alpha_c, extra) &&
        pubkey.rho_a_g2().verify_nizk(&self.rho_a, extra) &&
        pubkey.rho_b_g1().verify_nizk(&self.rho_b, extra) &&
        pubkey.beta_g2().verify_nizk(&self.beta, extra) &&
        pubkey.gamma_g1().verify_nizk(&self.gamma, extra)
    }
}

impl PublicKey {
    fn is_valid(&self) -> bool {
        !self.0.f1.is_zero() &&
        !self.0.f1_rho_a.is_zero() &&
        !self.0.f1_rho_a_alpha_a.is_zero() &&
        !self.0.f1_rho_a_rho_b.is_zero() &&
        !self.0.f1_rho_a_rho_b_alpha_c.is_zero() &&
        !self.0.f1_rho_a_rho_b_alpha_b.is_zero() &&
        !self.0.f2.is_zero() &&
        !self.0.f2_beta.is_zero() &&
        !self.0.f2_beta_gamma.is_zero() &&
        same_power(&self.0.f4_alpha_a, &Spair::new(self.0.f1_rho_a, self.0.f1_rho_a_alpha_a).unwrap()) &&
        same_power(&self.0.f5_alpha_c, &Spair::new(self.0.f1_rho_a_rho_b, self.0.f1_rho_a_rho_b_alpha_c).unwrap()) &&
        same_power(&self.0.f6_rho_b, &Spair::new(self.0.f1_rho_a, self.0.f1_rho_a_rho_b).unwrap()) &&
        same_power(&self.0.f7_rho_a_rho_b, &Spair::new(self.0.f1, self.0.f1_rho_a_rho_b).unwrap()) &&
        same_power(&self.0.f8_gamma, &Spair::new(self.0.f2_beta, self.0.f2_beta_gamma).unwrap())
    }

    pub fn hash(&self) -> Digest256 {
        Digest256::from(self).expect("PublicKey should never fail to encode")
    }

    pub fn nizks<R: Rng>(&self, rng: &mut R, privkey: &PrivateKey, extra: &Digest512) -> PublicKeyNizks {
        PublicKeyNizks {
            tau: self.tau_g2().nizk(rng, privkey.tau, extra),
            alpha_a: self.alpha_a_g1().nizk(rng, privkey.alpha_a, extra),
            alpha_b: self.alpha_b_g2().nizk(rng, privkey.alpha_b, extra),
            alpha_c: self.alpha_c_g1().nizk(rng, privkey.alpha_c, extra),
            rho_a: self.rho_a_g2().nizk(rng, privkey.rho_a, extra),
            rho_b: self.rho_b_g1().nizk(rng, privkey.rho_b, extra),
            beta: self.beta_g2().nizk(rng, privkey.beta, extra),
            gamma: self.gamma_g1().nizk(rng, privkey.gamma, extra)
        }
    }

    pub fn tau_g2(&self) -> Spair<G2> {
        self.0.f3_tau.clone()
    }

    pub fn alpha_a_g1(&self) -> Spair<G1> {
        self.0.f4_alpha_a.clone()
    }

    pub fn alpha_c_g1(&self) -> Spair<G1> {
        self.0.f5_alpha_c.clone()
    }

    pub fn rho_b_g1(&self) -> Spair<G1> {
        self.0.f6_rho_b.clone()
    }

    pub fn rho_a_rho_b_g1(&self) -> Spair<G1> {
        self.0.f7_rho_a_rho_b.clone()
    }

    pub fn gamma_g1(&self) -> Spair<G1> {
        self.0.f8_gamma.clone()
    }

    pub fn alpha_b_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f1_rho_a_rho_b, self.0.f1_rho_a_rho_b_alpha_b).unwrap()
    }

    pub fn rho_a_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f1, self.0.f1_rho_a).unwrap()
    }

    pub fn rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f1_rho_a, self.0.f1_rho_a_rho_b).unwrap()
    }

    pub fn alpha_a_rho_a_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f1, self.0.f1_rho_a_alpha_a).unwrap()
    }

    pub fn alpha_b_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f1_rho_a, self.0.f1_rho_a_rho_b_alpha_b).unwrap()
    }

    pub fn rho_a_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f1, self.0.f1_rho_a_rho_b).unwrap()
    }

    pub fn alpha_c_rho_a_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f1, self.0.f1_rho_a_rho_b_alpha_c).unwrap()
    }

    pub fn beta_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f2, self.0.f2_beta).unwrap()
    }

    pub fn beta_gamma_g2(&self) -> Spair<G2> {
        Spair::new(self.0.f2, self.0.f2_beta_gamma).unwrap()
    }
}

impl Encodable for PublicKey {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.0.encode(s)
    }
}

impl Decodable for PublicKey {
    fn decode<S: Decoder>(s: &mut S) -> Result<PublicKey, S::Error> {
        let perhaps_valid = PublicKey(
            try!(PublicKeyInner::decode(s))
        );

        if perhaps_valid.is_valid() {
            Ok(perhaps_valid)
        } else {
            Err(s.error("invalid public key"))
        }
    }
}

/// The secrets sampled by the player.
pub struct PrivateKey {
    pub tau: Fr,
    pub rho_a: Fr,
    pub rho_b: Fr,
    pub alpha_a: Fr,
    pub alpha_b: Fr,
    pub alpha_c: Fr,
    pub beta: Fr,
    pub gamma: Fr
}

impl PrivateKey {
    /// Construct the player's secrets given a random number
    /// generator.
    pub fn new<R: Rng>(rng: &mut R) -> PrivateKey {
        PrivateKey {
            tau: Fr::random(rng),
            rho_a: Fr::random(rng),
            rho_b: Fr::random(rng),
            alpha_a: Fr::random(rng),
            alpha_b: Fr::random(rng),
            alpha_c: Fr::random(rng),
            beta: Fr::random(rng),
            gamma: Fr::random(rng)
        }
    }

    /// Construct a "blank" private key for accumulating
    /// in tests.
    #[cfg(feature = "snark")]
    pub fn new_blank() -> PrivateKey {
        PrivateKey {
            tau: Fr::one(),
            rho_a: Fr::one(),
            rho_b: Fr::one(),
            alpha_a: Fr::one(),
            alpha_b: Fr::one(),
            alpha_c: Fr::one(),
            beta: Fr::one(),
            gamma: Fr::one()
        }
    }

    #[cfg(feature = "snark")]
    pub fn multiply(&mut self, other: &Self) {
        self.tau = self.tau * other.tau;
        self.alpha_a = self.alpha_a * other.alpha_a;
        self.alpha_b = self.alpha_b * other.alpha_b;
        self.alpha_c = self.alpha_c * other.alpha_c;
        self.rho_a = self.rho_a * other.rho_a;
        self.rho_b = self.rho_b * other.rho_b;
        self.beta = self.beta * other.beta;
        self.gamma = self.gamma * other.gamma;
    }

    #[cfg(feature = "snark")]
    pub fn libsnark_keypair(&self, cs: &CS) -> Keypair {
        Keypair::generate(
            cs,
            &self.tau,
            &self.alpha_a,
            &self.alpha_b,
            &self.alpha_c,
            &self.rho_a,
            &self.rho_b,
            &self.beta,
            &self.gamma
        )
    }

    /// Construct the "public key" used to verify that the player
    /// is performing their transformations correctly.
    pub fn pubkey<R: Rng>(&self, rng: &mut R) -> PublicKey {
        let f1 = G2::random(rng);
        let f1_rho_a = f1 * self.rho_a;
        let f1_rho_a_alpha_a = f1_rho_a * self.alpha_a;
        let f1_rho_a_rho_b = f1_rho_a * self.rho_b;
        let f1_rho_a_rho_b_alpha_c = f1_rho_a_rho_b * self.alpha_c;
        let f1_rho_a_rho_b_alpha_b = f1_rho_a_rho_b * self.alpha_b;
        let f2 = G2::random(rng);
        let f2_beta = f2 * self.beta;
        let f2_beta_gamma = f2_beta * self.gamma;

        let f3_tau = Spair::random(rng, self.tau).unwrap();
        let f4_alpha_a = Spair::random(rng, self.alpha_a).unwrap();
        let f5_alpha_c = Spair::random(rng, self.alpha_c).unwrap();
        let f6_rho_b = Spair::random(rng, self.rho_b).unwrap();
        let f7_rho_a_rho_b = Spair::random(rng, self.rho_a * self.rho_b).unwrap();
        let f8_gamma = Spair::random(rng, self.gamma).unwrap();

        let tmp = PublicKey(PublicKeyInner {
            f1: f1,
            f1_rho_a: f1_rho_a,
            f1_rho_a_alpha_a: f1_rho_a_alpha_a,
            f1_rho_a_rho_b: f1_rho_a_rho_b,
            f1_rho_a_rho_b_alpha_c: f1_rho_a_rho_b_alpha_c,
            f1_rho_a_rho_b_alpha_b: f1_rho_a_rho_b_alpha_b,
            f2: f2,
            f2_beta: f2_beta,
            f2_beta_gamma: f2_beta_gamma,

            f3_tau: f3_tau,
            f4_alpha_a: f4_alpha_a,
            f5_alpha_c: f5_alpha_c,
            f6_rho_b: f6_rho_b,
            f7_rho_a_rho_b: f7_rho_a_rho_b,
            f8_gamma: f8_gamma
        });

        assert!(tmp.is_valid());

        tmp
    }
}

#[test]
fn pubkey_nizks() {
    let rng = &mut ::rand::thread_rng();

    let privkey = PrivateKey::new(rng);
    let pubkey = privkey.pubkey(rng);

    let extra = Digest512::from(&"test").unwrap();
    let extra_wrong = Digest512::from(&"testt").unwrap();

    let nizks = pubkey.nizks(rng, &privkey, &extra);

    assert!(nizks.is_valid(&pubkey, &extra));
    assert!(!nizks.is_valid(&pubkey, &extra_wrong));
}

#[test]
fn pubkey_reserialize() {
    use bincode::rustc_serialize::{encode, decode};
    use bincode::SizeLimit::Infinite;

    let rng = &mut ::rand::thread_rng();

    let privkey = PrivateKey::new(rng);
    let pubkey = privkey.pubkey(rng);

    let a = encode(&pubkey, Infinite).unwrap();
    let b = decode(&a).unwrap();

    assert!(pubkey == b);
}

#[test]
fn pubkey_consistency() {
    // The public key is inherently malleable, but some
    // fields cannot be changed unless others are also
    // changed, which makes for a good consistency check
    // of the code.

    fn breaks_wf<F: for<'a> Fn(&'a mut PublicKey) -> &'a mut G2>(
        pubkey: &PublicKey,
        f: F,
        expected: bool
    ) {
        let mut pubkey = pubkey.clone();

        {
            // Change it in a way that should break consistency.

            let change = f(&mut pubkey);
            *change = *change + *change;
        }

        assert!(pubkey.is_valid() == !expected);
    }

    let rng = &mut ::rand::thread_rng();

    let privkey = PrivateKey::new(rng);
    let pubkey = privkey.pubkey(rng);

    assert!(pubkey.is_valid());

    breaks_wf(&pubkey, |p| &mut p.0.f1, true);
    breaks_wf(&pubkey, |p| &mut p.0.f1_rho_a, true);
    breaks_wf(&pubkey, |p| &mut p.0.f1_rho_a_alpha_a, true);
    breaks_wf(&pubkey, |p| &mut p.0.f1_rho_a_rho_b, true);
    breaks_wf(&pubkey, |p| &mut p.0.f1_rho_a_rho_b_alpha_c, true);
    breaks_wf(&pubkey, |p| &mut p.0.f2_beta, true);
    breaks_wf(&pubkey, |p| &mut p.0.f2_beta_gamma, true);

    // We only ever need beta (alone) in G2, so changing the
    // relationship between f2 and f2_beta cannot be
    // inconsistent
    breaks_wf(&pubkey, |p| &mut p.0.f2, false);

    // We only ever need alpha_b (alone) in G2 as well, so
    // f1_rho_a_rho_b_alpha_b cannot be inconsistent with other relationships
    breaks_wf(&pubkey, |p| &mut p.0.f1_rho_a_rho_b_alpha_b, false);
}
