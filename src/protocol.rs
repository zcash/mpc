use snark::*;
use spair::*;
use taupowers::*;
use lagrange::*;
use std::collections::HashMap;

#[derive(Clone)]
struct Info<T> {
    tau: T,
    rho_a: T,
    rho_b: T,
    alpha_a: T,
    alpha_b: T,
    alpha_c: T,
    beta: T,
    gamma: T
}

type BlakeHash = [u8; 1];
type Secrets = Info<Fr>;
type Spairs = Info<Spair<G2>>;

impl Secrets {
    fn new() -> Secrets {
        Info {
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
        Info {
            tau: Spair::random(&self.tau),
            rho_a: Spair::random(&self.rho_a),
            rho_b: Spair::random(&self.rho_b),
            alpha_a: Spair::random(&self.alpha_a),
            alpha_b: Spair::random(&self.alpha_b),
            alpha_c: Spair::random(&self.alpha_c),
            beta: Spair::random(&self.beta),
            gamma: Spair::random(&self.gamma)
        }
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
        pk_A: &[G1]) -> Vec<G1>
    {
        pk_A.iter().map(|a| {
            *a * self.secrets.rho_a
        }).collect()
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

        blake2s(&spairs) == self.commitments[i]
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
    // Compute PK_A
    let mut pk_A = at.clone();
    
    for (i, player) in players.iter().enumerate() {
        match *player {
            Some(ref player) => {
                let new_pk_A = player.random_coeffs_part_one(&pk_A);
            },
            None => {
                // Player aborted before this round.
            }
        }
    }    




}
