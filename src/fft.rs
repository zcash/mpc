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
        let (d, omega) = getqap();

        // Sample a random tau
        let tau = Fr::random();

        // Generate powers of tau in G1, from 0 to d exclusive of d
        let powers_of_tau = TauPowers::new(tau).take(d).map(|e| G1::one() * e).collect::<Vec<_>>();

        let overd = Fr::from_str(&format!("{}", d)).inverse();
        let lc = fft(&powers_of_tau, omega) // omit tau^d
                    .into_iter()
                    .rev() // coefficients are in reverse
                    .map(|e| e * overd) // divide by d
                    .collect::<Vec<_>>();

        // Compare against libsnark
        assert!(compare_tau(&lc, &tau));

        // Wrong tau
        assert!(!compare_tau(&lc, &Fr::random()));
    }
}
