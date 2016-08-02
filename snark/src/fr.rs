use std::ops::{Add,Sub,Mul,Neg};
use libc::c_char;
use std::ffi::CString;

/// The scalar field for the curve construction we use.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Fr([u64; 4]);

extern "C" {
    fn libsnarkwrap_Fr_random() -> Fr;
    fn libsnarkwrap_Fr_from(s: *const c_char) -> Fr;
    fn libsnarkwrap_Fr_add(a: *const Fr, b: *const Fr) -> Fr;
    fn libsnarkwrap_Fr_mul(a: *const Fr, b: *const Fr) -> Fr;
    fn libsnarkwrap_Fr_sub(a: *const Fr, b: *const Fr) -> Fr;
    fn libsnarkwrap_Fr_neg(a: *const Fr) -> Fr;
}

impl Fr {
    pub fn random() -> Self {
        unsafe { libsnarkwrap_Fr_random() }
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
