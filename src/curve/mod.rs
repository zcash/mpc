use std::ops::{Add,Sub,Mul,Neg};
use std::sync::Mutex;

mod fr;
mod g1;
mod g2;
mod gt;

pub use self::fr::Fr;
pub use self::gt::Gt;

pub type G1 = G<g1::G1>;
pub type G2 = G<g2::G2>;

extern "C" {
    fn bnwrap_init();
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

pub trait GroupElement: Sized + Copy + Clone {
    fn zero() -> Self;
    fn one() -> Self;
    fn random() -> Self;

    fn is_equal(&self, other: &Self) -> bool;
    fn is_zero(&self) -> bool;

    fn arith_neg(&self) -> Self;
    fn arith_add(&self, other: &Self) -> Self;
    fn arith_sub(&self, other: &Self) -> Self;
    fn arith_mul(&self, other: &Fr) -> Self;
}

#[derive(Copy, Clone)]
pub struct G<T: GroupElement>(T);

impl<T: GroupElement> G<T> {
    pub fn zero() -> Self {
        G(T::zero())
    }

    pub fn one() -> Self {
        G(T::one())
    }

    pub fn random() -> Self {
        G(T::random())
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T: GroupElement> PartialEq for G<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.is_equal(&other.0)
    }
}

impl<T: GroupElement> Neg for G<T> {
    type Output = G<T>;

    fn neg(self) -> G<T> {
        G(self.0.arith_neg())
    }
}

impl<T: GroupElement> Add for G<T> {
    type Output = G<T>;

    fn add(self, other: G<T>) -> G<T> {
        G(self.0.arith_add(&other.0))
    }
}

impl<T: GroupElement> Sub for G<T> {
    type Output = G<T>;

    fn sub(self, other: G<T>) -> G<T> {
        G(self.0.arith_sub(&other.0))
    }
}

impl<T: GroupElement> Mul<Fr> for G<T> {
    type Output = G<T>;

    fn mul(self, other: Fr) -> G<T> {
        G(self.0.arith_mul(&other))
    }
}

mod test_groups {
    use super::{G, Fr, g1, g2, initialize, GroupElement};

    fn test_allocations_and_moves<Group: GroupElement>() {
        let a: Vec<G<Group>> = (0..100)
                               .map(|i| (G::one() * Fr::from_str(&format!("{}", i))))
                               .collect();

        let b = a.into_iter().fold(G::zero(), |a, b| a + b);

        assert!(b == G::one() * Fr::from_str("4950"));
    }

    fn test_associative<Group: GroupElement>() {
        for _ in 0..50 {
            let a = G::<Group>::random();
            let b = G::<Group>::random();
            let c = G::<Group>::random();

            let x = (a + b) + c;
            let y = (a + c) + b;

            assert!(x == y);
        }
    }

    fn test_scalar_mul<Group: GroupElement>() {
        let r = G::<Group>::random();
        let res = r * Fr::from_str("16");

        let mut acc = G::<Group>::zero();

        for _ in 0..16 {
            acc = acc + r;
        }

        assert!(acc == res);
    }

    fn test_addition<Group: GroupElement>() {
        {
            let a = G::<Group>::random();
            let b = -(a);
            let c = a + b;

            assert!(c.is_zero());
        }
        {
            let a = G::<Group>::random();
            let b = -(a);
            let c = a - b;
            let d = a * Fr::from_str("2");

            assert!(c == d);
        }
    }

    fn test_primitives<Group: GroupElement>() {
        let a = G::<Group>::zero();
        let b = G::<Group>::one();

        assert_eq!(a.is_zero(), true);
        assert_eq!(b.is_zero(), false);
    }

    fn test_group_ops<Group: GroupElement>() {
        test_associative::<Group>();
        test_primitives::<Group>();
        test_scalar_mul::<Group>();
        test_addition::<Group>();
        test_allocations_and_moves::<Group>();
    }

    #[test]
    fn test_g1() {
        initialize();

        test_group_ops::<g1::G1>();
    }

    #[test]
    fn test_g2() {
        initialize();

        test_group_ops::<g2::G2>();
    }
}
