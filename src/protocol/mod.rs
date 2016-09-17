//! The protocol works in the following steps:
//! 
//! 1. All of the players construct a `PrivateKey`/`PublicKey` keypair
//!    for protocol evaluation. As a technicality for the security
//!    proof, they do not reveal the `PublicKey` immediately, but
//!    instead commit to it (with a blake2 hash) and send the hash to
//!    the coordinator.
//! 2. The coordinator collects the commitments from all the players.
//!    Now, it deterministically performs an R1CS to QAP reduction for
//!    the constraint system and constructs the initial contents of the
//!    first stage, `Stage1Contents`.
//! 3. *Stage 1: Powers of Tau* - The coordinator gives the first player
//!    the initial `Stage1Contents`, and the player transforms it with
//!    their secrets, creating the new `Stage1Contents`, and sending it
//!    to the coordinator along with the `PublicKey` generated in the
//!    second step. The coordinator relays it to the next player, assuming
//!    it is valid, and records it in the transcript for later verification.
//!    This process continues until each player has participated.
//! 4. *Stage 2: Random coefficients, part 1* - The coordinator takes
//!    the final `Stage1Contents` of stage 1 and uses it to construct
//!    `Stage2Contents`. In particular, it must perform an FFT to
//!    evaluate the QAP at tau in the lagrange basis. It now proceeds
//!    as in stage 1, sending `Stage2Contents` to the player, receiving
//!    a transformed `Stage2Contents`, and relaying that to the next
//!    player.
//! 5. *Stage 3: Random coefficients, part 2* - As in the previous stage
//!    the final `Stage2Contents` is transformed into `Stage3Contents`
//!    by the coordinator, and the protocol proceeds as in the previous
//!    two steps, except with `Stage2Contents` instead.
//! 6. The coordinator writes the transcript to disk.

use bn::*;

#[cfg(feature = "snark")]
use snark::*;

mod secrets;
mod spair;
mod multicore;
pub use self::secrets::*;
use self::spair::*;
use self::multicore::*;

#[cfg(feature = "snark")]
mod qap;

/// The powers of tau.
#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Stage1Contents {
    v1: Vec<G1>,
    v2: Vec<G2>
}

impl Stage1Contents {
    #[cfg(feature = "snark")]
    pub fn new(cs: &CS) -> Self {
        Stage1Contents {
            v1: (0..cs.d+1).map(|_| G1::one()).collect(),
            v2: (0..cs.d+1).map(|_| G2::one()).collect()
        }
    }

    pub fn transform(&mut self, s: &PrivateKey) {
        parallel_two(&mut self.v1, &mut self.v2, |start, v1, v2| {
            let mut c = s.tau.pow(Fr::from_str(&format!("{}", start)).unwrap());

            for (g1, g2) in v1.iter_mut().zip(v2.iter_mut()) {
                *g1 = *g1 * c;
                *g2 = *g2 * c;
                c = c * s.tau;
            }
        }, ::THREADS);
    }

    pub fn verify_transform(&self, prev: &Self, p: &PublicKey) -> bool {
        self.v1.len() == prev.v1.len() &&
        self.v2.len() == prev.v2.len() &&
        self.v1[0] == G1::one() &&
        self.v2[0] == G2::one() &&
        prev.v1[0] == G1::one() &&
        prev.v2[0] == G2::one() &&
        !self.v1[1].is_zero() &&
        !self.v2[1].is_zero() &&
        !prev.v1[1].is_zero() &&
        !prev.v2[1].is_zero() &&
        same_power(
            &Spair::new(prev.v1[1], self.v1[1]).unwrap(),
            &p.tau_g2()
        ) &&
        checkseq(&self.v1, &Spair::new(self.v2[0], self.v2[1]).unwrap()) &&
        checkseq(&self.v2, &Spair::new(self.v1[0], self.v1[1]).unwrap())
    }
}

/// Random coefficients, part 1.
#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Stage2Contents {
    vk_a: G2,
    vk_b: G1,
    vk_c: G2,
    vk_z: G2,
    pk_a: Vec<G1>,
    pk_a_prime: Vec<G1>,
    pk_b: Vec<G2>,
    pk_b_temp: Vec<G1>, // compute pk_B in G1 for K query
    pk_b_prime: Vec<G1>,
    pk_c: Vec<G1>,
    pk_c_prime: Vec<G1>
}

