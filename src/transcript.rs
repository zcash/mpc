use bn::*;
use spairs::*;
use snark::*;
use qap::*;
use rand::Rng;

pub trait State {
    type Metadata;
}

/// We're currently receiving commitments.
pub struct ReceivingCommitments;

/// We're performing the powers of tau.
pub struct PowersOfTau {
    commitments: Box<Iterator<Item=BlakeHash>>,
    spairs: Vec<Spairs>,
    prev_g1: Vec<G1>,
    prev_g2: Vec<G2>
}

/// We're performing the first round of random coefficients.
pub struct RandomCoeffStage1 {
    spairs: Vec<Spairs>,
    powers_of_tau_g1: Vec<G1>,
    values: Stage1Values,
    curplayer: usize
}

/// We're performing the second round of random coefficients.
pub struct RandomCoeffStage2 {
    spairs: Vec<Spairs>,
    powers_of_tau_g1: Vec<G1>,
    coeffs_1: Stage1Values,
    values: Stage2Values,
    curplayer: usize
}

impl State for ReceivingCommitments {
    type Metadata = Vec<BlakeHash>;
}

impl State for PowersOfTau {
    type Metadata = Self;
}

impl State for RandomCoeffStage1 {
    type Metadata = Self;
}

impl State for RandomCoeffStage2 {
    type Metadata = Self;
}

pub struct Transcript<'a, R: Rng, S: State> {
    meta: S::Metadata,
    cs: &'a CS,
    rng: R
}

impl<'a, R: Rng> Transcript<'a, R, ReceivingCommitments> {
    pub fn new(rng: R, cs: &'a CS) -> Self {
        Transcript {
            meta: vec![],
            cs: cs,
            rng: rng
        }
    }

    pub fn take(&mut self, h: BlakeHash) {
        self.meta.push(h);
    }

    pub fn next(self) -> Transcript<'a, R, PowersOfTau> {
        Transcript {
            meta: PowersOfTau {
                commitments: Box::new(self.meta.into_iter()) as Box<Iterator<Item=BlakeHash>>,
                spairs: vec![],
                prev_g1: (0..self.cs.d+1).map(|_| G1::one()).collect(),
                prev_g2: (0..self.cs.d+1).map(|_| G2::one()).collect()
            },
            cs: self.cs,
            rng: self.rng
        }
    }
}

impl<'a, R: Rng> Transcript<'a, R, PowersOfTau> {
    pub fn current(&self) -> (&Vec<G1>, &Vec<G2>) {
        (&self.meta.prev_g1, &self.meta.prev_g2)
    }

    pub fn take(
        &mut self,
        spairs: Spairs,
        g1: Vec<G1>,
        g2: Vec<G2>
    ) -> bool
    {
        if
            g1.len() == self.meta.prev_g1.len() &&
            g2.len() == self.meta.prev_g2.len() &&
            g1[0] == G1::one() &&
            g2[0] == G2::one() &&
            !g1[1].is_zero() &&
            !g2[1].is_zero() &&
            // The player is supposed to multiply the i'th element
            // by tau^i. Let's check one non-degenerate case in G1.
            same_power(
                &Spair::new(self.meta.prev_g1[1], g1[1]).unwrap(),
                &spairs.tau
            ) &&
            // Check that all G1 elements are exponentiated the same as G2 elements
            checkseq(&mut self.rng, &g1, &Spair::new(g2[0], g2[1]).unwrap()) &&
            // Check that all G2 elements are exponentiated the same as G1 elements
            checkseq(&mut self.rng, &g2, &Spair::new(g1[0], g1[1]).unwrap())
        {
            self.meta.prev_g1 = g1;
            self.meta.prev_g2 = g2;

            // Check that the commitment is correct.
            match self.meta.commitments.next() {
                Some(commitment) => {
                    if spairs.hash() != commitment {
                        false
                    } else {
                        self.meta.spairs.push(spairs);

                        true
                    }
                },
                None => false
            }
        } else {
            false
        }
    }

    pub fn next(self) -> Transcript<'a, R, RandomCoeffStage1> {
        // evaluate QAP for the next round
        let (at, bt1, bt2, ct) = evaluate_qap(&self.meta.prev_g1, &self.meta.prev_g2, &self.cs);

        // initialize pieces of the params for the next round
        let values = Stage1Values::new(&at, &bt1, &bt2, &ct);

