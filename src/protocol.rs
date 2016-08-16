use snark::*;
use spair::*;
use taupowers::*;
use qap::*;
use std::collections::HashMap;

fn mul_all_by<G: Group>(v: &[G], c: Fr) -> Vec<G> {
    v.iter().map(|g| *g * c).collect()
}

#[derive(Clone)]
struct Secrets {
    tau: Fr,
    rho_a: Fr,
    rho_b: Fr,
    alpha_a: Fr,
    alpha_b: Fr,
    alpha_c: Fr,
    beta: Fr,
    gamma: Fr
}

type BlakeHash = [u8; 1];

#[derive(Clone)]
struct Spairs {
    tau: Spair<G2>,
    f1: G2, // f1
    f1pA: G2, // f1 * rho_a
    f1pAaA: G2, // f1 * rho_a * alpha_a
    f1pApB: G2, // f1 * rho_a * rho_b
    f1pApBaC: G2, // f1 * rho_a * rho_b * alpha_c
    f1pApBaB: G2, // f1 * rho_a * rho_b * alpha_b
    f2: G2, // f2
    f2beta: G2, // f2 * beta
    f2betagamma: G2, // f2 * beta * gamma
    aA: Spair<G1>, // (f3, f3 * alpha_a)
    aC: Spair<G1>, // (f4, f4 * alpha_c)
    pB: Spair<G1>, // (f5, f5 * rho_b)
    pApB: Spair<G1>, // (f6, f6 * rho_a)
    gamma: Spair<G1> // (f7, f7 * gamma)
}

impl Spairs {
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
        same_power(&self.aA, &Spair::new(&self.f1pA, &self.f1pAaA).unwrap()) &&
        same_power(&self.aC, &Spair::new(&self.f1pApB, &self.f1pApBaC).unwrap()) &&
        same_power(&self.pB, &Spair::new(&self.f1pA, &self.f1pApB).unwrap()) &&
        same_power(&self.pApB, &Spair::new(&self.f1, &self.f1pApB).unwrap()) &&
        same_power(&self.gamma, &Spair::new(&self.f2beta, &self.f2betagamma).unwrap())
    }

    fn alpha_b(&self) -> Spair<G2> {
        Spair::new(&self.f1pApB, &self.f1pApBaB).unwrap()
    }

    fn rho_a(&self) -> Spair<G2> {
        Spair::new(&self.f1, &self.f1pA).unwrap()
    }

    fn rho_b(&self) -> Spair<G2> {
        Spair::new(&self.f1pA, &self.f1pApB).unwrap()
    }

    fn alpha_a_rho_a(&self) -> Spair<G2> {
        Spair::new(&self.f1, &self.f1pAaA).unwrap()
    }

    fn alpha_b_rho_b(&self) -> Spair<G2> {
        Spair::new(&self.f1pA, &self.f1pApBaB).unwrap()
    }

    fn rho_a_rho_b(&self) -> Spair<G2> {
        Spair::new(&self.f1, &self.f1pApB).unwrap()
    }

    fn alpha_c_rho_a_rho_b(&self) -> Spair<G2> {
        Spair::new(&self.f1, &self.f1pApBaC).unwrap()
    }

    fn beta(&self) -> Spair<G2> {
        Spair::new(&self.f2, &self.f2beta).unwrap()
    }

    fn beta_gamma(&self) -> Spair<G2> {
        Spair::new(&self.f2, &self.f2betagamma).unwrap()
    }

}

