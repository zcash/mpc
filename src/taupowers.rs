use bn::Fr;
use rand;

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

#[test]
fn test_tau_powers() {
    let rng = &mut rand::thread_rng();

    let tau = Fr::random(rng);
    let mut taupowers = TauPowers::new(tau);
    assert!(taupowers.next() == Some(Fr::one()));
    assert!(taupowers.next() == Some(tau));
    assert!(taupowers.next() == Some(tau * tau));
    assert!(taupowers.next() == Some(tau * tau * tau));
}
