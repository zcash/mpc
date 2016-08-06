use snark::*;
use randompowers::*;

#[derive(Debug)]
pub enum ProtocolError {
    InvalidTauPowersSize
}

pub struct Samples<T> {
    tau: T,
    rho_a: T,
    rho_b: T,
    alpha_a: T,
    alpha_b: T,
    alpha_c: T,
    beta: T,
    gamma: T
}

pub struct Player {
    secrets: Samples<Fr>,
    pub d: usize,
    omega: Fr,
    cs: CS
}

impl Player {
    pub fn new() -> Player {
        let (d, omega, cs) = getqap();

        Player {
            secrets: Samples {
                tau: Fr::random_nonzero(),
                rho_a: Fr::random_nonzero(),
                rho_b: Fr::random_nonzero(),
                alpha_a: Fr::random_nonzero(),
                alpha_b: Fr::random_nonzero(),
                alpha_c: Fr::random_nonzero(),
                beta: Fr::random_nonzero(),
                gamma: Fr::random_nonzero()
            },
            d: d,
            omega: omega,
            cs: cs
        }
    }

    pub fn spairs<G: Group>(&self) -> Samples<Spair<G>> {
        Samples {
            tau: Spair::random(&self.secrets.tau),
            rho_a: Spair::random(&self.secrets.rho_a),
            rho_b: Spair::random(&self.secrets.rho_b),
            alpha_a: Spair::random(&self.secrets.alpha_a),
            alpha_b: Spair::random(&self.secrets.alpha_b),
            alpha_c: Spair::random(&self.secrets.alpha_c),
            beta: Spair::random(&self.secrets.beta),
            gamma: Spair::random(&self.secrets.gamma)
        }
    }

    pub fn randompowers(&self, v1: &[G1], v2: &[G2]) -> Result<(Vec<G1>, Vec<G2>), ProtocolError> {
        if (v1.len() != v2.len()) || (v1.len() != self.d+1) {
            return Err(ProtocolError::InvalidTauPowersSize)
        }

        let mut t1 = Vec::with_capacity(self.d+1);
        let mut t2 = Vec::with_capacity(self.d+1);

        let mut tp = Fr::one();
        for i in 0..self.d+1 {
            t1.push(v1[i] * tp);
            t2.push(v2[i] * tp);

            tp = tp * self.secrets.tau;
        }

        Ok((t1, t2))
    }
}

pub fn verify_randompowers(
    current: &(Vec<G1>, Vec<G2>),
    last: Option<&(Vec<G1>, Vec<G2>)>,
    rp: &Spair<G2>
) -> bool {
    current.0[0] == G1::one() &&
    current.1[0] == G2::one() &&
    match last {
        Some(last) => {
            checkseq(current.0.iter(), &Spair::new(&current.1[0], &current.1[1])) &&
            checkseq(current.1.iter(), &Spair::new(&current.0[0], &current.0[1])) &&
            same_power(&Spair::new(&last.0[1], &current.0[1]), rp)
        },
        None => {
            checkseq(current.0.iter(), rp) &&
            checkseq(current.1.iter(), &Spair::new(&current.0[0], &current.0[1]))
        }
    }
}

#[test]
fn randompowers_test() {
    initialize();

    const NUM_PARTIES: usize = 3;

    // All parties should initialize with their secret randomness
    let parties: Vec<Player> = (0..NUM_PARTIES).map(|_| Player::new()).collect();
    // All parties should reveal their s-pairs
    let spairs: Vec<Samples<Spair<G2>>> = parties.iter().map(|p| p.spairs()).collect();
    
    let mut transcript = vec![];

    for (i, p) in parties.iter().enumerate() {
        use std::iter::repeat;

        if i == 0 {
            let v1 = repeat(G1::one()).take(p.d + 1).collect::<Vec<_>>();
            let v2 = repeat(G2::one()).take(p.d + 1).collect::<Vec<_>>();
            transcript.push(p.randompowers(&v1, &v2).unwrap());
        } else {
            let v = p.randompowers(&transcript[i-1].0, &transcript[i-1].1).unwrap();
            transcript.push(v);
        }
    }

    // Verification
    for i in 0..NUM_PARTIES {
        assert!(verify_randompowers(&transcript[i],
                                    if i == 0 { None } else { Some(&transcript[i-1]) },
                                    &spairs[i].tau));
    }
}
