use snark::{Group, Fr};
use crossbeam;

pub fn lagrange_coeffs<G: Group>(v: &[G], omega: Fr) -> Vec<G>
{
    assert_eq!((v.len() / 2) * 2, v.len());
    const THREADS: usize = 8;

    let overd = Fr::from_str(&format!("{}", v.len())).inverse();
    let mut tmp = fft(v, omega, THREADS);
    tmp.reverse(); // coefficients are in reverse

    crossbeam::scope(|scope| {
        for i in tmp.chunks_mut(v.len() / THREADS) {
            scope.spawn(move || {
                for i in i {
                    *i = *i * overd;
                }
            });
        }
    });

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

#[cfg(test)]
mod test {
    use super::lagrange_coeffs;
    use snark::*;
    use taupowers::*;

    #[test]
    fn compare_to_libsnark() {
        initialize();

        // Get the QAP degree and omega (for FFT evaluation)
        let cs = getcs();

        // Sample a random tau
        let tau = Fr::random();

        // Generate powers of tau in G1, from 0 to d exclusive of d
        let powers_of_tau_g1 = TauPowers::new(tau).take(cs.d).map(|e| G1::one() * e).collect::<Vec<_>>();
        // Generate powers of tau in G2, from 0 to d exclusive of d
        let powers_of_tau_g2 = TauPowers::new(tau).take(cs.d).map(|e| G2::one() * e).collect::<Vec<_>>();

        // Perform FFT to compute lagrange coeffs in G1/G2
        let lc1 = lagrange_coeffs(&powers_of_tau_g1, cs.omega);
        let lc2 = lagrange_coeffs(&powers_of_tau_g2, cs.omega);

        {
            // Perform G1 FFT with wrong omega
            let lc1 = lagrange_coeffs(&powers_of_tau_g1, Fr::random());
            assert!(!cs.test_compare_tau(&lc1, &lc2, &tau));
        }
        {
            // Perform G2 FFT with wrong omega
            let lc2 = lagrange_coeffs(&powers_of_tau_g2, Fr::random());
            assert!(!cs.test_compare_tau(&lc1, &lc2, &tau));
        }

        // Compare against libsnark
        assert!(cs.test_compare_tau(&lc1, &lc2, &tau));

        // Wrong tau
        assert!(!cs.test_compare_tau(&lc1, &lc2, &Fr::random()));

        // Evaluate At, Ct in G1 and Bt in G1/G2
        let mut at = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();
        let mut bt1 = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();
        let mut bt2 = (0..cs.num_vars).map(|_| G2::zero()).collect::<Vec<_>>();
        let mut ct = (0..cs.num_vars).map(|_| G1::zero()).collect::<Vec<_>>();

        cs.eval(&lc1, &lc2, &mut at, &mut bt1, &mut bt2, &mut ct);

        // Compare evaluation with libsnark
        assert!(cs.test_eval(&tau, &at, &bt1, &bt2, &ct));

        // Wrong tau
        assert!(!cs.test_eval(&Fr::random(), &at, &bt1, &bt2, &ct));

        // Wrong polynomials
        assert!(!cs.test_eval(&Fr::random(), &bt1, &bt1, &bt2, &ct));
    }
}
