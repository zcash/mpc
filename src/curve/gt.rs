#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Gt {
    a: [u64; 4 * 6],
    b: [u64; 4 * 6]
}
