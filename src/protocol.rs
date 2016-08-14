use snark::*;
use spair::*;
use taupowers::*;
use lagrange::*;
use std::collections::HashMap;

#[derive(Clone)]
struct Secrets {
    tau: Fr,
    rho_a: Fr,
    rho_b: Fr,
    alpha_a: Fr,
    alpha_b: Fr,
    alpha_c: Fr
}

type BlakeHash = [u8; 1];

#[derive(Clone)]
struct Spairs {
    tau: Spair<G2>,
    a: G2, // f1
    b: G2, // f1 * rho_a
    c: G2, // f1 * rho_a * alpha_a
    d: G2, // f1 * rho_a * rho_b
    e: G2, // f1 * rho_a * rho_b * alpha_c
    f: G2, // f1 * rho_a * rho_b * alpha_b
    aA: Spair<G1>, // (f2, f2 * alpha_a)
    aC: Spair<G1>, // (f3, f3 * alpha_c)
    pB: Spair<G1>, // (f4, f4 * rho_b)
    pApB: Spair<G1> // (f5, f5 * rho_a)
}

impl Spairs {
    fn is_valid(&self) -> bool {
        !self.a.is_zero() &&
        !self.b.is_zero() &&
        !self.c.is_zero() &&
        !self.d.is_zero() &&
        !self.e.is_zero() &&
        !self.f.is_zero() &&
        same_power(&self.aA, &Spair::new(&self.b, &self.c).unwrap()) &&
        same_power(&self.aC, &Spair::new(&self.d, &self.e).unwrap()) &&
        same_power(&self.pB, &Spair::new(&self.b, &self.d).unwrap()) &&
        same_power(&self.pApB, &Spair::new(&self.a, &self.d).unwrap())
    }

    fn alpha_b(&self) -> Spair<G2> {
        Spair::new(&self.d, &self.f).unwrap()
    }

    fn rho_a(&self) -> Spair<G2> {
        Spair::new(&self.a, &self.b).unwrap()
    }

    fn alpha_a_rho_a(&self) -> Spair<G2> {
        Spair::new(&self.a, &self.c).unwrap()
    }

    fn alpha_b_rho_b(&self) -> Spair<G2> {
        Spair::new(&self.b, &self.f).unwrap()
    }

    fn rho_a_rho_b(&self) -> Spair<G2> {
        Spair::new(&self.a, &self.d).unwrap()
    }

    fn alpha_c_rho_a_rho_b(&self) -> Spair<G2> {
        Spair::new(&self.a, &self.e).unwrap()
    }
}

impl Secrets {
    fn new() -> Secrets {
        Secrets {
            tau: Fr::random_nonzero(),
            rho_a: Fr::random_nonzero(),
            rho_b: Fr::random_nonzero(),
            alpha_a: Fr::random_nonzero(),
            alpha_b: Fr::random_nonzero(),
            alpha_c: Fr::random_nonzero()
        }
    }