impl Secrets {
    #[cfg(test)]
    fn new_blank() -> Secrets {
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

    fn new() -> Secrets {
        Secrets {
            tau: Fr::random_nonzero(),
            rho_a: Fr::random_nonzero(),
            rho_b: Fr::random_nonzero(),
            alpha_a: Fr::random_nonzero(),
            alpha_b: Fr::random_nonzero(),
            alpha_c: Fr::random_nonzero(),
            beta: Fr::random_nonzero(),
            gamma: Fr::random_nonzero()
        }
    }

    fn spairs(&self) -> Spairs {
        let f1 = G2::random_nonzero();
        let f1pA = f1 * self.rho_a;
        let f1pAaA = f1pA * self.alpha_a;
        let f1pApB = f1pA * self.rho_b;
        let f1pApBaC = f1pApB * self.alpha_c;
        let f1pApBaB = f1pApB * self.alpha_b;
        let f2 = G2::random_nonzero();
        let f2beta = f2 * self.beta;
        let f2betagamma = f2beta * self.gamma;

        let tmp = Spairs {
            tau: Spair::random(&self.tau),
            f1: f1,
            f1pA: f1pA,
            f1pAaA: f1pAaA,
            f1pApB: f1pApB,
            f1pApBaC: f1pApBaC,
            f1pApBaB: f1pApBaB,
            f2: f2,
            f2beta: f2beta,
            f2betagamma: f2betagamma,
            aA: Spair::random(&self.alpha_a),
            aC: Spair::random(&self.alpha_c),
            pB: Spair::random(&self.rho_b),
            pApB: Spair::random(&(self.rho_a * self.rho_b)),
            gamma: Spair::random(&self.gamma)
        };

        assert!(tmp.is_valid());

        tmp
    }
}

struct Player {
    secrets: Secrets,
    spairs: Spairs
}

fn blake2s(s: &Spairs) -> BlakeHash {
    // TODO
    [0]
}

impl Player {
    fn new() -> Player {
        let secrets = Secrets::new();
        let spairs = secrets.spairs();

        Player {
            secrets: secrets,
            spairs: spairs
        }
    }

    #[cfg(test)]
    fn test_multiply_secrets(&self, acc: &mut Secrets) {
        acc.tau = acc.tau * self.secrets.tau;
        acc.alpha_a = acc.alpha_a * self.secrets.alpha_a;
        acc.alpha_b = acc.alpha_b * self.secrets.alpha_b;
        acc.alpha_c = acc.alpha_c * self.secrets.alpha_c;
        acc.rho_a = acc.rho_a * self.secrets.rho_a;
        acc.rho_b = acc.rho_b * self.secrets.rho_b;
        acc.beta = acc.beta * self.secrets.beta;
        acc.gamma = acc.gamma * self.secrets.gamma;
    }

    fn spairs_commitment(&self) -> BlakeHash {
        blake2s(&self.spairs)
    }

    fn exponentiate_with_tau(
        &self,
        prev_g1: &[G1],
        prev_g2: &[G2]) -> (Vec<G1>, Vec<G2>)
    {
        assert_eq!(prev_g1.len(), prev_g2.len());

        let mut new_g1 = Vec::with_capacity(prev_g1.len());
        let mut new_g2 = Vec::with_capacity(prev_g2.len());

        for ((&g1, &g2), tp) in prev_g1.iter().zip(prev_g2.iter()).zip(TauPowers::new(self.secrets.tau)) {
            new_g1.push(g1 * tp);
            new_g2.push(g2 * tp);
        }

        assert_eq!(new_g1.len(), prev_g1.len());
        assert_eq!(new_g2.len(), prev_g2.len());

        (new_g1, new_g2)
    }

    fn random_coeffs_part_one(
        &self,
        vk_A: &G2,
        vk_B: &G1,
        vk_C: &G2,
        vk_Z: &G2,
        pk_A: &[G1],
        pk_A_prime: &[G1],
        pk_B: &[G2],
        pk_B_temp: &[G1],
        pk_B_prime: &[G1],
        pk_C: &[G1],
        pk_C_prime: &[G1]) -> (G2, G1, G2, G2, Vec<G1>, Vec<G1>, Vec<G2>, Vec<G1>, Vec<G1>, Vec<G1>, Vec<G1>)
    {
        (
            *vk_A * self.secrets.alpha_a,
            *vk_B * self.secrets.alpha_b,
            *vk_C * self.secrets.alpha_c,
            *vk_Z * (self.secrets.rho_a * self.secrets.rho_b),
            mul_all_by(pk_A, self.secrets.rho_a),
            mul_all_by(pk_A_prime, (self.secrets.rho_a * self.secrets.alpha_a)),
            mul_all_by(pk_B, self.secrets.rho_b),
            mul_all_by(pk_B_temp, self.secrets.rho_b),
            mul_all_by(pk_B_prime, (self.secrets.rho_b * self.secrets.alpha_b)),
            mul_all_by(pk_C, (self.secrets.rho_a * self.secrets.rho_b)),
            mul_all_by(pk_C_prime, (self.secrets.rho_a * self.secrets.rho_b * self.secrets.alpha_c))
        )
    }

