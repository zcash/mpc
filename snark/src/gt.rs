use std::ops::Mul;
use super::Fr;

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Gt {
    a: [u64; 4 * 6],
    b: [u64; 4 * 6]
}

extern "C" {
    fn bnwrap_gt_exp(p: *const Gt, s: *const Fr) -> Gt;
}

impl Mul<Fr> for Gt {
    type Output = Gt;

    fn mul(self, other: Fr) -> Gt {
        unsafe { bnwrap_gt_exp(&self, &other) }
    }
}
