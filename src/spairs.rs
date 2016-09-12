use bn::*;
use rand::Rng;
use snark::*;
use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
use sequences::*;
use taupowers::*;

pub type BlakeHash = [u8; 32];

// TODO: make more efficient using windowing
fn mul_all_by<G: Group>(v: &mut [G], c: Fr) {
    for g in v {
        *g = *g * c;
    }
}

#[derive(Clone)]
pub struct Stage1Values {
    pub vk_a: G2,
    pub vk_b: G1,
    pub vk_c: G2,
    pub vk_z: G2,
    pub pk_a: Vec<G1>,
    pub pk_a_prime: Vec<G1>,
    pub pk_b: Vec<G2>,
    pub pk_b_temp: Vec<G1>, // compute pk_B in G1 for K query
    pub pk_b_prime: Vec<G1>,
    pub pk_c: Vec<G1>,
    pub pk_c_prime: Vec<G1>
}

impl Stage1Values {
    pub fn new(at: &Vec<G1>, bt1: &Vec<G1>, bt2: &Vec<G2>, ct: &Vec<G1>) -> Self {
        Stage1Values {
            vk_a: G2::one(),
            vk_b: G1::one(),
            vk_c: G2::one(),
            vk_z: bt2[bt2.len() - 1],
            pk_a: at.clone(),
            pk_a_prime: at.clone(),
            pk_b: bt2.clone(),
            pk_b_temp: bt1.clone(),
            pk_b_prime: bt1.clone(),
            pk_c: ct.clone(),
            pk_c_prime: ct.clone()
        }
    }
}

#[derive(Clone)]
pub struct Stage2Values {
    pub vk_gamma: G2,
    pub vk_beta_gamma_one: G1,
    pub vk_beta_gamma_two: G2,
    pub pk_k: Vec<G1>
}

#[derive(Clone, PartialEq, Eq)]
pub struct Spair<G: Group> {
    f: G,
    fs: G
}

impl<G: Group> Encodable for Spair<G> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        try!(self.f.encode(s));
        try!(self.fs.encode(s));

        Ok(())
    }
}

impl<G: Group> Decodable for Spair<G> {
    fn decode<S: Decoder>(s: &mut S) -> Result<Spair<G>, S::Error> {
        let f = try!(G::decode(s));
        let fs = try!(G::decode(s));

        Spair::new(f, fs).ok_or_else(|| s.error("invalid s-pair"))
    }
}

impl<G: Group> Spair<G> {
    pub fn new(f: G, fs: G) -> Option<Self> {
        if f.is_zero() || fs.is_zero() {
            None
        } else {
            Some(Spair {
                f: f,
                fs: fs
            })
        }
    }

    pub fn random<R: Rng>(rng: &mut R, s: Fr) -> Option<Self> {
        let f = G::random(rng);

        Spair::new(f, f * s)
    }
}

pub struct Secrets {
    tau: Fr,
    rho_a: Fr,
    rho_b: Fr,
    alpha_a: Fr,
    alpha_b: Fr,
    alpha_c: Fr,
    beta: Fr,
    gamma: Fr
}

