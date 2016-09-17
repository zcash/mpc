use rand::Rng;
use bn::*;
use super::multicore::*;
use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

#[derive(Clone, PartialEq, Eq)]
pub struct Spair<G: Group> {
    f: G,
    fs: G
}

impl<G: Group> Encodable for Spair<G> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        try!(self.f.encode(s));
        try!(self.fs.encode(s));

        Ok(())
    }
}

impl<G: Group> Decodable for Spair<G> {
    fn decode<S: Decoder>(s: &mut S) -> Result<Spair<G>, S::Error> {
        let f = try!(G::decode(s));
        let fs = try!(G::decode(s));

        Spair::new(f, fs).ok_or_else(|| s.error("invalid s-pair"))
    }
}

impl<G: Group> Spair<G> {
    pub fn new(f: G, fs: G) -> Option<Self> {
        if f.is_zero() || fs.is_zero() {
            None
        } else {
            Some(Spair {
                f: f,
                fs: fs
            })
        }
    }

    pub fn random<R: Rng>(rng: &mut R, s: Fr) -> Option<Self> {
        let f = G::random(rng);

        Spair::new(f, f * s)
    }
}


pub trait Pairing<G: Group>: Group {
    fn pairing(self, other: G) -> Gt;
}

impl Pairing<G2> for G1 {
    fn pairing(self, other: G2) -> Gt {
        pairing(self, other)
    }
}

impl Pairing<G1> for G2 {
    fn pairing(self, other: G1) -> Gt {
        pairing(other, self)
    }
}

pub fn same_power<Group1: Group, Group2: Group>(a: &Spair<Group1>, b: &Spair<Group2>) -> bool
where Group1: Pairing<Group2> {
    a.f.pairing(b.fs) == a.fs.pairing(b.f)
}

pub fn checkvec<Group1: Group, Group2: Group>(
    v1: &[Group1], v2: &[Group1], a: &Spair<Group2>
) -> bool
where Group1: Pairing<Group2>
{
    parallel_all(v1, v2, |v1, v2| {
        let rng = &mut ::rand::thread_rng();
        let mut p = Group1::zero();
        let mut q = Group1::zero();

        for (a, b) in v1.iter().zip(v2.iter()) {
            let alpha = Fr::random(rng);
            p = p + (*a * alpha);
            q = q + (*b * alpha);
        }

        if p.is_zero() || q.is_zero() {
            false
        } else {
            same_power(&Spair::new(p, q).unwrap(), a)
        }
    }, ::THREADS)
}

pub fn checkseq<Group1: Group, Group2: Group>(
    v: &[Group1], a: &Spair<Group2>
) -> bool
where Group1: Pairing<Group2>
{
    checkvec(&v[0..v.len()-1], &v[1..], a)
}

#[test]
fn trivial_samepower() {
    let rng = &mut ::rand::thread_rng();

    let f = Fr::random(rng);
    let e = Fr::random(rng);
    let a = Spair::<G1>::random(rng, f).unwrap();
    let b = Spair::<G2>::random(rng, f).unwrap();
    let c = Spair::<G1>::random(rng, e).unwrap();

    assert!(same_power(&a, &b));
    assert!(same_power(&b, &a));
    assert!(!same_power(&b, &c));
}

#[test]
fn samepower_seq() {
    fn general_seq_test<Group1: Group, Group2: Group>()
    where Group1: Pairing<Group2>
    {
        let rng = &mut ::rand::thread_rng();

        // Test working
        {
            let s = Fr::random(rng);
            let p = Spair::<Group2>::random(rng, s).unwrap();

            let mut a = vec![];
            a.push(Group1::random(rng));

            for _ in 0..50 {
                let n = *a.last().unwrap() * s;
                a.push(n);
            }

            assert!(checkseq(&a, &p));
        }

        // Test not working.
        {
            let s = Fr::random(rng);
            let p = Spair::<Group2>::random(rng, s).unwrap();

            let mut a = vec![];
            a.push(Group1::random(rng));

            for i in 0..50 {
                if i == 10 {
                    a.push(Group1::random(rng));
                } else {
                    let n = *a.last().unwrap() * s;
                    a.push(n);
                }
            }

            assert!(!checkseq(&a, &p));
        }
    }

    general_seq_test::<G1, G2>();
    general_seq_test::<G2, G1>();
}
