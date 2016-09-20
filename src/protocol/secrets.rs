use bn::*;
use rand::Rng;
use super::spair::{Spair, same_power};
use super::nizk::Nizk;
#[cfg(feature = "snark")]
use snark::*;
use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

pub type PublicKeyHash = Vec<u8>;

#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey {
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
    f8_gamma: Spair<G1>, // (f8, f8 * gamma)

    nizk_tau: Nizk<G2>,
    nizk_alpha_a: Nizk<G1>,
    nizk_alpha_b: Nizk<G2>,
    nizk_alpha_c: Nizk<G1>,
    nizk_rho_a: Nizk<G2>,
    nizk_rho_b: Nizk<G1>,
    nizk_beta: Nizk<G2>,
    nizk_gamma: Nizk<G1>
}

impl PublicKey {
    fn is_valid(&self) -> bool {
        self.is_well_formed() &&
        self.f3_tau.verify_nizk(&self.nizk_tau) &&
        self.f4_alpha_a.verify_nizk(&self.nizk_alpha_a) &&
        self.nizk_alpha_b.verify(self.f1_rho_a_rho_b, self.f1_rho_a_rho_b_alpha_b) &&
        self.f5_alpha_c.verify_nizk(&self.nizk_alpha_c) &&
        self.nizk_rho_a.verify(self.f1, self.f1_rho_a) &&
        self.f6_rho_b.verify_nizk(&self.nizk_rho_b) &&
        self.nizk_beta.verify(self.f2, self.f2_beta) &&
        self.f8_gamma.verify_nizk(&self.nizk_gamma)
    }

    fn is_well_formed(&self) -> bool {
        !self.f1.is_zero() &&
        !self.f1_rho_a.is_zero() &&
        !self.f1_rho_a_alpha_a.is_zero() &&
        !self.f1_rho_a_rho_b.is_zero() &&
        !self.f1_rho_a_rho_b_alpha_c.is_zero() &&
        !self.f1_rho_a_rho_b_alpha_b.is_zero() &&
        !self.f2.is_zero() &&
        !self.f2_beta.is_zero() &&
        !self.f2_beta_gamma.is_zero() &&
        same_power(&self.f4_alpha_a, &Spair::new(self.f1_rho_a, self.f1_rho_a_alpha_a).unwrap()) &&
        same_power(&self.f5_alpha_c, &Spair::new(self.f1_rho_a_rho_b, self.f1_rho_a_rho_b_alpha_c).unwrap()) &&
        same_power(&self.f6_rho_b, &Spair::new(self.f1_rho_a, self.f1_rho_a_rho_b).unwrap()) &&
        same_power(&self.f7_rho_a_rho_b, &Spair::new(self.f1, self.f1_rho_a_rho_b).unwrap()) &&
        same_power(&self.f8_gamma, &Spair::new(self.f2_beta, self.f2_beta_gamma).unwrap())
    }

    pub fn hash(&self) -> PublicKeyHash {
        use bincode::SizeLimit::Infinite;
        use bincode::rustc_serialize::encode;
        use blake2_rfc::blake2b::blake2b;

        let serialized = encode(self, Infinite).unwrap();

        blake2b(64, &[], &serialized).as_bytes().to_vec()
    }

    pub fn tau_g2(&self) -> Spair<G2> {
        self.f3_tau.clone()
    }

    pub fn alpha_a_g1(&self) -> Spair<G1> {
        self.f4_alpha_a.clone()
    }

    pub fn alpha_c_g1(&self) -> Spair<G1> {
        self.f5_alpha_c.clone()
    }

    pub fn rho_b_g1(&self) -> Spair<G1> {
        self.f6_rho_b.clone()
    }

    pub fn rho_a_rho_b_g1(&self) -> Spair<G1> {
        self.f7_rho_a_rho_b.clone()
    }

    pub fn gamma_g1(&self) -> Spair<G1> {
        self.f8_gamma.clone()
    }

    pub fn alpha_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1_rho_a_rho_b, self.f1_rho_a_rho_b_alpha_b).unwrap()
    }

    pub fn rho_a_g2(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1_rho_a).unwrap()
    }

    pub fn rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1_rho_a, self.f1_rho_a_rho_b).unwrap()
    }

    pub fn alpha_a_rho_a_g2(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1_rho_a_alpha_a).unwrap()
    }

    pub fn alpha_b_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1_rho_a, self.f1_rho_a_rho_b_alpha_b).unwrap()
    }

    pub fn rho_a_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1_rho_a_rho_b).unwrap()
    }

    pub fn alpha_c_rho_a_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1_rho_a_rho_b_alpha_c).unwrap()
    }

    pub fn beta_g2(&self) -> Spair<G2> {
        Spair::new(self.f2, self.f2_beta).unwrap()
    }

    pub fn beta_gamma_g2(&self) -> Spair<G2> {
        Spair::new(self.f2, self.f2_beta_gamma).unwrap()
    }
}

impl Encodable for PublicKey {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        try!(self.f1.encode(s));
        try!(self.f1_rho_a.encode(s));
        try!(self.f1_rho_a_alpha_a.encode(s));
        try!(self.f1_rho_a_rho_b.encode(s));
        try!(self.f1_rho_a_rho_b_alpha_c.encode(s));
        try!(self.f1_rho_a_rho_b_alpha_b.encode(s));
        try!(self.f2.encode(s));
        try!(self.f2_beta.encode(s));
        try!(self.f2_beta_gamma.encode(s));