impl Stage2Contents {
    #[cfg(feature = "snark")]
    pub fn new(cs: &CS, stage1: &Stage1Contents) -> Self {
        // evaluate QAP for the next round
        let (at, bt1, bt2, ct) = qap::evaluate(&stage1.v1, &stage1.v2, cs);

        Stage2Contents {
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

    pub fn transform(&mut self, s: &PrivateKey) {
        self.vk_a = self.vk_a * s.alpha_a;
        self.vk_b = self.vk_b * s.alpha_b;
        self.vk_c = self.vk_c * s.alpha_c;
        self.vk_z = self.vk_z * (s.rho_a * s.rho_b);
        mul_all_by(&mut self.pk_a, s.rho_a);
        mul_all_by(&mut self.pk_a_prime, s.rho_a * s.alpha_a);
        mul_all_by(&mut self.pk_b, s.rho_b);
        mul_all_by(&mut self.pk_b_temp, s.rho_b);
        mul_all_by(&mut self.pk_b_prime, s.rho_b * s.alpha_b);
        mul_all_by(&mut self.pk_c, s.rho_a * s.rho_b);
        mul_all_by(&mut self.pk_c_prime, s.rho_a * s.rho_b * s.alpha_c);
    }

    pub fn verify_transform(&self, prev: &Self, p: &PublicKey) -> bool {
        !prev.vk_a.is_zero() &&
        !prev.vk_b.is_zero() &&
        !prev.vk_c.is_zero() &&
        !prev.vk_z.is_zero() &&
        !self.vk_a.is_zero() &&
        !self.vk_b.is_zero() &&
        !self.vk_c.is_zero() &&
        !self.vk_z.is_zero() &&
        // Sizes need to match up
        self.pk_a.len() == prev.pk_a.len() &&
        self.pk_a_prime.len() == prev.pk_a_prime.len() &&
        self.pk_b.len() == prev.pk_b.len() &&
        self.pk_b_temp.len() == prev.pk_b_temp.len() &&
        self.pk_b_prime.len() == prev.pk_b_prime.len() &&
        self.pk_c.len() == prev.pk_c.len() &&
        self.pk_c_prime.len() == prev.pk_c_prime.len() &&
        // Check parts of the verification key
        same_power(
            &Spair::new(prev.vk_a, self.vk_a).unwrap(),
            &p.alpha_a_g1()
        ) &&
        same_power(
            &Spair::new(prev.vk_b, self.vk_b).unwrap(),
            &p.alpha_b_g2()
        ) &&
        same_power(
            &Spair::new(prev.vk_c, self.vk_c).unwrap(),
            &p.alpha_c_g1()
        ) &&
        same_power(
            &Spair::new(prev.vk_z, self.vk_z).unwrap(),
            &p.rho_a_rho_b_g1()
        ) &&
        // Check parts of the proving key
        checkvec(
            &prev.pk_a,
            &self.pk_a,
            &p.rho_a_g2()
        ) &&
        checkvec(
            &prev.pk_a_prime,
            &self.pk_a_prime,
            &p.alpha_a_rho_a_g2()
        ) &&
        checkvec(
            &prev.pk_b,
            &self.pk_b,
            &p.rho_b_g1()
        ) &&
        checkvec(
            &prev.pk_b_temp,
            &self.pk_b_temp,
            &p.rho_b_g2()
        ) &&
        checkvec(
            &prev.pk_b_prime,
            &self.pk_b_prime,
            &p.alpha_b_rho_b_g2()
        ) &&
        checkvec(
            &prev.pk_c,
            &self.pk_c,
            &p.rho_a_rho_b_g2()
        ) &&
        checkvec(
            &prev.pk_c_prime,
            &self.pk_c_prime,
            &p.alpha_c_rho_a_rho_b_g2()
        )
    }
}

/// Random coefficients, part 2.
#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Stage3Contents {
    vk_gamma: G2,
    vk_beta_gamma_one: G1,
    vk_beta_gamma_two: G2,
    pk_k: Vec<G1>
}

impl Stage3Contents {
    pub fn new(cs: &CS, stage2: &Stage2Contents) -> Self {
        assert_eq!(stage2.pk_a.len(), cs.num_vars + 1);
        assert_eq!(stage2.pk_b_temp.len(), cs.num_vars + 1);
        assert_eq!(stage2.pk_c.len(), cs.num_vars + 1);

        let mut pk_k = Vec::with_capacity(cs.num_vars + 3);

        // Perform Z extention as libsnark does.
        pk_k.extend_from_slice(&stage2.pk_a);
        pk_k.push(stage2.pk_b_temp[cs.num_vars]);
        pk_k.push(stage2.pk_c[cs.num_vars]);

        // Add B and C
        add_all_to(&mut pk_k[0..cs.num_vars], &stage2.pk_b_temp[0..cs.num_vars]);
        add_all_to(&mut pk_k[0..cs.num_vars], &stage2.pk_c[0..cs.num_vars]);

        Stage3Contents {
            vk_gamma: G2::one(),
            vk_beta_gamma_one: G1::one(),
            vk_beta_gamma_two: G2::one(),
            pk_k: pk_k
        }
    }

