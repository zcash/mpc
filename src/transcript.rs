use bn::*;
use spairs::*;
use snark::*;
use taupowers::*;
use qap::*;
use rand::Rng;

trait State {
    type Metadata;
}

/// We're currently receiving commitments.
struct ReceivingCommitments;

/// We're performing the powers of tau.
struct PowersOfTau {
    commitments: Box<Iterator<Item=BlakeHash>>,
    spairs: Vec<Spairs>,
    prev_g1: Vec<G1>,
    prev_g2: Vec<G2>
}

/// We're performing the first round of random coefficients.
struct RandomCoeffStage1 {
    spairs: Vec<Spairs>,
    powers_of_tau_g1: Vec<G1>,
    at: Vec<G1>,
    bt1: Vec<G1>,
    bt2: Vec<G2>,
    ct: Vec<G1>,
    values: Stage1Values,
    curplayer: usize
}

/// We're performing the second round of random coefficients.
struct RandomCoeffStage2 {
    spairs: Vec<Spairs>,
    powers_of_tau_g1: Vec<G1>,
    coeffs_1: Stage1Values,
    values: Stage2Values,
    curplayer: usize
}

#[derive(Clone)]
struct Stage1Values {
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

#[derive(Clone)]
struct Stage2Values {
    vk_gamma: G2,
    vk_beta_gamma_one: G1,
    vk_beta_gamma_two: G2,
    pk_k: Vec<G1>
}

impl Stage1Values {
    fn new(at: &Vec<G1>, bt1: &Vec<G1>, bt2: &Vec<G2>, ct: &Vec<G1>) -> Self {
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

struct Transcript<'a, R: Rng, S: State> {
    meta: S::Metadata,
    cs: &'a CS,
    rng: R
}

impl<'a, R: Rng> Transcript<'a, R, ReceivingCommitments> {
    fn new(rng: R, cs: &'a CS) -> Self {
        Transcript {
            meta: vec![],
            cs: cs,
            rng: rng
        }
    }

    fn take(&mut self, h: BlakeHash) {
        self.meta.push(h);
    }

    fn next(self) -> Transcript<'a, R, PowersOfTau> {
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
    fn current(&self) -> (Vec<G1>, Vec<G2>) {
        (self.meta.prev_g1.clone(), self.meta.prev_g2.clone())
    }

    fn take(
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
            checkseq(&mut self.rng, g1.iter(), &Spair::new(g2[0], g2[1]).unwrap()) &&
            // Check that all G2 elements are exponentiated the same as G1 elements
            checkseq(&mut self.rng, g2.iter(), &Spair::new(g1[0], g1[1]).unwrap())
        {
            self.meta.prev_g1 = g1;
            self.meta.prev_g2 = g2;

            // Check that the commitment is correct.
            match self.meta.commitments.next() {
                Some(commitment) => {
                    if spairs.hash() != commitment {
                        false
                    } else {
                        self.meta.spairs.push(spairs.clone());

                        true
                    }
                },
                None => false
            }
        } else {
            false
        }
    }

    fn next(self) -> Transcript<'a, R, RandomCoeffStage1> {
        // evaluate QAP for the next round
        let (at, bt1, bt2, ct) = evaluate_qap(&self.meta.prev_g1, &self.meta.prev_g2, &self.cs);

        // initialize pieces of the params for the next round
        let values = Stage1Values::new(&at, &bt1, &bt2, &ct);

        Transcript {
            meta: RandomCoeffStage1 {
                spairs: self.meta.spairs,
                powers_of_tau_g1: self.meta.prev_g1,
                at: at,
                bt1: bt1,
                bt2: bt2,
                ct: ct,
                values: values,
                curplayer: 0
            },
            cs: self.cs,
            rng: self.rng
        }
    }
}

impl<'a, R: Rng> Transcript<'a, R, RandomCoeffStage1> {
    fn current(&self) -> Stage1Values {
        self.meta.values.clone()
    }

