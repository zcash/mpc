use snark::*;
use util::Sequences;

struct Spair<G: Group> {
    p: G,
    q: G
}

impl<G: Group> Spair<G> {
    fn random(s: &Fr) -> Self {
        let mut p = G::zero();

        while p.is_zero() {
            p = G::random();
        }

        Spair {
            p: p,
            q: p * (*s)
        }
    }

    fn new(p: &G, q: &G) -> Self {
        if p.is_zero() {
            panic!("tried to initialize spair with zero base")
        }

        Spair {
            p: *p,
            q: *q
        }
    }
}

fn same_power<Group1: Group, Group2: Group>(a: &Spair<Group1>, b: &Spair<Group2>) -> bool
where Group1: Pairing<Group2> {
    pairing(&a.p, &b.q) == pairing(&a.q, &b.p)
}

/// This performs a check to see if a large number of (p,q) pairs in G
/// have the same power, with only one pairing.
fn check<'a,
         Group1: Group,
         Group2: Group,
         I: IntoIterator<Item=(&'a Group1, &'a Group1)>>
         (i: I, a: &Spair<Group2>) -> bool
where Group1: Pairing<Group2>
{
    let mut p = Group1::zero();
    let mut q = Group1::zero();

    for v in i {
        let alpha = Fr::random_nonzero();
        p = p + *v.0 * alpha;
        q = q + *v.1 * alpha;
    }

    if p.is_zero() { return false; }

    same_power(&Spair::new(&p, &q), &a)
}

fn checkvec<'a,
            Group1: Group,
            Group2: Group,
            I: IntoIterator<Item=&'a Spair<Group1>>>
            (i: I, a: &Spair<Group2>) -> bool
where Group1: Pairing<Group2>
{
    check(i.into_iter().map(|s| (&s.p, &s.q)), a)
}

fn checkseq<'a,
            Group1: Group,
            Group2: Group,
            I: Iterator<Item=&'a Group1>>
            (i: I, a: &Spair<Group2>) -> bool
where Group1: Pairing<Group2>
{
    check(Sequences::new(i), a)
}

struct TauPowers {
    acc: Fr,
    tau: Fr
}

impl TauPowers {
    fn new(tau: Fr) -> TauPowers {
        TauPowers { acc: Fr::from_str("1"), tau: tau }
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
fn randompowers_simulation() {
    initialize();

    let parties = 3;
    let d = 1024;

    let mut messages: Vec<(Spair<G2>, Vec<G1>, Vec<G2>)> = vec![];
    messages.reserve(parties);

    for i in 0..parties {
        let tau = Fr::random_nonzero();
        let rp = Spair::random(&tau);

        if i == 0 {
            messages.push((
                rp,
                TauPowers::new(tau).map(|p| G1::one() * p).take(d).collect(),
                TauPowers::new(tau).map(|p| G2::one() * p).take(d).collect()
            ));
        } else {
            let v1 = messages[i-1].1.iter().zip(TauPowers::new(tau)).map(|(b, p)| *b * p).collect();
            let v2 = messages[i-1].2.iter().zip(TauPowers::new(tau)).map(|(b, p)| *b * p).collect();

            messages.push((
                rp,
                v1,
                v2
            ));
        }
    }

    // Check validity
    for i in 0..parties {
        if i == 0 {
            assert!(checkseq(messages[i].1.iter(), &messages[i].0));
            assert!(checkseq(messages[i].2.iter(), &Spair::new(&messages[i].1[0], &messages[i].1[1])));
        } else {
            assert!(checkseq(messages[i].1.iter(), &Spair::new(&messages[i].2[0], &messages[i].2[1])));
            assert!(checkseq(messages[i].2.iter(), &Spair::new(&messages[i].1[0], &messages[i].1[1])));
            assert!(same_power(&Spair::new(&messages[i-1].1[1], &messages[i].1[1]), &messages[i].0));
        }
    }
}

#[test]
fn samepower_seq() {
    initialize();

    fn general_seq_test<Group1: Group, Group2: Group>()
    where Group1: Pairing<Group2>
    {
        // Test working
        {
            let s = Fr::random();
            let p = Spair::<Group2>::random(&s);

            let mut a = vec![];
            a.push(Group1::random());

            for _ in 0..50 {
                let n = *a.last().unwrap() * s;
                a.push(n);
            }

            assert!(checkseq(a.iter(), &p));
        }

        // Test not working.
        {
            let s = Fr::random();
            let p = Spair::<Group2>::random(&s);

            let mut a = vec![];
            a.push(Group1::random());

            for i in 0..50 {
                if i == 10 {
                    a.push(Group1::random());
                } else {
                    let n = *a.last().unwrap() * s;
                    a.push(n);
                }
            }

            assert!(!checkseq(a.iter(), &p));
        }
    }

    general_seq_test::<G1, G2>();
    general_seq_test::<G2, G1>();
}

#[test]
fn samepower_vec() {
    initialize();

    fn samepower_general_test<Group1: Group, Group2: Group>(i: usize, f: usize)
    where Group1: Pairing<Group2>
    {
        // Test working
        {
            let s = Fr::random();
            let p = Spair::<Group2>::random(&s);

            let a: Vec<Spair<Group1>> = (0..i).map(|_| Spair::random(&s)).collect();

            assert!(checkvec(&a, &p));
        }

        // Test different scalar
        {
            let s = Fr::random();
            let p = Spair::<Group2>::random(&Fr::random());

            let a: Vec<Spair<Group1>> = (0..i).map(|_| Spair::random(&s)).collect();

            assert!(!checkvec(&a, &p));
        }

        // Test incorrect spair
        {
            let s = Fr::random();
            let p = Spair::<Group2>::random(&s);

            let a: Vec<Spair<Group1>> = (0..i).map(|i| {
                if i == f {
                    Spair::random(&Fr::random())
                } else {
                    Spair::random(&s)
                }
            }).collect();

            assert!(!checkvec(&a, &p));
        }
    }

    fn samepower_test_each_group(i: usize, f: usize)
    {
        samepower_general_test::<G1, G2>(i, f);
        samepower_general_test::<G2, G1>(i, f);
    }

    samepower_test_each_group(1, 0);
    samepower_test_each_group(10, 5);
    samepower_test_each_group(100, 50);
}

#[test]
fn trivial_samepower() {
    initialize();

    let f = Fr::random();
    let a = Spair::<G1>::random(&f);
    let b = Spair::<G2>::random(&f);
    let c = Spair::<G1>::random(&Fr::random());

    assert!(same_power(&a, &b));
    assert!(same_power(&b, &a));
    assert!(!same_power(&b, &c));
}