    pub fn transform(&mut self, s: &PrivateKey) {
        let betagamma = s.beta * s.gamma;
        self.vk_gamma = self.vk_gamma * s.gamma;
        self.vk_beta_gamma_one = self.vk_beta_gamma_one * betagamma;
        self.vk_beta_gamma_two = self.vk_beta_gamma_two * betagamma;
        mul_all_by(&mut self.pk_k, s.beta);
    }

    pub fn verify_transform(&self, prev: &Self, p: &PublicKey) -> bool {
        !prev.vk_gamma.is_zero() &&
        !prev.vk_beta_gamma_one.is_zero() &&
        !prev.vk_beta_gamma_two.is_zero() &&
        !self.vk_gamma.is_zero() &&
        !self.vk_beta_gamma_one.is_zero() &&
        !self.vk_beta_gamma_two.is_zero() &&
        self.pk_k.len() == prev.pk_k.len() &&
        same_power(
            &Spair::new(prev.vk_gamma, self.vk_gamma).unwrap(),
            &p.gamma_g1()
        ) &&
        same_power(
            &Spair::new(prev.vk_beta_gamma_one, self.vk_beta_gamma_one).unwrap(),
            &p.beta_gamma_g2()
        ) &&
        same_power(
            &Spair::new(prev.vk_beta_gamma_two, self.vk_beta_gamma_two).unwrap(),
            &Spair::new(prev.vk_beta_gamma_one, self.vk_beta_gamma_one).unwrap()
        ) &&
        checkvec(
            &prev.pk_k,
            &self.pk_k,
            &p.beta_g2()
        )
    }
}

#[cfg(feature = "snark")]
pub fn keypair(
    cs: &CS,
    stage1: &Stage1Contents,
    stage2: &Stage2Contents,
    stage3: &Stage3Contents
) -> Keypair {
    Keypair::from(
        cs,
        &stage2.pk_a,
        &stage2.pk_a_prime,
        &stage2.pk_b,
        &stage2.pk_b_prime,
        &stage2.pk_c,
        &stage2.pk_c_prime,
        &stage3.pk_k,
        &stage1.v1,
        &stage2.vk_a,
        &stage2.vk_b,
        &stage2.vk_c,
        &stage3.vk_gamma,
        &stage3.vk_beta_gamma_one,
        &stage3.vk_beta_gamma_two,
        &stage2.vk_z
    )
}

#[test]
fn compare_to_libsnark_generate() {
    let rng = &mut ::rand::thread_rng();

    let privkeys: Vec<_> = (0..3).map(|_| PrivateKey::new(rng)).collect();
    let pubkeys: Vec<_> = privkeys.iter().map(|p| p.pubkey(rng)).collect();

    let cs = CS::dummy();

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
    for private in privkeys.iter() {
        acc.multiply(private);
    }

    assert!(kp == acc.libsnark_keypair(&cs));
}