    fn random_coeffs_part_two(
        &self,
        vk_gamma: &G2,
        vk_beta_gamma_one: &G1,
        vk_beta_gamma_two: &G2,
        pk_K: &[G1]) -> (G2, G1, G2, Vec<G1>)
    {
        (
            *vk_gamma * self.secrets.gamma,
            *vk_beta_gamma_one * (self.secrets.beta * self.secrets.gamma),
            *vk_beta_gamma_two * (self.secrets.beta * self.secrets.gamma),
            mul_all_by(pk_K, self.secrets.beta)
        )
    }
}

struct Coordinator {
    commitments: Vec<BlakeHash>,
    spairs: HashMap<usize, Spairs>
}

impl Coordinator {
    fn new() -> Self {
        Coordinator { commitments: vec![], spairs: HashMap::new() }
    }

    fn receive_commitment(&mut self, h: BlakeHash)
    {
        self.commitments.push(h);
    }

    fn check_commitment(&mut self, i: usize, spairs: Spairs) -> bool
    {
        self.spairs.insert(i, spairs.clone());

        spairs.is_valid() && blake2s(&spairs) == self.commitments[i]
    }

    fn check_taupowers(
        &self,
        prev_g1: &[G1],
        prev_g2: &[G2],
        cur_g1: &[G1],
        cur_g2: &[G2],
        player: usize) -> bool
    {
        prev_g1.len() >= 2 &&
        prev_g1.len() == prev_g2.len() &&
        prev_g2.len() == cur_g1.len() &&
        cur_g1.len() == cur_g2.len() &&
        prev_g1[0] == G1::one() &&
        prev_g2[0] == G2::one() &&
        cur_g1[0] == G1::one() &&
        cur_g2[0] == G2::one() &&
        prev_g1[1] != G1::zero() &&
        cur_g1[1] != G1::zero() &&
        prev_g2[1] != G2::zero() &&
        cur_g2[1] != G2::zero() &&
        // Check that we've exponentiated on top of the previous player correctly
        same_power(&Spair::new(&prev_g1[1], &cur_g1[1]).unwrap(), &self.spairs[&player].tau) &&
        // Check that all G1 elements are exponentiated correctly
        checkseq(cur_g1.iter(), &Spair::new(&cur_g2[0], &cur_g2[1]).unwrap()) &&
        // Check that all G2 elements are exponentiated correctly
        checkseq(cur_g2.iter(), &Spair::new(&cur_g1[0], &cur_g1[1]).unwrap())
    }

    fn check_random_coeffs_part_one(
        &self,
        player: usize,
        prev_vk_A: &G2,
        prev_vk_B: &G1,
        prev_vk_C: &G2,
        prev_vk_Z: &G2,
        prev_pk_A: &[G1],
        prev_pk_A_prime: &[G1],
        prev_pk_B: &[G2],
        prev_pk_B_temp: &[G1],
        prev_pk_B_prime: &[G1],
        prev_pk_C: &[G1],
        prev_pk_C_prime: &[G1],
        cur_vk_A: &G2,
        cur_vk_B: &G1,
        cur_vk_C: &G2,
        cur_vk_Z: &G2,
        cur_pk_A: &[G1],
        cur_pk_A_prime: &[G1],
        cur_pk_B: &[G2],
        cur_pk_B_temp: &[G1],
        cur_pk_B_prime: &[G1],
        cur_pk_C: &[G1],
        cur_pk_C_prime: &[G1]
    ) -> bool
    {
        !prev_vk_A.is_zero() && !cur_vk_A.is_zero() &&
        !prev_vk_B.is_zero() && !cur_vk_B.is_zero() &&
        !prev_vk_C.is_zero() && !cur_vk_C.is_zero() &&
        !prev_vk_Z.is_zero() && !cur_vk_Z.is_zero() &&
        prev_pk_A.len() == cur_pk_A.len() &&
        prev_pk_A_prime.len() == cur_pk_A_prime.len() &&
        prev_pk_B.len() == cur_pk_B.len() &&
        prev_pk_B_temp.len() == cur_pk_B_temp.len() &&
        prev_pk_B_prime.len() == cur_pk_B_prime.len() &&
        prev_pk_C.len() == cur_pk_C.len() &&
        prev_pk_C_prime.len() == cur_pk_C_prime.len() &&
        same_power(&Spair::new(prev_vk_A, cur_vk_A).unwrap(), &self.spairs[&player].aA) &&
        same_power(&Spair::new(prev_vk_B, cur_vk_B).unwrap(), &self.spairs[&player].alpha_b()) &&
        same_power(&Spair::new(prev_vk_C, cur_vk_C).unwrap(), &self.spairs[&player].aC) &&
        same_power(&Spair::new(prev_vk_Z, cur_vk_Z).unwrap(), &self.spairs[&player].pApB) &&
        check(prev_pk_A.iter().zip(cur_pk_A.iter()), &self.spairs[&player].rho_a()) &&
        check(prev_pk_A_prime.iter().zip(cur_pk_A_prime.iter()), &self.spairs[&player].alpha_a_rho_a()) &&
        check(prev_pk_B.iter().zip(cur_pk_B.iter()), &self.spairs[&player].pB) &&
        check(prev_pk_B_temp.iter().zip(cur_pk_B_temp.iter()), &self.spairs[&player].rho_b()) &&
        check(prev_pk_B_prime.iter().zip(cur_pk_B_prime.iter()), &self.spairs[&player].alpha_b_rho_b()) &&
        check(prev_pk_C.iter().zip(cur_pk_C.iter()), &self.spairs[&player].rho_a_rho_b()) &&
        check(prev_pk_C_prime.iter().zip(cur_pk_C_prime.iter()), &self.spairs[&player].alpha_c_rho_a_rho_b())
    }