impl Secrets {
    pub fn new_blank() -> Secrets {
        Secrets {
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

    // TODO: use windowing method to make this more efficient
    pub fn taupowers(&self, g1: &mut [G1], g2: &mut [G2])
    {
        for ((g1, g2), tp) in g1.iter_mut().zip(g2.iter_mut()).zip(TauPowers::new(self.tau)) {
            *g1 = *g1 * tp;
            *g2 = *g2 * tp;
        }
    }

    pub fn stage1(&self, v: &mut Stage1Values)
    {
        v.vk_a = v.vk_a * self.alpha_a;
        v.vk_b = v.vk_b * self.alpha_b;
        v.vk_c = v.vk_c * self.alpha_c;
        v.vk_z = v.vk_z * (self.rho_a * self.rho_b);
        // Contribute to proving key
        mul_all_by(&mut v.pk_a, self.rho_a);
        mul_all_by(&mut v.pk_a_prime, (self.rho_a * self.alpha_a));
        mul_all_by(&mut v.pk_b, self.rho_b);
        mul_all_by(&mut v.pk_b_temp, self.rho_b);
        mul_all_by(&mut v.pk_b_prime, (self.rho_b * self.alpha_b));
        mul_all_by(&mut v.pk_c, (self.rho_a * self.rho_b));
        mul_all_by(&mut v.pk_c_prime, (self.rho_a * self.rho_b * self.alpha_c));
    }

    pub fn stage2(&self, v: &mut Stage2Values)
    {
        let betagamma = self.beta * self.gamma;
        v.vk_gamma = v.vk_gamma * self.gamma;
        v.vk_beta_gamma_one = v.vk_beta_gamma_one * betagamma;
        v.vk_beta_gamma_two = v.vk_beta_gamma_two * betagamma;
        mul_all_by(&mut v.pk_k, self.beta);
    }

    pub fn multiply(&mut self, other: &Secrets) {
        self.tau = self.tau * other.tau;
        self.alpha_a = self.alpha_a * other.alpha_a;
        self.alpha_b = self.alpha_b * other.alpha_b;
        self.alpha_c = self.alpha_c * other.alpha_c;
        self.rho_a = self.rho_a * other.rho_a;
        self.rho_b = self.rho_b * other.rho_b;
        self.beta = self.beta * other.beta;
        self.gamma = self.gamma * other.gamma;
    }

    pub fn keypair(&self, cs: &CS) -> Keypair {
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

    pub fn new<R: Rng>(rng: &mut R) -> Secrets {
        Secrets {
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

    pub fn spairs<R: Rng>(&self, rng: &mut R) -> Spairs {
        let f1 = G2::random(rng);
        let f1pA = f1 * self.rho_a;
        let f1pAaA = f1pA * self.alpha_a;
        let f1pApB = f1pA * self.rho_b;
        let f1pApBaC = f1pApB * self.alpha_c;
        let f1pApBaB = f1pApB * self.alpha_b;
        let f2 = G2::random(rng);
        let f2beta = f2 * self.beta;
        let f2betagamma = f2beta * self.gamma;

        let tmp = Spairs {
            tau: Spair::random(rng, self.tau).unwrap(),
            f1: f1,
            f1pA: f1pA,
            f1pAaA: f1pAaA,
            f1pApB: f1pApB,
            f1pApBaC: f1pApBaC,
            f1pApBaB: f1pApBaB,
            f2: f2,
            f2beta: f2beta,
            f2betagamma: f2betagamma,
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

#[derive(Clone, PartialEq, Eq)]
pub struct Spairs {
    // TODO: use a getter for some of this stuff

    pub tau: Spair<G2>,
    f1: G2, // f1
    f1pA: G2, // f1 * rho_a
    f1pAaA: G2, // f1 * rho_a * alpha_a
    f1pApB: G2, // f1 * rho_a * rho_b
    f1pApBaC: G2, // f1 * rho_a * rho_b * alpha_c
    f1pApBaB: G2, // f1 * rho_a * rho_b * alpha_b
    f2: G2, // f2
    f2beta: G2, // f2 * beta
    f2betagamma: G2, // f2 * beta * gamma
    pub aA: Spair<G1>, // (f3, f3 * alpha_a)
    pub aC: Spair<G1>, // (f4, f4 * alpha_c)
    pub pB: Spair<G1>, // (f5, f5 * rho_b)
    pub pApB: Spair<G1>, // (f6, f6 * rho_a)
    pub gamma: Spair<G1> // (f7, f7 * gamma)
}

impl Encodable for Spairs {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        try!(self.tau.encode(s));
        try!(self.f1.encode(s));
        try!(self.f1pA.encode(s));
        try!(self.f1pAaA.encode(s));
        try!(self.f1pApB.encode(s));
        try!(self.f1pApBaC.encode(s));
        try!(self.f1pApBaB.encode(s));
        try!(self.f2.encode(s));
        try!(self.f2beta.encode(s));
        try!(self.f2betagamma.encode(s));
        try!(self.aA.encode(s));
        try!(self.aC.encode(s));
        try!(self.pB.encode(s));
        try!(self.pApB.encode(s));
        try!(self.gamma.encode(s));

        Ok(())
    }
}

impl Decodable for Spairs {
    fn decode<S: Decoder>(s: &mut S) -> Result<Spairs, S::Error> {
        let perhaps_valid = Spairs {
            tau: try!(Spair::decode(s)),
            f1: try!(G2::decode(s)),
            f1pA: try!(G2::decode(s)),
            f1pAaA: try!(G2::decode(s)),
            f1pApB: try!(G2::decode(s)),
            f1pApBaC: try!(G2::decode(s)),
            f1pApBaB: try!(G2::decode(s)),
            f2: try!(G2::decode(s)),
            f2beta: try!(G2::decode(s)),
            f2betagamma: try!(G2::decode(s)),
            aA: try!(Spair::decode(s)),
            aC: try!(Spair::decode(s)),
            pB: try!(Spair::decode(s)),
            pApB: try!(Spair::decode(s)),
            gamma: try!(Spair::decode(s))
        };

        if perhaps_valid.is_valid() {
            Ok(perhaps_valid)
        } else {
            Err(s.error("invalid spairs"))
        }
    }
}

impl Spairs {
    pub fn hash(&self) -> BlakeHash {
        // TODO
        [0; 32]
    }

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

    pub fn alpha_b(&self) -> Spair<G2> {
        Spair::new(self.f1pApB, self.f1pApBaB).unwrap()
    }

    pub fn rho_a(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1pA).unwrap()
    }

    pub fn rho_b(&self) -> Spair<G2> {
        Spair::new(self.f1pA, self.f1pApB).unwrap()
    }

    pub fn alpha_a_rho_a(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1pAaA).unwrap()
    }

    pub fn alpha_b_rho_b(&self) -> Spair<G2> {
        Spair::new(self.f1pA, self.f1pApBaB).unwrap()
    }

    pub fn rho_a_rho_b(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1pApB).unwrap()
    }

    pub fn alpha_c_rho_a_rho_b(&self) -> Spair<G2> {
        Spair::new(self.f1, self.f1pApBaC).unwrap()
    }

    pub fn beta(&self) -> Spair<G2> {
        Spair::new(self.f2, self.f2beta).unwrap()
    }

    pub fn beta_gamma(&self) -> Spair<G2> {
        Spair::new(self.f2, self.f2betagamma).unwrap()
    }
}

pub trait Pairing<G: Group>: Group {
    fn pairing(self, other: G) -> Gt;
}

impl Pairing<G2> for G1 {
    fn pairing(self, other: G2) -> Gt {
        pairing(self, other)
    }
}

impl Pairing<G1> for G2 {
    fn pairing(self, other: G1) -> Gt {
        pairing(other, self)
    }
}

pub fn same_power<Group1: Group, Group2: Group>(a: &Spair<Group1>, b: &Spair<Group2>) -> bool
where Group1: Pairing<Group2> {
    a.f.pairing(b.fs) == a.fs.pairing(b.f)
}

/// This performs a check to see if a large number of (p,q) pairs in G
/// have the same power, with only one pairing.
pub fn check<'a,
         R: Rng,
         Group1: Group,
         Group2: Group,
         I: IntoIterator<Item=(&'a Group1, &'a Group1)>>
         (rng: &mut R, i: I, a: &Spair<Group2>) -> bool
where Group1: Pairing<Group2>
{
    let mut p = Group1::zero();
    let mut q = Group1::zero();

    for v in i {
        let alpha = Fr::random(rng);
        p = p + *v.0 * alpha;
        q = q + *v.1 * alpha;
    }

    if p.is_zero() || q.is_zero() { return false; }

    same_power(&Spair::new(p, q).unwrap(), &a)
}

pub fn checkseq<'a,
            R: Rng,
            Group1: Group,
            Group2: Group,
            I: Iterator<Item=&'a Group1>>
            (rng: &mut R, i: I, a: &Spair<Group2>) -> bool
where Group1: Pairing<Group2>
{
    check(rng, Sequences::new(i), a)
}

#[test]
fn trivial_samepower() {
    let rng = &mut ::rand::thread_rng();

    let f = Fr::random(rng);
    let e = Fr::random(rng);
    let a = Spair::<G1>::random(rng, f).unwrap();
    let b = Spair::<G2>::random(rng, f).unwrap();
    let c = Spair::<G1>::random(rng, e).unwrap();

    assert!(same_power(&a, &b));
    assert!(same_power(&b, &a));
    assert!(!same_power(&b, &c));
}

#[test]
fn samepower_seq() {
    fn general_seq_test<Group1: Group, Group2: Group>()
    where Group1: Pairing<Group2>
    {
        let rng = &mut ::rand::thread_rng();

        // Test working
        {
            let s = Fr::random(rng);
            let p = Spair::<Group2>::random(rng, s).unwrap();

            let mut a = vec![];
            a.push(Group1::random(rng));

            for _ in 0..50 {
                let n = *a.last().unwrap() * s;
                a.push(n);
            }

            assert!(checkseq(rng, a.iter(), &p));
        }

        // Test not working.
        {
            let s = Fr::random(rng);
            let p = Spair::<Group2>::random(rng, s).unwrap();

            let mut a = vec![];
            a.push(Group1::random(rng));

            for i in 0..50 {
                if i == 10 {
                    a.push(Group1::random(rng));
                } else {
                    let n = *a.last().unwrap() * s;
                    a.push(n);
                }
            }

            assert!(!checkseq(rng, a.iter(), &p));
        }
    }

    general_seq_test::<G1, G2>();
    general_seq_test::<G2, G1>();
}
