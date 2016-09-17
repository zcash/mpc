use bn::*;
use rand::Rng;
use super::spair::{Spair, same_power};
#[cfg(feature = "snark")]
use snark::*;
use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

pub type PublicKeyHash = [u8; 32];

pub struct PublicKey {
    f1: G2, // f1
    f1pA: G2, // f1 * rho_a
    f1pAaA: G2, // f1 * rho_a * alpha_a
    f1pApB: G2, // f1 * rho_a * rho_b
    f1pApBaC: G2, // f1 * rho_a * rho_b * alpha_c
    f1pApBaB: G2, // f1 * rho_a * rho_b * alpha_b
    f2: G2, // f2
    f2beta: G2, // f2 * beta
    f2betagamma: G2, // f2 * beta * gamma
    tau: Spair<G2>, // (f0, f0 * tau)
    aA: Spair<G1>, // (f3, f3 * alpha_a)
    aC: Spair<G1>, // (f4, f4 * alpha_c)
    pB: Spair<G1>, // (f5, f5 * rho_b)
    pApB: Spair<G1>, // (f6, f6 * rho_a)
    gamma: Spair<G1> // (f7, f7 * gamma)
}

impl PublicKey {
    fn is_valid(&self) -> bool {
        !self.f1.is_zero() &&
        !self.f1pA.is_zero() &&
        !self.f1pAaA.is_zero() &&
        !self.f1pApB.is_zero() &&
        !self.f1pApBaC.is_zero() &&
        !self.f1pApBaB.is_zero() &&
        !self.f2.is_zero() &&
        !self.f2beta.is_zero() &&
        !self.f2betagamma.is_zero() &&
        same_power(&self.aA, &Spair::new(self.f1pA, self.f1pAaA).unwrap()) &&
        same_power(&self.aC, &Spair::new(self.f1pApB, self.f1pApBaC).unwrap()) &&
        same_power(&self.pB, &Spair::new(self.f1pA, self.f1pApB).unwrap()) &&
        same_power(&self.pApB, &Spair::new(self.f1, self.f1pApB).unwrap()) &&
        same_power(&self.gamma, &Spair::new(self.f2beta, self.f2betagamma).unwrap())
    }

    pub fn hash(&self) -> PublicKeyHash {
        // TODO
        [0xff; 32]
    }

    pub fn tau_g2(&self) -> Spair<G2> {
        self.tau.clone()
    }

    pub fn alpha_a_g1(&self) -> Spair<G1> {
        self.aA.clone()
    }

    pub fn alpha_c_g1(&self) -> Spair<G1> {
        self.aC.clone()
    }

    pub fn rho_b_g1(&self) -> Spair<G1> {
        self.pB.clone()
    }

    pub fn rho_a_rho_b_g1(&self) -> Spair<G1> {
        self.pApB.clone()
    }

    pub fn gamma_g1(&self) -> Spair<G1> {
        self.gamma.clone()
    }

    pub fn alpha_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1pApB, self.f1pApBaB).unwrap()
    }

    pub fn rho_a_g2(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1pA).unwrap()
    }

    pub fn rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1pA, self.f1pApB).unwrap()
    }

    pub fn alpha_a_rho_a_g2(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1pAaA).unwrap()
    }

    pub fn alpha_b_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1pA, self.f1pApBaB).unwrap()
    }

    pub fn rho_a_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1pApB).unwrap()
    }

    pub fn alpha_c_rho_a_rho_b_g2(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1pApBaC).unwrap()
    }

    pub fn beta_g2(&self) -> Spair<G2> {
        Spair::new(self.f2, self.f2beta).unwrap()
    }

    pub fn beta_gamma_g2(&self) -> Spair<G2> {
        Spair::new(self.f2, self.f2betagamma).unwrap()
    }
}

impl Encodable for PublicKey {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        try!(self.f1.encode(s));
        try!(self.f1pA.encode(s));
        try!(self.f1pAaA.encode(s));
        try!(self.f1pApB.encode(s));
        try!(self.f1pApBaC.encode(s));
        try!(self.f1pApBaB.encode(s));
        try!(self.f2.encode(s));
        try!(self.f2beta.encode(s));
        try!(self.f2betagamma.encode(s));
        try!(self.tau.encode(s));
        try!(self.aA.encode(s));
        try!(self.aC.encode(s));
        try!(self.pB.encode(s));
        try!(self.pApB.encode(s));
        try!(self.gamma.encode(s));

        Ok(())
    }
}

impl Decodable for PublicKey {
    fn decode<S: Decoder>(s: &mut S) -> Result<PublicKey, S::Error> {
        let f1 = try!(G2::decode(s));
        let f1pA = try!(G2::decode(s));
        let f1pAaA = try!(G2::decode(s));
        let f1pApB = try!(G2::decode(s));
        let f1pApBaC = try!(G2::decode(s));
        let f1pApBaB = try!(G2::decode(s));
        let f2 = try!(G2::decode(s));
        let f2beta = try!(G2::decode(s));
        let f2betagamma = try!(G2::decode(s));
        let tau = try!(Spair::decode(s));
        let aA = try!(Spair::decode(s));
        let aC = try!(Spair::decode(s));
        let pB = try!(Spair::decode(s));
        let pApB = try!(Spair::decode(s));
        let gamma = try!(Spair::decode(s));

        let perhaps_valid = PublicKey {
            f1: f1,
            f1pA: f1pA,
            f1pAaA: f1pAaA,
            f1pApB: f1pApB,
            f1pApBaC: f1pApBaC,
            f1pApBaB: f1pApBaB,
            f2: f2,
            f2beta: f2beta,
            f2betagamma: f2betagamma,
            tau: tau,
            aA: aA,
            aC: aC,
            pB: pB,
            pApB: pApB,
            gamma: gamma
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
        let f1pA = f1 * self.rho_a;
        let f1pAaA = f1pA * self.alpha_a;
        let f1pApB = f1pA * self.rho_b;
        let f1pApBaC = f1pApB * self.alpha_c;
        let f1pApBaB = f1pApB * self.alpha_b;
        let f2 = G2::random(rng);
        let f2beta = f2 * self.beta;
        let f2betagamma = f2beta * self.gamma;

        let tmp = PublicKey {
            f1: f1,
            f1pA: f1pA,
            f1pAaA: f1pAaA,
            f1pApB: f1pApB,
            f1pApBaC: f1pApBaC,
            f1pApBaB: f1pApBaB,
            f2: f2,
            f2beta: f2beta,
            f2betagamma: f2betagamma,
            tau: Spair::random(rng, self.tau).unwrap(),
            aA: Spair::random(rng, self.alpha_a).unwrap(),
            aC: Spair::random(rng, self.alpha_c).unwrap(),
            pB: Spair::random(rng, self.rho_b).unwrap(),
            pApB: Spair::random(rng, self.rho_a * self.rho_b).unwrap(),
            gamma: Spair::random(rng, self.gamma).unwrap()
        };

        assert!(tmp.is_valid());

        tmp
    }
}

#[test]
fn create_keypair() {
    let rng = &mut ::rand::thread_rng();

    let privkey = PrivateKey::new(rng);
    let pubkey = privkey.pubkey(rng);

    assert!(pubkey.is_valid());
}
