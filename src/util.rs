use snark::Fr;

pub struct Sequences<'a, T: 'a, I: Iterator<Item=&'a T>> {
    v: I,
    last: Option<&'a T>
}

impl<'a, T: 'a, I: Iterator<Item=&'a T>> Sequences<'a, T, I> {
    pub fn new(v: I) -> Self {
        Sequences { v: v, last: None }
    }
}

impl<'a, T: 'a, I: Iterator<Item=&'a T>> Iterator for Sequences<'a, T, I> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<(&'a T, &'a T)> {
        match (self.last, self.v.next()) {
            (Some(a), Some(b)) => {
                self.last = Some(b);
                Some((a, b))
            },
            (None, Some(b)) => {
                self.last = Some(b);
                self.next()
            },
            _ => None
        }
    }
}

#[test]
fn test_sequences() {
    let a = vec![10, 57, 34, 12];
    let b: Vec<(&usize, &usize)> = Sequences::new(a.iter()).collect();
    let expected = vec![(&a[0], &a[1]), (&a[1], &a[2]), (&a[2], &a[3])];
    assert_eq!(b, expected);
}



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