        Transcript {
            meta: RandomCoeffStage1 {
                spairs: self.meta.spairs,
                powers_of_tau_g1: self.meta.prev_g1,
                values: values,
                curplayer: 0
            },
            cs: self.cs,
            rng: self.rng
        }
    }
}

impl<'a, R: Rng> Transcript<'a, R, RandomCoeffStage1> {
    pub fn current(&self) -> &Stage1Values {
        &self.meta.values
    }

    pub fn take(
        &mut self,
        new_values: Stage1Values
    ) -> bool
    {
        if
            new_values.vk_a.is_zero() ||
            new_values.vk_b.is_zero() ||
            new_values.vk_c.is_zero() ||
            new_values.vk_z.is_zero() ||
            // Sizes need to match up
            new_values.pk_a.len() != self.meta.values.pk_a.len() ||
            new_values.pk_a_prime.len() != self.meta.values.pk_a_prime.len() ||
            new_values.pk_b.len() != self.meta.values.pk_b.len() ||
            new_values.pk_b_temp.len() != self.meta.values.pk_b_temp.len() ||
            new_values.pk_b_prime.len() != self.meta.values.pk_b_prime.len() ||
            new_values.pk_c.len() != self.meta.values.pk_c.len() ||
            new_values.pk_c_prime.len() != self.meta.values.pk_c_prime.len() ||
            // Check parts of the verification key
            !same_power(
                &Spair::new(self.meta.values.vk_a, new_values.vk_a).unwrap(),
                &self.meta.spairs[self.meta.curplayer].aA
            ) ||
            !same_power(
                &Spair::new(self.meta.values.vk_b, new_values.vk_b).unwrap(),
                &self.meta.spairs[self.meta.curplayer].alpha_b()
            ) ||
            !same_power(
                &Spair::new(self.meta.values.vk_c, new_values.vk_c).unwrap(),
                &self.meta.spairs[self.meta.curplayer].aC
            ) ||
            !same_power(
                &Spair::new(self.meta.values.vk_z, new_values.vk_z).unwrap(),
                &self.meta.spairs[self.meta.curplayer].pApB
            ) ||
            // Check parts of the proving key
            !checkvec(
                &mut self.rng,
                &self.meta.values.pk_a,
                &new_values.pk_a,
                &self.meta.spairs[self.meta.curplayer].rho_a()
            ) ||
            !checkvec(
                &mut self.rng,
                &self.meta.values.pk_a_prime,
                &new_values.pk_a_prime,
                &self.meta.spairs[self.meta.curplayer].alpha_a_rho_a()
            ) ||
            !checkvec(
                &mut self.rng,
                &self.meta.values.pk_b,
                &new_values.pk_b,
                &self.meta.spairs[self.meta.curplayer].pB
            ) ||
            !checkvec(
                &mut self.rng,
                &self.meta.values.pk_b_temp,
                &new_values.pk_b_temp,
                &self.meta.spairs[self.meta.curplayer].rho_b()
            ) ||
            !checkvec(
                &mut self.rng,
                &self.meta.values.pk_b_prime,
                &new_values.pk_b_prime,
                &self.meta.spairs[self.meta.curplayer].alpha_b_rho_b()
            ) ||
            !checkvec(
                &mut self.rng,
                &self.meta.values.pk_c,
                &new_values.pk_c,
                &self.meta.spairs[self.meta.curplayer].rho_a_rho_b()
            ) ||
            !checkvec(
                &mut self.rng,
                &self.meta.values.pk_c_prime,
                &new_values.pk_c_prime,
                &self.meta.spairs[self.meta.curplayer].alpha_c_rho_a_rho_b()
            )
        {
            self.meta.curplayer += 1;
            return false;
        } else {
            self.meta.values = new_values;
            self.meta.curplayer += 1;
            return true;
        }
    }

    pub fn next(self) -> Transcript<'a, R, RandomCoeffStage2> {
        let stage2 = Stage2Values::new(
            &self.meta.values.pk_a,
            &self.meta.values.pk_b_temp,
            &self.meta.values.pk_c
        );

        Transcript {
            meta: RandomCoeffStage2 {
                spairs: self.meta.spairs,
                powers_of_tau_g1: self.meta.powers_of_tau_g1,
                coeffs_1: self.meta.values,
                values: stage2,
                curplayer: 0
            },
            cs: self.cs,
            rng: self.rng
        }
    }
}