    fn take(
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
            !check(
                &mut self.rng,
                self.meta.values.pk_a.iter().zip(new_values.pk_a.iter()),
                &self.meta.spairs[self.meta.curplayer].rho_a()
            ) ||
            !check(
                &mut self.rng,
                self.meta.values.pk_a_prime.iter().zip(new_values.pk_a_prime.iter()),
                &self.meta.spairs[self.meta.curplayer].alpha_a_rho_a()
            ) ||
            !check(
                &mut self.rng,
                self.meta.values.pk_b.iter().zip(new_values.pk_b.iter()),
                &self.meta.spairs[self.meta.curplayer].pB
            ) ||
            !check(
                &mut self.rng,
                self.meta.values.pk_b_temp.iter().zip(new_values.pk_b_temp.iter()),
                &self.meta.spairs[self.meta.curplayer].rho_b()
            ) ||
            !check(
                &mut self.rng,
                self.meta.values.pk_b_prime.iter().zip(new_values.pk_b_prime.iter()),
                &self.meta.spairs[self.meta.curplayer].alpha_b_rho_b()
            ) ||
            !check(
                &mut self.rng,
                self.meta.values.pk_c.iter().zip(new_values.pk_c.iter()),
                &self.meta.spairs[self.meta.curplayer].rho_a_rho_b()
            ) ||
            !check(
                &mut self.rng,
                self.meta.values.pk_c_prime.iter().zip(new_values.pk_c_prime.iter()),
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

    fn next(self) -> Transcript<'a, R, RandomCoeffStage2> {
        let mut pk_k = Vec::with_capacity(self.meta.values.pk_a.len()+3);

        for ((&a, &b), &c) in self.meta.values.pk_a.iter().take(self.meta.values.pk_a.len() - 1)
                                  .zip(self.meta.values.pk_b_temp.iter().take(self.meta.values.pk_b_temp.len() - 1))
                                  .zip(self.meta.values.pk_c.iter().take(self.meta.values.pk_c.len() - 1))
        {
            pk_k.push(a + b + c);
        }

        // Perform Z extention as libsnark does.
        pk_k.push(self.meta.values.pk_a[self.meta.values.pk_a.len() - 1]);
        pk_k.push(self.meta.values.pk_b_temp[self.meta.values.pk_b_temp.len() - 1]);
        pk_k.push(self.meta.values.pk_c[self.meta.values.pk_c.len() - 1]);

        Transcript {
            meta: RandomCoeffStage2 {
                spairs: self.meta.spairs,
                powers_of_tau_g1: self.meta.powers_of_tau_g1,
                coeffs_1: self.meta.values,
                values: Stage2Values {
                    vk_gamma: G2::one(),
                    vk_beta_gamma_one: G1::one(),
                    vk_beta_gamma_two: G2::one(),
                    pk_k: pk_k
                },
                curplayer: 0
            },
            cs: self.cs,
            rng: self.rng
        }
    }
}

impl<'a, R: Rng> Transcript<'a, R, RandomCoeffStage2> {
    fn current(&self) -> Stage2Values {
        self.meta.values.clone()
    }

    fn take(
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
            !check(
                &mut self.rng,
                self.meta.values.pk_k.iter().zip(new_values.pk_k.iter()),
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

    fn keypair(&self) -> Keypair {
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
    fn mul_all_by<G: Group>(v: &mut [G], c: Fr) {
        for g in v {
            *g = *g * c;
        }
    }

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
        let (mut cur_g1, mut cur_g2) = transcript.current();

        for (secrets, spairs) in secrets.iter().zip(spairs.iter()) {
            for ((g1, g2), tp) in cur_g1.iter_mut().zip(cur_g2.iter_mut()).zip(TauPowers::new(secrets.tau)) {
                *g1 = *g1 * tp;
                *g2 = *g2 * tp;
            }

            assert!(transcript.take(spairs.clone(), cur_g1.clone(), cur_g2.clone()));
        }
    }

    let mut transcript = transcript.next();

    // Random coeff stage 1
    {
        let mut cur_values = transcript.current();

        for secrets in secrets.iter() {
            // Contribute to verification key
            cur_values.vk_a = cur_values.vk_a * secrets.alpha_a;
            cur_values.vk_b = cur_values.vk_b * secrets.alpha_b;
            cur_values.vk_c = cur_values.vk_c * secrets.alpha_c;
            cur_values.vk_z = cur_values.vk_z * (secrets.rho_a * secrets.rho_b);
            // Contribute to proving key
            mul_all_by(&mut cur_values.pk_a, secrets.rho_a);
            mul_all_by(&mut cur_values.pk_a_prime, (secrets.rho_a * secrets.alpha_a));
            mul_all_by(&mut cur_values.pk_b, secrets.rho_b);
            mul_all_by(&mut cur_values.pk_b_temp, secrets.rho_b);
            mul_all_by(&mut cur_values.pk_b_prime, (secrets.rho_b * secrets.alpha_b));
            mul_all_by(&mut cur_values.pk_c, (secrets.rho_a * secrets.rho_b));
            mul_all_by(&mut cur_values.pk_c_prime, (secrets.rho_a * secrets.rho_b * secrets.alpha_c));

            assert!(transcript.take(cur_values.clone()));
        }
    }

    let mut transcript = transcript.next();

    // Random coeff stage 2
    {
        let mut cur_values = transcript.current();

        for secrets in secrets.iter() {
            let betagamma = secrets.beta * secrets.gamma;
            cur_values.vk_gamma = cur_values.vk_gamma * secrets.gamma;
            cur_values.vk_beta_gamma_one = cur_values.vk_beta_gamma_one * betagamma;
            cur_values.vk_beta_gamma_two = cur_values.vk_beta_gamma_two * betagamma;
            mul_all_by(&mut cur_values.pk_k, secrets.beta);

            assert!(transcript.take(cur_values.clone()));
        }
    }

    // Construct the final keypair:
    let keypair = transcript.keypair();

    {
        // Compare with libsnark
        let mut tau = Fr::one();
        let mut alpha_a = Fr::one();
        let mut alpha_b = Fr::one();
        let mut alpha_c = Fr::one();
        let mut rho_a = Fr::one();
        let mut rho_b = Fr::one();
        let mut beta = Fr::one();
        let mut gamma = Fr::one();

        for s in secrets {
            tau = tau * s.tau;
            alpha_a = alpha_a * s.alpha_a;
            alpha_b = alpha_b * s.alpha_b;
            alpha_c = alpha_c * s.alpha_c;
            rho_a = rho_a * s.rho_a;
            rho_b = rho_b * s.rho_b;
            beta = beta * s.beta;
            gamma = gamma * s.gamma;
        }

        let expected_keypair = Keypair::generate(
            &cs,
            &tau,
            &alpha_a,
            &alpha_b,
            &alpha_c,
            &rho_a,
            &rho_b,
            &beta,
            &gamma
        );

        assert!(expected_keypair == keypair);
    }
}
