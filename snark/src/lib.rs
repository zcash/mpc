extern crate libc;
#[macro_use]
extern crate lazy_static;

use std::ops::{Add,Sub,Mul,Neg};
use std::sync::Mutex;

mod fr;
mod g1;
mod g2;
mod gt;

pub use self::fr::Fr;
pub use self::gt::Gt;
pub use self::g1::G1;
pub use self::g2::G2;

extern "C" {
    fn libsnarkwrap_init();
    fn libsnarkwrap_pairing(p: *const G1, q: *const G2) -> Gt;
    fn libsnarkwrap_getcs(d: *mut libc::uint32_t, omega: *mut Fr) -> *mut libc::c_void;
    fn libsnarkwrap_dropcs(cs: *mut libc::c_void);
    fn libsnarkwrap_test_compare_tau(
        i: *const G1,
        tau: *const Fr,
        d: libc::uint32_t,
        qap: *const libc::c_void) -> bool;
}

lazy_static! {
    static ref INIT_LOCK: Mutex<bool> = Mutex::new(false);
}

/// This must be called before anything in this module is used.
pub fn initialize() {
    let mut l = INIT_LOCK.lock().unwrap();

    if !*l {
        unsafe { libsnarkwrap_init(); }
        *l = true;
    }
}

pub struct CS(*mut libc::c_void);

impl Drop for CS {
    fn drop(&mut self) {
        unsafe { libsnarkwrap_dropcs(self.0) }
    }
}

/// Get the QAP info for the generation routines
pub fn getqap() -> (usize, Fr, CS) {
    let mut d = 0;
    let mut o = Fr::zero();

    let cs = unsafe { libsnarkwrap_getcs(&mut d, &mut o) };

    (d as usize, o, CS(cs))
}

/// Check that the lagrange coefficients computed by tau over
/// G1 equal the expected vector.
pub fn compare_tau(v: &[G1], tau: &Fr, cs: &CS) -> bool {
    unsafe { libsnarkwrap_test_compare_tau(&v[0], tau, v.len() as u32, cs.0) }
}

pub trait Pairing<Other: Group> {
    fn g1<'a>(&'a self, other: &'a Other) -> &'a G1;
    fn g2<'a>(&'a self, other: &'a Other) -> &'a G2;
}

impl Pairing<G2> for G1 {
    fn g1<'a>(&'a self, _: &'a G2) -> &'a G1 {
        self
    }
    fn g2<'a>(&'a self, other: &'a G2) -> &'a G2 {
        other
    }
}

impl Pairing<G1> for G2 {
    fn g1<'a>(&'a self, other: &'a G1) -> &'a G1 {
        other
    }
    fn g2<'a>(&'a self, _: &'a G1) -> &'a G2 {
        self
    }
}

pub fn pairing<Ga: Group, Gb: Group>(p: &Ga, q: &Gb) -> Gt where Ga: Pairing<Gb> {
    unsafe { libsnarkwrap_pairing(p.g1(q), p.g2(q)) }
}

pub trait Group: Sized +
                        Copy +
                        Clone +
                        Mul<Fr, Output=Self> +
                        Add<Output=Self> +
                        Sub<Output=Self> +
                        Neg<Output=Self> +
                        PartialEq +
                        'static {
    fn zero() -> Self;
    fn one() -> Self;
    fn random() -> Self;
    fn is_zero(&self) -> bool;
}

#[test]
fn pairing_test() {
    initialize();

    for _ in 0..50 {
        let p = G1::random();
        let q = G2::random();
        let s = Fr::random();

        let sp = p * s;
        let sq = q * s;

        let a = pairing(&p, &q) * s;
        let b = pairing(&sp, &q);
        let c = pairing(&p, &sq);

        assert!(a == b);
        assert!(b == c);
    }
}

#[test]
fn pairing_ordering_irrelevant() {
    initialize();

    let p = G1::random();
    let q = G2::random();

    let a = pairing(&p, &q);
    let b = pairing(&q, &p);

    assert!(a == b);
}

#[cfg(test)]
mod test_groups {
    use super::{Fr, G1, G2, initialize, Group};

    fn test_associative<G: Group>() {
        for _ in 0..50 {
            let a = G::random();
            let b = G::random();
            let c = G::random();

            let x = (a + b) + c;
            let y = (a + c) + b;

            assert!(x == y);
        }
    }

    fn test_primitives<G: Group>() {
        let a = G::zero();
        let b = G::one();

        assert_eq!(a.is_zero(), true);
        assert_eq!(b.is_zero(), false);
    }

    fn test_scalar_mul<G: Group>() {
        let r = G::random();
        let res = r * Fr::from_str("16");

        let mut acc = G::zero();

        for _ in 0..16 {
            acc = acc + r;
        }

        assert!(acc == res);
    }

    fn test_addition<G: Group>() {
        {
            let a = G::random();
            let b = -(a);
            let c = a + b;

            assert!(c.is_zero());
        }
        {
            let a = G::random();
            let b = -(a);
            let c = a - b;
            let d = a * Fr::from_str("2");

            assert!(c == d);
        }
    }

    fn test_allocations_and_moves<G: Group>() {
        let a: Vec<G> = (0..100)
                               .map(|i| (G::one() * Fr::from_str(&format!("{}", i))))
                               .collect();

        let b = a.iter().fold(G::zero(), |a, b| a + *b);

        assert!(b == G::one() * Fr::from_str("4950"));
    }

    fn test_group_ops<G: Group>() {
        test_associative::<G>();
        test_primitives::<G>();
        test_scalar_mul::<G>();
        test_addition::<G>();
        test_allocations_and_moves::<G>();
    }

    #[test]
    fn test_g1() {
        initialize();

        test_group_ops::<G1>();
    }

    #[test]
    fn test_g2() {
        initialize();

        test_group_ops::<G2>();
    }
}