impl<'a, R: Rng> Transcript<'a, R, RandomCoeffStage2> {
    pub fn current(&self) -> &Stage2Values {
        &self.meta.values
    }

    pub fn take(
        &mut self,
        new_values: Stage2Values
    ) -> bool
    {
        if 
            new_values.vk_gamma.is_zero() ||
            new_values.vk_beta_gamma_one.is_zero() ||
            new_values.vk_beta_gamma_two.is_zero() ||
            new_values.pk_k.len() != self.meta.values.pk_k.len() ||
            !same_power(
                &Spair::new(self.meta.values.vk_gamma, new_values.vk_gamma).unwrap(),
                &self.meta.spairs[self.meta.curplayer].gamma
            ) ||
            !same_power(
                &Spair::new(self.meta.values.vk_beta_gamma_one, new_values.vk_beta_gamma_one).unwrap(),
                &self.meta.spairs[self.meta.curplayer].beta_gamma()
            ) ||
            !same_power(
                &Spair::new(self.meta.values.vk_beta_gamma_two, new_values.vk_beta_gamma_two).unwrap(),
                &Spair::new(self.meta.values.vk_beta_gamma_one, new_values.vk_beta_gamma_one).unwrap()
            ) ||
            !checkvec(
                &mut self.rng,
                &self.meta.values.pk_k,
                &new_values.pk_k,
                &self.meta.spairs[self.meta.curplayer].beta()
            )
        {
            self.meta.curplayer += 1;
            return false;
        } else {
            self.meta.values = new_values;
            self.meta.curplayer += 1;
            return true;
        }
    }

    pub fn keypair(&self) -> Keypair {
        Keypair::from(
            &self.cs,
            &self.meta.coeffs_1.pk_a,
            &self.meta.coeffs_1.pk_a_prime,
            &self.meta.coeffs_1.pk_b,
            &self.meta.coeffs_1.pk_b_prime,
            &self.meta.coeffs_1.pk_c,
            &self.meta.coeffs_1.pk_c_prime,
            &self.meta.values.pk_k,
            &self.meta.powers_of_tau_g1,
            &self.meta.coeffs_1.vk_a,
            &self.meta.coeffs_1.vk_b,
            &self.meta.coeffs_1.vk_c,
            &self.meta.values.vk_gamma,
            &self.meta.values.vk_beta_gamma_one,
            &self.meta.values.vk_beta_gamma_two,
            &self.meta.coeffs_1.vk_z
        )
    }
}

#[test]
fn mpc_simulation() {
    let cs = CS::dummy();

    let rng = &mut ::rand::thread_rng();
    const PARTIES: usize = 3;

    let secrets = (0..PARTIES).map(|_| Secrets::new(rng)).collect::<Vec<_>>();
    let spairs = secrets.iter().map(|s| s.spairs(rng)).collect::<Vec<_>>();
    let commitments = spairs.iter().map(|s| s.hash()).collect::<Vec<_>>();

    let mut transcript = Transcript::new(rng, &cs);
    for i in &commitments {
        transcript.take(*i);
    }

    let mut transcript = transcript.next();

    // Random powers protocol
    {
        for (secrets, spairs) in secrets.iter().zip(spairs.iter()) {
            let (mut cur_g1, mut cur_g2) = {
                let (cur_g1, cur_g2) = transcript.current();
                (cur_g1.clone(), cur_g2.clone())
            };

            secrets.taupowers(&mut cur_g1, &mut cur_g2);

            assert!(transcript.take(spairs.clone(), cur_g1, cur_g2));
        }
    }

    let mut transcript = transcript.next();

    // Random coeff stage 1
    {
        for secrets in secrets.iter() {
            let mut cur_values = transcript.current().clone();

            secrets.stage1(&mut cur_values);

            assert!(transcript.take(cur_values));
        }
    }

    let mut transcript = transcript.next();

    // Random coeff stage 2
    {
        for secrets in secrets.iter() {
            let mut cur_values = transcript.current().clone();
            
            secrets.stage2(&mut cur_values);

            assert!(transcript.take(cur_values.clone()));
        }
    }

    // Construct the final keypair:
    let keypair = transcript.keypair();

    {
        let mut acc = Secrets::new_blank();

        for s in &secrets {
            acc.multiply(s);
        }

        let expected_keypair = acc.keypair(&cs);

        assert!(expected_keypair == keypair);
    }
}
