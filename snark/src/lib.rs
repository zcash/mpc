extern crate libc;
#[macro_use]
extern crate lazy_static;

mod curve;

pub use self::curve::{G1, G2, Gt, Fr, pairing, initialize};
