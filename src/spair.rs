use snark::*;
use sequences::*;

#[derive(Clone)]
pub struct Spair<G: Group> {
    p: G,
    q: G
}

impl<G: Group> Spair<G> {
    pub fn random(s: &Fr) -> Self {
        let p = G::random_nonzero();

        Spair {
            p: p,
            q: p * (*s)
        }
    }

    pub fn new(p: &G, q: &G) -> Option<Self> {
        if p.is_zero() || q.is_zero() {
            None
        } else {
            Some(Spair {
                p: *p,
                q: *q
            })
        }
    }
}

pub fn same_power<Group1: Group, Group2: Group>(a: &Spair<Group1>, b: &Spair<Group2>) -> bool
where Group1: Pairing<Group2> {
    pairing(&a.p, &b.q) == pairing(&a.q, &b.p)
}

/// This performs a check to see if a large number of (p,q) pairs in G
/// have the same power, with only one pairing.
pub fn check<'a,
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

    if p.is_zero() || q.is_zero() { return false; }

    same_power(&Spair::new(&p, &q).unwrap(), &a)
}

pub fn checkseq<'a,
            Group1: Group,
            Group2: Group,
            I: Iterator<Item=&'a Group1>>
            (i: I, a: &Spair<Group2>) -> bool
where Group1: Pairing<Group2>
{
    check(Sequences::new(i), a)
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
