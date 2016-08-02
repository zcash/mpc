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

impl GroupElement for G2 {
    fn zero() -> G2 {
        unsafe { bnwrap_G2_zero() }
    }

    fn one() -> G2 {
        unsafe { bnwrap_G2_one() }
    }

    fn is_equal(&self, other: &Self) -> bool {
        unsafe { bnwrap_G2_is_equal(self, other) }
    }

    fn random() -> G2 {
        unsafe { bnwrap_G2_random() }
    }

    fn is_zero(&self) -> bool {
        unsafe { bnwrap_G2_is_zero(self) }
    }

    fn arith_neg(&self) -> Self {
        unsafe { bnwrap_G2_neg(self) }
    }

    fn arith_add(&self, other: &Self) -> Self {
        unsafe { bnwrap_G2_add(self, other) }
    }

    fn arith_sub(&self, other: &Self) -> Self {
        unsafe { bnwrap_G2_sub(self, other) }
    }

    fn arith_mul(&self, other: &Fr) -> Self {
        unsafe { bnwrap_G2_scalarmul(self, other) }
    }
}