    fn check_random_coeffs_part_two(
        &self,
        player: usize,
        prev_vk_gamma: &G2,
        prev_vk_beta_gamma_one: &G1,
        prev_vk_beta_gamma_two: &G2,
        prev_pk_K: &[G1],
        cur_vk_gamma: &G2,
        cur_vk_beta_gamma_one: &G1,
        cur_vk_beta_gamma_two: &G2,
        cur_pk_K: &[G1]   
    ) -> bool
    {
        !prev_vk_gamma.is_zero() && !cur_vk_gamma.is_zero() &&
        !prev_vk_beta_gamma_one.is_zero() && !cur_vk_beta_gamma_one.is_zero() &&
        !prev_vk_beta_gamma_two.is_zero() && !cur_vk_beta_gamma_two.is_zero() &&
        prev_pk_K.len() == cur_pk_K.len() &&
        same_power(&Spair::new(prev_vk_gamma, cur_vk_gamma).unwrap(), &self.spairs[&player].gamma) &&
        same_power(&Spair::new(prev_vk_beta_gamma_one, cur_vk_beta_gamma_one).unwrap(), &self.spairs[&player].beta_gamma()) &&
        same_power(&Spair::new(prev_vk_beta_gamma_two, cur_vk_beta_gamma_two).unwrap(), &Spair::new(prev_vk_beta_gamma_one, cur_vk_beta_gamma_one).unwrap()) &&
        check(prev_pk_K.iter().zip(cur_pk_K.iter()), &self.spairs[&player].beta())
    }
}

#[test]
fn implthing() {
    initialize();

    let cs = CS::dummy();

    // Each player initializes
    const NUM_PARTIES: usize = 15;
    let players = (0..NUM_PARTIES).map(|_| Player::new()).collect::<Vec<_>>();

    // Coordinator initializes
    let mut coordinator = Coordinator::new();

    // Phase 1: Commitments
    let mut players = players.into_iter().map(|player| {
        coordinator.receive_commitment(player.spairs_commitment());

        Some(player)
    }).collect::<Vec<_>>();

    // Phase 2: Random powers protocol
    //  Each player needs to output spairs
    //  Each player needs to output powers of tau in G1/G2
    let mut powers_of_tau_g1: Vec<G1> = (0..cs.d+1).map(|_| G1::one()).collect::<Vec<_>>();
    let mut powers_of_tau_g2: Vec<G2> = (0..cs.d+1).map(|_| G2::one()).collect::<Vec<_>>();

    for (i, player) in players.iter().enumerate() {
        match *player {
            Some(ref player) => {
                // Players reveal their spairs, which we check against their commitments
                assert!(coordinator.check_commitment(i, player.spairs.clone()));

                // Players compute the powers of tau given the previous player
                let (new_g1, new_g2) = player.exponentiate_with_tau(
                    &powers_of_tau_g1, &powers_of_tau_g2
                );

                // Coordinator checks the powers of tau were computed correctly.
                assert!(coordinator.check_taupowers(&powers_of_tau_g1, &powers_of_tau_g2, &new_g1, &new_g2, i));

                powers_of_tau_g1 = new_g1;
                powers_of_tau_g2 = new_g2;
            },
            None => {
                // Player aborted before this round.
            }
        }
    }

    // Phase 3: Remote computation
    // The coordinator performs an FFT and evaluates the QAP,
    // also performing Z extention.
    let (at, bt1, bt2, ct) = evaluate_qap(&powers_of_tau_g1, &powers_of_tau_g2, &cs);


    // Phase 4: Random Coefficients, part I
    let mut vk_A = G2::one();
    let mut vk_B = G1::one();
    let mut vk_C = G2::one();
    let mut vk_Z = bt2[bt2.len() - 1]; // last value is Z(tau) in G2
    let mut pk_A = at.clone();
    let mut pk_A_prime = at.clone();
    let mut pk_B = bt2.clone();
    let mut pk_B_temp = bt1.clone(); // Compute pk_B in G1 although not part of the key to use for pk_K later
    let mut pk_B_prime = bt1.clone();
    let mut pk_C = ct.clone();
    let mut pk_C_prime = ct.clone();
    
    for (i, player) in players.iter().enumerate() {
        match *player {
            Some(ref player) => {
                let (
                    new_vk_A,
                    new_vk_B,
                    new_vk_C,
                    new_vk_Z,
                    new_pk_A,
                    new_pk_A_prime,
                    new_pk_B,
                    new_pk_B_temp,
                    new_pk_B_prime,
                    new_pk_C,
                    new_pk_C_prime
                ) = player.random_coeffs_part_one(
                    &vk_A,
                    &vk_B,
                    &vk_C,
                    &vk_Z,
                    &pk_A,
                    &pk_A_prime,
                    &pk_B,
                    &pk_B_temp,
                    &pk_B_prime,
                    &pk_C,
                    &pk_C_prime
                );

                assert!(coordinator.check_random_coeffs_part_one(
                    i,
                    &vk_A,
                    &vk_B,
                    &vk_C,
                    &vk_Z,
                    &pk_A,
                    &pk_A_prime,
                    &pk_B,
                    &pk_B_temp,
                    &pk_B_prime,
                    &pk_C,
                    &pk_C_prime,
                    &new_vk_A,
                    &new_vk_B,
                    &new_vk_C,
                    &new_vk_Z,
                    &new_pk_A,
                    &new_pk_A_prime,
                    &new_pk_B,
                    &new_pk_B_temp,
                    &new_pk_B_prime,
                    &new_pk_C,
                    &new_pk_C_prime
                ));

                vk_A = new_vk_A;
                vk_B = new_vk_B;
                vk_C = new_vk_C;
                vk_Z = new_vk_Z;
                pk_A = new_pk_A;
                pk_A_prime = new_pk_A_prime;
                pk_B = new_pk_B;
                pk_B_temp = new_pk_B_temp;
                pk_B_prime = new_pk_B_prime;
                pk_C = new_pk_C;
                pk_C_prime = new_pk_C_prime;
            },
            None => {
                // Player aborted before this round.
            }
        }
    }

    // Phase 5: Random Coefficients, part II
    let mut vk_gamma = G2::one();
    let mut vk_beta_gamma_one = G1::one();
    let mut vk_beta_gamma_two = G2::one();

    // Initializing pk_K as pk_A + pk _B + pk_C
    let mut pk_K = Vec::with_capacity(pk_A.len());

    for ((&a, &b), &c) in pk_A.iter().zip(pk_B_temp.iter()).zip(pk_C.iter()) {
        pk_K.push(a + b + c);
    }

    for (i, player) in players.iter().enumerate() {
        match *player {
            Some(ref player) => {
                let (
                    new_vk_gamma,
                    new_vk_beta_gamma_one,
                    new_vk_beta_gamma_two,
                    new_pk_K
                ) = player.random_coeffs_part_two(
                    &vk_gamma,
                    &vk_beta_gamma_one,
                    &vk_beta_gamma_two,
                    &pk_K
                );

                assert!(coordinator.check_random_coeffs_part_two(
                    i,
                    &vk_gamma,
                    &vk_beta_gamma_one,
                    &vk_beta_gamma_two,
                    &pk_K,
                    &new_vk_gamma,
                    &new_vk_beta_gamma_one,
                    &new_vk_beta_gamma_two,
                    &new_pk_K
                ));

                vk_gamma = new_vk_gamma;
                vk_beta_gamma_one = new_vk_beta_gamma_one;
                vk_beta_gamma_two = new_vk_beta_gamma_two;
                pk_K = new_pk_K;
            },
            None => {
                // Player aborted before this round.
            }
        }
    }
    
    let mut shared_secrets = Secrets::new_blank();

    for p in &players {
        match *p {
            Some(ref p) => {
                p.test_multiply_secrets(&mut shared_secrets);
            },
            None => {
                unreachable!()
            }
        }
    }

    
}