        try!(self.f3_tau.encode(s));
        try!(self.f4_alpha_a.encode(s));
        try!(self.f5_alpha_c.encode(s));
        try!(self.f6_rho_b.encode(s));
        try!(self.f7_rho_a_rho_b.encode(s));
        try!(self.f8_gamma.encode(s));

        try!(self.nizk_tau.encode(s));
        try!(self.nizk_alpha_a.encode(s));
        try!(self.nizk_alpha_b.encode(s));
        try!(self.nizk_alpha_c.encode(s));
        try!(self.nizk_rho_a.encode(s));
        try!(self.nizk_rho_b.encode(s));
        try!(self.nizk_beta.encode(s));
        try!(self.nizk_gamma.encode(s));

        Ok(())
    }
}

impl Decodable for PublicKey {
    fn decode<S: Decoder>(s: &mut S) -> Result<PublicKey, S::Error> {
        let f1 = try!(G2::decode(s));
        let f1_rho_a = try!(G2::decode(s));
        let f1_rho_a_alpha_a = try!(G2::decode(s));
        let f1_rho_a_rho_b = try!(G2::decode(s));
        let f1_rho_a_rho_b_alpha_c = try!(G2::decode(s));
        let f1_rho_a_rho_b_alpha_b = try!(G2::decode(s));
        let f2 = try!(G2::decode(s));
        let f2_beta = try!(G2::decode(s));
        let f2_beta_gamma = try!(G2::decode(s));

        let f3_tau = try!(Spair::decode(s));
        let f4_alpha_a = try!(Spair::decode(s));
        let f5_alpha_c = try!(Spair::decode(s));
        let f6_rho_b = try!(Spair::decode(s));
        let f7_rho_a_rho_b = try!(Spair::decode(s));
        let f8_gamma = try!(Spair::decode(s));

        let nizk_tau = try!(Nizk::decode(s));
        let nizk_alpha_a = try!(Nizk::decode(s));
        let nizk_alpha_b = try!(Nizk::decode(s));
        let nizk_alpha_c = try!(Nizk::decode(s));
        let nizk_rho_a = try!(Nizk::decode(s));
        let nizk_rho_b = try!(Nizk::decode(s));
        let nizk_beta = try!(Nizk::decode(s));
        let nizk_gamma = try!(Nizk::decode(s));

        let perhaps_valid = PublicKey {
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
            f8_gamma: f8_gamma,
            nizk_tau: nizk_tau,
            nizk_alpha_a: nizk_alpha_a,
            nizk_alpha_b: nizk_alpha_b,
            nizk_alpha_c: nizk_alpha_c,
            nizk_rho_a: nizk_rho_a,
            nizk_rho_b: nizk_rho_b,
            nizk_beta: nizk_beta,
            nizk_gamma: nizk_gamma
        };

        if perhaps_valid.is_valid() {
            Ok(perhaps_valid)
        } else {
            Err(s.error("invalid s-pairs"))
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

        let nizk_tau = f3_tau.nizk(rng, self.tau);
        let nizk_alpha_a = f4_alpha_a.nizk(rng, self.alpha_a);
        let nizk_alpha_b = Nizk::new(rng, f1_rho_a_rho_b, self.alpha_b);
        let nizk_alpha_c = f5_alpha_c.nizk(rng, self.alpha_c);
        let nizk_rho_a = Nizk::new(rng, f1, self.rho_a);
        let nizk_rho_b = f6_rho_b.nizk(rng, self.rho_b);
        let nizk_beta = Nizk::new(rng, f2, self.beta);
        let nizk_gamma = f8_gamma.nizk(rng, self.gamma);

        let tmp = PublicKey {
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
            f8_gamma: f8_gamma,

            nizk_tau: nizk_tau,
            nizk_alpha_a: nizk_alpha_a,
            nizk_alpha_b: nizk_alpha_b,
            nizk_alpha_c: nizk_alpha_c,
            nizk_rho_a: nizk_rho_a,
            nizk_rho_b: nizk_rho_b,
            nizk_beta: nizk_beta,
            nizk_gamma: nizk_gamma
        };

        assert!(tmp.is_valid());

        tmp
    }
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

        assert!(pubkey.is_well_formed() == !expected);
    }

    let rng = &mut ::rand::thread_rng();

    let privkey = PrivateKey::new(rng);
    let pubkey = privkey.pubkey(rng);

    assert!(pubkey.is_valid());

    breaks_wf(&pubkey, |p| &mut p.f1, true);
    breaks_wf(&pubkey, |p| &mut p.f1_rho_a, true);
    breaks_wf(&pubkey, |p| &mut p.f1_rho_a_alpha_a, true);
    breaks_wf(&pubkey, |p| &mut p.f1_rho_a_rho_b, true);
    breaks_wf(&pubkey, |p| &mut p.f1_rho_a_rho_b_alpha_c, true);
    breaks_wf(&pubkey, |p| &mut p.f2_beta, true);
    breaks_wf(&pubkey, |p| &mut p.f2_beta_gamma, true);

    // We only ever need beta (alone) in G2, so changing the
    // relationship between f2 and f2_beta cannot be
    // inconsistent
    breaks_wf(&pubkey, |p| &mut p.f2, false);

    // We only ever need alpha_b (alone) in G2 as well, so
    // f1_rho_a_rho_b_alpha_b cannot be inconsistent with other relationships
    breaks_wf(&pubkey, |p| &mut p.f1_rho_a_rho_b_alpha_b, false);
}
