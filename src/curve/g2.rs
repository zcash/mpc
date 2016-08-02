use std::ops::{Add,Sub,Mul,Neg};
use super::{Fr,GroupElement};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct G2 {
    x: [u64; 4 * 2],
    y: [u64; 4 * 2],
    z: [u64; 4 * 2]
}

extern "C" {
    fn bnwrap_G2_zero() -> G2;
    fn bnwrap_G2_one() -> G2;
    fn bnwrap_G2_random() -> G2;

    fn bnwrap_G2_is_zero(p: *const G2) -> bool;
    fn bnwrap_G2_is_equal(p: *const G2, q: *const G2) -> bool;

    fn bnwrap_G2_add(p: *const G2, q: *const G2) -> G2;
    fn bnwrap_G2_sub(p: *const G2, q: *const G2) -> G2;
    fn bnwrap_G2_neg(p: *const G2) -> G2;
    fn bnwrap_G2_scalarmul(p: *const G2, s: *const Fr) -> G2;
}

impl PartialEq for G2 {
    fn eq(&self, other: &G2) -> bool {
        unsafe { bnwrap_G2_is_equal(self, other) }
    }
}

impl GroupElement for G2 {
    fn zero() -> G2 {
        unsafe { bnwrap_G2_zero() }
    }

    fn one() -> G2 {
        unsafe { bnwrap_G2_one() }
    }

    fn random() -> G2 {
        unsafe { bnwrap_G2_random() }
    }

    fn is_zero(&self) -> bool {
        unsafe { bnwrap_G2_is_zero(self) }
    }
}

impl Add for G2 {
    type Output = G2;

    fn add(self, other: G2) -> G2 {
        unsafe { bnwrap_G2_add(&self, &other) }
    }
}

impl Mul<Fr> for G2 {
    type Output = G2;

    fn mul(self, other: Fr) -> G2 {
        unsafe { bnwrap_G2_scalarmul(&self, &other) }
    }
}

impl Sub for G2 {
    type Output = G2;

    fn sub(self, other: G2) -> G2 {
        unsafe { bnwrap_G2_sub(&self, &other) }
    }
}

impl Neg for G2 {
    type Output = G2;

    fn neg(self) -> G2 {
        unsafe { bnwrap_G2_neg(&self) }
    }
}
