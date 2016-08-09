use snark;

pub struct TauPowers {
    acc: snark::Fr,
    tau: snark::Fr
}

impl TauPowers {
    pub fn new(tau: snark::Fr) -> TauPowers {
        TauPowers { acc: snark::Fr::one(), tau: tau }
    }
}

impl Iterator for TauPowers {
    type Item = snark::Fr;

    fn next(&mut self) -> Option<snark::Fr> {
        let tmp = self.acc;
        self.acc = tmp * self.tau;
        Some(tmp)
    }
}

#[test]
fn test_tau_powers() {
    snark::initialize();

    let tau = snark::Fr::random();
    let mut taupowers = TauPowers::new(tau);
    assert!(taupowers.next() == Some(snark::Fr::one()));
    assert!(taupowers.next() == Some(tau));
    assert!(taupowers.next() == Some(tau * tau));
    assert!(taupowers.next() == Some(tau * tau * tau));
}