    fn spairs(&self) -> Spairs {
        let a = G2::random_nonzero();
        let b = a * self.rho_a;
        let c = b * self.alpha_a;
        let d = b * self.rho_b;
        let e = d * self.alpha_c;
        let f = d * self.alpha_b;

        let tmp = Spairs {
            tau: Spair::random(&self.tau),
            a: a,
            b: b,
            c: c,
            d: d,
            e: e,
            f: f,
            aA: Spair::random(&self.alpha_a),
            aC: Spair::random(&self.alpha_c),
            pB: Spair::random(&self.rho_b),
            pApB: Spair::random(&(self.rho_a * self.rho_b))
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
        vk_A: &mut G2,
        vk_B: &mut G1,
        vk_C: &mut G2,
        vk_Z: &mut G2,
        pk_A: &mut [G1],
        pk_A_prime: &mut [G1],
        pk_B: &mut [G2],
        pk_B_prime: &mut [G1],
        pk_C: &mut [G1],
        pk_C_prime: &mut [G1])
    {
        *vk_A = *vk_A * self.secrets.alpha_a;
        *vk_B = *vk_B * self.secrets.alpha_b;
        *vk_C = *vk_C * self.secrets.alpha_c;
        *vk_Z = *vk_Z * (self.secrets.rho_a * self.secrets.rho_b);

        fn mul_all_by<G: Group>(v: &mut [G], by: Fr) {
            for e in v {
                *e = *e * by;
            }
        }

        mul_all_by(pk_A, self.secrets.rho_a);
        mul_all_by(pk_A_prime, (self.secrets.rho_a * self.secrets.alpha_a));
        mul_all_by(pk_B, self.secrets.rho_b);
        mul_all_by(pk_B_prime, (self.secrets.rho_b * self.secrets.alpha_b));
        mul_all_by(pk_C, (self.secrets.rho_a * self.secrets.rho_b));
        mul_all_by(pk_C_prime, (self.secrets.rho_a * self.secrets.rho_b * self.secrets.alpha_c));
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
        // Check that we've exponentiated on top of the previous one correctly
        same_power(&Spair::new(&prev_g1[1], &cur_g1[1]).unwrap(), &self.spairs[&player].tau) &&
        // Check that all G1 elements are exponentiated correctly
        checkseq(cur_g1.iter(), &Spair::new(&cur_g2[0], &cur_g2[1]).unwrap()) &&
        // Check that all G2 elements are exponentiated correctly
        checkseq(cur_g2.iter(), &Spair::new(&cur_g1[0], &cur_g1[1]).unwrap())
    }

    fn evaluate_qap(&self, g1_powers: &[G1], g2_powers: &[G2], cs: &CS) -> (Vec<G1>, Vec<G1>, Vec<G2>, Vec<G1>)
    {
        assert_eq!(g1_powers.len(), g2_powers.len());
        assert_eq!(g2_powers.len(), cs.d+1);

        let lc1 = lagrange_coeffs(&g1_powers[0..cs.d], cs.omega);
        let lc2 = lagrange_coeffs(&g2_powers[0..cs.d], cs.omega);

        let mut at = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();
        let mut bt1 = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();
        let mut bt2 = (0..cs.num_vars).map(|_| G2::zero()).collect::<Vec<_>>();
        let mut ct = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();

        cs.eval(&lc1, &lc2, &mut at, &mut bt1, &mut bt2, &mut ct);

        // Push Zt = g^(tau^d - 1) = g^(tau^d) - g
        at.push(g1_powers[cs.d] - G1::one());
        bt1.push(g1_powers[cs.d] - G1::one());
        bt2.push(g2_powers[cs.d] - G2::one());
        ct.push(g1_powers[cs.d] - G1::one());

        (at, bt1, bt2, ct)
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
        check(prev_pk_B_prime.iter().zip(cur_pk_B_prime.iter()), &self.spairs[&player].alpha_b_rho_b()) &&
        check(prev_pk_C.iter().zip(cur_pk_C.iter()), &self.spairs[&player].rho_a_rho_b()) &&
        check(prev_pk_C_prime.iter().zip(cur_pk_C_prime.iter()), &self.spairs[&player].alpha_c_rho_a_rho_b())
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

    // Simulate one participant leaving the protocol
    players[3] = None;

    // Phase 2: Random powers protocol
    //  Each player needs to output spairs
    //  Each player needs to output powers of tau in G1/G2
    let mut powers_of_tau_g1: Vec<G1> = (0..cs.d+1).map(|_| G1::one()).collect::<Vec<_>>();
    let mut powers_of_tau_g2: Vec<G2> = (0..cs.d+1).map(|_| G2::one()).collect::<Vec<_>>();

    for (i, player) in players.iter().enumerate() {
        match *player {
            Some(ref player) => {
                assert!(coordinator.check_commitment(i, player.spairs.clone()));

                let (new_g1, new_g2) = player.exponentiate_with_tau(
                    &powers_of_tau_g1, &powers_of_tau_g2
                );

                assert!(coordinator.check_taupowers(&powers_of_tau_g1, &powers_of_tau_g2, &new_g1, &new_g2, i));

                powers_of_tau_g1 = new_g1;
                powers_of_tau_g2 = new_g2;
            },
            None => {
                // Player aborted before this round.
            }
        }
    }

    // Simulate another participant leaving the protocol
    players[6] = None;

    // Phase 3: Remote computation
    // The coordinator performs an FFT and evaluates the QAP,
    // also performing Z extention.
    let (at, bt1, bt2, ct) = coordinator.evaluate_qap(&powers_of_tau_g1, &powers_of_tau_g2, &cs);


    // Phase 4: Random Coefficients, part I
    let mut vk_A = G2::one();
    let mut vk_B = G1::one();
    let mut vk_C = G2::one();
    let mut vk_Z = bt2[bt2.len() - 1]; // last value is Z(tau) in G2
    let mut pk_A = at.clone();
    let mut pk_A_prime = at.clone();
    let mut pk_B = bt2.clone();
    let mut pk_B_prime = bt1.clone();
    let mut pk_C = ct.clone();
    let mut pk_C_prime = ct.clone();
    
    for (i, player) in players.iter().enumerate() {
        match *player {
            Some(ref player) => {
                let mut new_vk_A = vk_A.clone();
                let mut new_vk_B = vk_B.clone();
                let mut new_vk_C = vk_C.clone();
                let mut new_vk_Z = vk_Z.clone();
                let mut new_pk_A = pk_A.clone();
                let mut new_pk_A_prime = pk_A_prime.clone();
                let mut new_pk_B = pk_B.clone();
                let mut new_pk_B_prime = pk_B_prime.clone();
                let mut new_pk_C = pk_C.clone();
                let mut new_pk_C_prime = pk_C_prime.clone();

                player.random_coeffs_part_one(
                    &mut new_vk_A,
                    &mut new_vk_B,
                    &mut new_vk_C,
                    &mut new_vk_Z,
                    &mut new_pk_A,
                    &mut new_pk_A_prime,
                    &mut new_pk_B,
                    &mut new_pk_B_prime,
                    &mut new_pk_C,
                    &mut new_pk_C_prime
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
                pk_B_prime = new_pk_B_prime;
                pk_C = new_pk_C;
                pk_C_prime = new_pk_C_prime;
            },
            None => {
                // Player aborted before this round.
            }
        }
    }    

    // Compare against libsnark:

}
