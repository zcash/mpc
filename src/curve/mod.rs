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
    fn bnwrap_init();
    fn bnwrap_pairing(p: *const G1, q: *const G2) -> Gt;
}

lazy_static! {
    static ref INIT_LOCK: Mutex<bool> = Mutex::new(false);
}

/// This must be called before anything in this module is used.
pub fn initialize() {
    let mut l = INIT_LOCK.lock().unwrap();

    if !*l {
        unsafe { bnwrap_init(); }
        *l = true;
    }
}

pub fn pairing(p: &G1, q: &G2) -> Gt {
    unsafe { bnwrap_pairing(p, q) }
}

pub trait Group: Sized +
                        Copy +
                        Clone +
                        Mul<Fr, Output=Self> +
                        Add<Output=Self> +
                        Sub<Output=Self> +
                        Neg<Output=Self> +
                        PartialEq {
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
