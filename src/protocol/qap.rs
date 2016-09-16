use bn::*;
use snark::*;
use super::multicore::*;

/// Evaluates the QAP A, B and C polynomials at tau given the powers of tau.
/// Converts the powers of tau in G1 and G2 into the lagrange basis with an FFT
/// Extends with Z(tau) as (effectively) done in libsnark.
pub fn evaluate(g1_powers: &[G1], g2_powers: &[G2], cs: &CS) -> (Vec<G1>, Vec<G1>, Vec<G2>, Vec<G1>)
{
    assert_eq!(g1_powers.len(), cs.d+1);
    assert_eq!(g2_powers.len(), cs.d+1);

    let lc1 = lagrange_coeffs(&g1_powers[0..cs.d], cs.omega);
    let lc2 = lagrange_coeffs(&g2_powers[0..cs.d], cs.omega);

    let (mut at, mut bt1, mut bt2, mut ct) = evaluate_qap_polynomials(&lc1, &lc2, cs);

    // Extention of Z(tau)
    at.push(g1_powers[cs.d] - G1::one());
    bt1.push(g1_powers[cs.d] - G1::one());
    bt2.push(g2_powers[cs.d] - G2::one());
    ct.push(g1_powers[cs.d] - G1::one());

    (at, bt1, bt2, ct)
}

fn evaluate_qap_polynomials(lc1: &[G1], lc2: &[G2], cs: &CS) -> (Vec<G1>, Vec<G1>, Vec<G2>, Vec<G1>)
{
    assert_eq!(lc1.len(), cs.d);
    assert_eq!(lc2.len(), cs.d);

    let mut at = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();
    let mut bt1 = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();
    let mut bt2 = (0..cs.num_vars).map(|_| G2::zero()).collect::<Vec<_>>();
    let mut ct = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();

    cs.eval(&lc1, &lc2, &mut at, &mut bt1, &mut bt2, &mut ct);

    (at, bt1, bt2, ct)
}

fn lagrange_coeffs<G: Group>(v: &[G], omega: Fr) -> Vec<G>
{
    assert!(v.len() >= 2);
    assert_eq!((v.len() / 2) * 2, v.len());

    let overd = Fr::from_str(&format!("{}", v.len())).unwrap().inverse().unwrap();
    let mut tmp = fft(v, omega, ::THREADS);
    tmp.reverse(); // coefficients are in reverse

    mul_all_by(&mut tmp, overd);

    tmp
}

fn fft<G: Group>(v: &[G], omega: Fr, threads: usize) -> Vec<G>
{
    if v.len() == 2 {
        vec![
            v[0] + v[1] * omega,
            v[0] + v[1]
        ]
    } else {
        let d2 = v.len() / 2;
        let mut evens = Vec::with_capacity(d2);
        let mut odds = Vec::with_capacity(d2);

        for (i, x) in v.iter().enumerate() {
            if i % 2 == 0 {
                evens.push(*x);
            } else {
                odds.push(*x);
            }
        }

        let o2 = omega * omega;
        let (evens, odds) = if threads < 2 {
            (fft(&evens, o2, 1), fft(&odds, o2, 1))
        } else {
            use std::sync::mpsc::channel;
            use std::thread;

            let (tx_evens, rx_evens) = channel();
            let (tx_odds, rx_odds) = channel();

            thread::spawn(move || {
                tx_evens.send(fft(&evens, o2, threads/2)).unwrap();
            });

            thread::spawn(move || {
                tx_odds.send(fft(&odds, o2, threads/2)).unwrap();
            });

            (rx_evens.recv().unwrap(), rx_odds.recv().unwrap())
        };

        let mut acc = omega;
        let mut res = Vec::with_capacity(v.len());
        for i in 0..v.len() {
            res.push(evens[i%d2] + odds[i%d2] * acc);
            acc = acc * omega;
        }

        res
    }
}

#[test]
fn compare_to_libsnark() {
    pub struct TauPowers {
        acc: Fr,
        tau: Fr
    }

    impl TauPowers {
        pub fn new(tau: Fr) -> TauPowers {
            TauPowers { acc: Fr::one(), tau: tau }
        }
    }

    impl Iterator for TauPowers {
        type Item = Fr;

        fn next(&mut self) -> Option<Fr> {
            let tmp = self.acc;
            self.acc = tmp * self.tau;
            Some(tmp)
        }
    }

    {
        let rng = &mut ::rand::thread_rng();

        let tau = Fr::random(rng);
        let mut taupowers = TauPowers::new(tau);
        assert!(taupowers.next() == Some(Fr::one()));
        assert!(taupowers.next() == Some(tau));
        assert!(taupowers.next() == Some(tau * tau));
        assert!(taupowers.next() == Some(tau * tau * tau));
    }

    let rng = &mut ::rand::thread_rng();

    // Get the QAP degree and omega (for FFT evaluation)
    let cs = CS::dummy();

    // Sample a random tau
    let tau = Fr::random(rng);

    // Generate powers of tau in G1, from 0 to d exclusive of d
    let powers_of_tau_g1 = TauPowers::new(tau).take(cs.d).map(|e| G1::one() * e).collect::<Vec<_>>();
    // Generate powers of tau in G2, from 0 to d exclusive of d
    let powers_of_tau_g2 = TauPowers::new(tau).take(cs.d).map(|e| G2::one() * e).collect::<Vec<_>>();

    // Perform FFT to compute lagrange coeffs in G1/G2
    let lc1 = lagrange_coeffs(&powers_of_tau_g1, cs.omega);
    let lc2 = lagrange_coeffs(&powers_of_tau_g2, cs.omega);

    {
        // Perform G1 FFT with wrong omega
        let lc1 = lagrange_coeffs(&powers_of_tau_g1, Fr::random(rng));
        assert!(!cs.test_compare_tau(&lc1, &lc2, &tau));
    }
    {
        // Perform G2 FFT with wrong omega
        let lc2 = lagrange_coeffs(&powers_of_tau_g2, Fr::random(rng));
        assert!(!cs.test_compare_tau(&lc1, &lc2, &tau));
    }

    // Compare against libsnark
    assert!(cs.test_compare_tau(&lc1, &lc2, &tau));

    // Wrong tau
    assert!(!cs.test_compare_tau(&lc1, &lc2, &Fr::random(rng)));

    let (at, bt1, bt2, ct) = evaluate_qap_polynomials(&lc1, &lc2, &cs);

    // Compare evaluation with libsnark
    assert!(cs.test_eval(&tau, &at, &bt1, &bt2, &ct));

    // Wrong tau
    assert!(!cs.test_eval(&Fr::random(rng), &at, &bt1, &bt2, &ct));

    // Wrong polynomials
    assert!(!cs.test_eval(&Fr::random(rng), &bt1, &bt1, &bt2, &ct));
}
