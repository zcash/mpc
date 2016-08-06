use snark::{Group, Fr};

pub fn fft<G: Group>(v: &[G], omega: Fr) -> Vec<G>
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
        let evens = fft(&evens, o2);
        let odds = fft(&odds, o2);

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
    use super::fft;
    use snark::*;
    use util::*;

    #[test]
    fn compare_to_libsnark() {
        initialize();

        // Get the QAP degree and omega (for FFT evaluation)
        let (d, num_vars, omega, cs) = getqap();

        // Sample a random tau
        let tau = Fr::random();

        // Generate powers of tau in G1, from 0 to d exclusive of d
        let powers_of_tau_g1 = TauPowers::new(tau).take(d).map(|e| G1::one() * e).collect::<Vec<_>>();
        let powers_of_tau_g2 = TauPowers::new(tau).take(d).map(|e| G2::one() * e).collect::<Vec<_>>();

        let overd = Fr::from_str(&format!("{}", d)).inverse();
        let lc1 = fft(&powers_of_tau_g1, omega) // omit tau^d
                     .into_iter()
                     .rev() // coefficients are in reverse
                     .map(|e| e * overd) // divide by d
                     .collect::<Vec<_>>();
        let lc2 = fft(&powers_of_tau_g2, omega) // omit tau^d
                     .into_iter()
                     .rev() // coefficients are in reverse
                     .map(|e| e * overd) // divide by d
                     .collect::<Vec<_>>();

        // Compare against libsnark
        assert!(compare_tau(&lc1, &tau, &cs));

        // Wrong tau
        assert!(!compare_tau(&lc1, &Fr::random(), &cs));

        // Evaluate At, Ct in G1 and Bt in G1/G2
        let mut At = (0..num_vars).map(|_| G1::zero()).collect::<Vec<_>>();
        let mut Bt1 = (0..num_vars).map(|_| G1::zero()).collect::<Vec<_>>();
        let mut Bt2 = (0..num_vars).map(|_| G2::zero()).collect::<Vec<_>>();
        let mut Ct = (0..num_vars).map(|_| G1::zero()).collect::<Vec<_>>();

        cs.eval(&lc1, &lc2, &mut At, &mut Bt1, &mut Bt2, &mut Ct);

        // Compare evaluation with libsnark
        assert!(cs.test_eval(&tau, &At, &Bt1, &Bt2, &Ct));

        // Wrong tau
        assert!(!cs.test_eval(&Fr::random(), &At, &Bt1, &Bt2, &Ct));

        // Wrong polynomials
        assert!(!cs.test_eval(&Fr::random(), &Bt1, &Bt1, &Bt2, &Ct));
    }
}
