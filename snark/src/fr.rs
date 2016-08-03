use std::ops::{Add,Sub,Mul,Neg};
use libc::c_char;
use std::ffi::CString;

/// The scalar field for the curve construction we use.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Fr([u64; 4]);

extern "C" {
    fn libsnarkwrap_Fr_random() -> Fr;
    fn libsnarkwrap_Fr_zero() -> Fr;
    fn libsnarkwrap_Fr_one() -> Fr;
    fn libsnarkwrap_Fr_from(s: *const c_char) -> Fr;
    fn libsnarkwrap_Fr_exp(a: *const Fr, b: u32) -> Fr;
    fn libsnarkwrap_Fr_add(a: *const Fr, b: *const Fr) -> Fr;
    fn libsnarkwrap_Fr_mul(a: *const Fr, b: *const Fr) -> Fr;
    fn libsnarkwrap_Fr_sub(a: *const Fr, b: *const Fr) -> Fr;
    fn libsnarkwrap_Fr_neg(a: *const Fr) -> Fr;
    fn libsnarkwrap_Fr_is_zero(a: *const Fr) -> bool;
}

impl Fr {
    pub fn zero() -> Self {
        unsafe { libsnarkwrap_Fr_zero() }
    }

    pub fn one() -> Self {
        unsafe { libsnarkwrap_Fr_one() }
    }

    pub fn random() -> Self {
        unsafe { libsnarkwrap_Fr_random() }
    }

    pub fn is_zero(&self) -> bool {
        unsafe { libsnarkwrap_Fr_is_zero(self) }
    }

    pub fn exp(&self, e: u32) -> Self {
        unsafe { libsnarkwrap_Fr_exp(self, e) }
    }

    pub fn random_nonzero() -> Self {
        let mut tmp = Self::random();

        while tmp.is_zero() {
            tmp = Self::random();
        }

        return tmp;
    }

    pub fn from_str(s: &str) -> Self {
        for c in s.chars() {
            if c != '0' &&
               c != '1' &&
               c != '2' &&
               c != '3' &&
               c != '4' &&
               c != '5' &&
               c != '6' &&
               c != '7' &&
               c != '8' &&
               c != '9' {
                panic!("character out of range")
            }
        }

        let s = CString::new(s).unwrap();

        unsafe { libsnarkwrap_Fr_from(s.as_ptr()) }
    }
}

impl Add for Fr {
    type Output = Fr;

    fn add(self, other: Fr) -> Fr {
        unsafe { libsnarkwrap_Fr_add(&self, &other) }
    }
}

impl Mul for Fr {
    type Output = Fr;

    fn mul(self, other: Fr) -> Fr {
        unsafe { libsnarkwrap_Fr_mul(&self, &other) }
    }
}

impl Sub for Fr {
    type Output = Fr;

    fn sub(self, other: Fr) -> Fr {
        unsafe { libsnarkwrap_Fr_sub(&self, &other) }
    }
}

impl Neg for Fr {
    type Output = Fr;

    fn neg(self) -> Fr {
        unsafe { libsnarkwrap_Fr_neg(&self) }
    }
}

#[test]
fn test_basic_arith() {
    super::initialize();

    let a = Fr::from_str("34563126457335463");
    let b = Fr::from_str("23463856875665981");
    let ab = Fr::from_str("810984252370463483215040853984203");
    let aplusb = Fr::from_str("58026983333001444");
    let aminusb = Fr::from_str("11099269581669482");
    let aneg = Fr::from_str("21888242871839275222246405745257275088548364400416034343698169623449351160154");
    let a50 = Fr::from_str("18657215030604597165059661904200246872501020503322948614804364624353607925980");

    assert!(ab == (a * b));
    assert!(aplusb == (a + b));
    assert!(aminusb == (a - b));
    assert!(aneg == (-a));
    assert!(a50 == a.exp(50));
}

#[test]
fn test_primitives() {
    super::initialize();

    let a = Fr::from_str("0");
    assert!(a.is_zero());
    let a = Fr::from_str("1");
    assert!(!a.is_zero());

    let a = Fr::zero();
    assert!(a.is_zero());
    let a = Fr::one();
    assert!(!a.is_zero());
}
