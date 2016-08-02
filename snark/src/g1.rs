use std::ops::{Add,Sub,Mul,Neg};
use super::{Fr,Group};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct G1 {
    x: [u64; 4],
    y: [u64; 4],
    z: [u64; 4]
}

extern "C" {
    fn libsnarkwrap_G1_zero() -> G1;
    fn libsnarkwrap_G1_one() -> G1;
    fn libsnarkwrap_G1_random() -> G1;

    fn libsnarkwrap_G1_is_zero(p: *const G1) -> bool;
    fn libsnarkwrap_G1_is_equal(p: *const G1, q: *const G1) -> bool;

    fn libsnarkwrap_G1_add(p: *const G1, q: *const G1) -> G1;
    fn libsnarkwrap_G1_sub(p: *const G1, q: *const G1) -> G1;
    fn libsnarkwrap_G1_neg(p: *const G1) -> G1;
    fn libsnarkwrap_G1_scalarmul(p: *const G1, s: *const Fr) -> G1;
}

impl PartialEq for G1 {
    fn eq(&self, other: &G1) -> bool {
        unsafe { libsnarkwrap_G1_is_equal(self, other) }
    }
}

impl Group for G1 {
    fn zero() -> G1 {
        unsafe { libsnarkwrap_G1_zero() }
    }

    fn one() -> G1 {
        unsafe { libsnarkwrap_G1_one() }
    }

    fn random() -> G1 {
        unsafe { libsnarkwrap_G1_random() }
    }

    fn is_zero(&self) -> bool {
        unsafe { libsnarkwrap_G1_is_zero(self) }
    }
}

impl Add for G1 {
    type Output = G1;

    fn add(self, other: G1) -> G1 {
        unsafe { libsnarkwrap_G1_add(&self, &other) }
    }
}

impl Mul<Fr> for G1 {
    type Output = G1;

    fn mul(self, other: Fr) -> G1 {
        unsafe { libsnarkwrap_G1_scalarmul(&self, &other) }
    }
}

impl Sub for G1 {
    type Output = G1;

    fn sub(self, other: G1) -> G1 {
        unsafe { libsnarkwrap_G1_sub(&self, &other) }
    }
}

impl Neg for G1 {
    type Output = G1;

    fn neg(self) -> G1 {
        unsafe { libsnarkwrap_G1_neg(&self) }
    }
}
