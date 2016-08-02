use super::{Fr,GroupElement};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct G1 {
    x: [u64; 4],
    y: [u64; 4],
    z: [u64; 4]
}

extern "C" {
    fn bnwrap_G1_zero() -> G1;
    fn bnwrap_G1_one() -> G1;
    fn bnwrap_G1_random() -> G1;

    fn bnwrap_G1_is_zero(p: *const G1) -> bool;
    fn bnwrap_G1_is_equal(p: *const G1, q: *const G1) -> bool;

    fn bnwrap_G1_add(p: *const G1, q: *const G1) -> G1;
    fn bnwrap_G1_sub(p: *const G1, q: *const G1) -> G1;
    fn bnwrap_G1_neg(p: *const G1) -> G1;
    fn bnwrap_G1_scalarmul(p: *const G1, s: *const Fr) -> G1;
}

impl GroupElement for G1 {
    fn zero() -> G1 {
        unsafe { bnwrap_G1_zero() }
    }

    fn one() -> G1 {
        unsafe { bnwrap_G1_one() }
    }

    fn is_equal(&self, other: &Self) -> bool {
        unsafe { bnwrap_G1_is_equal(self, other) }
    }

    fn random() -> G1 {
        unsafe { bnwrap_G1_random() }
    }

    fn is_zero(&self) -> bool {
        unsafe { bnwrap_G1_is_zero(self) }
    }

    fn arith_neg(&self) -> Self {
        unsafe { bnwrap_G1_neg(self) }
    }

    fn arith_add(&self, other: &Self) -> Self {
        unsafe { bnwrap_G1_add(self, other) }
    }

    fn arith_sub(&self, other: &Self) -> Self {
        unsafe { bnwrap_G1_sub(self, other) }
    }

    fn arith_mul(&self, other: &Fr) -> Self {
        unsafe { bnwrap_G1_scalarmul(self, other) }
    }
}
