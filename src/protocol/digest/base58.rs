// Rust Bitcoin Library
// Written in 2014 by
//   Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Base58 encoder and decoder

use std::{error, fmt};

use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

// Unlike base58check, we use blake2s for the checksum,
// because we didn't want to pull in yet another hash
// algorithm. We don't care about being well-specified
// in our use of base58 here.
fn blake2_u32(input: &[u8]) -> u32 {
    use blake2_rfc::blake2s::blake2s;

    let mut buf = [0; 32];
    buf.copy_from_slice(&blake2s(32, &[], input).as_bytes());

    LittleEndian::read_u32(&buf[28..])
}

/// An error that might occur during base58 decoding
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    /// Invalid character encountered
    BadByte(u8),
    /// Checksum was not correct (expected, actual)
    BadChecksum(u32, u32),
    /// The length (in bytes) of the object was not correct
    InvalidLength(usize),
    /// Version byte(s) were not recognized
    InvalidVersion(Vec<u8>),
    /// Checked data was less than 4 bytes
    TooShort(usize),
    /// Any other error
    Other(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BadByte(b) => write!(f, "invalid base58 character 0x{:x}", b),
            Error::BadChecksum(exp, actual) => write!(f, "base58ck checksum 0x{:x} does not match expected 0x{:x}", actual, exp),
            Error::InvalidLength(ell) => write!(f, "length {} invalid for this base58 type", ell),
            Error::InvalidVersion(ref v) => write!(f, "version {:?} invalid for this base58 type", v),
            Error::TooShort(_) => write!(f, "base58ck data not even long enough for a checksum"),
            Error::Other(ref s) => f.write_str(s)
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> { None }
    fn description(&self) -> &'static str {
        match *self {
            Error::BadByte(_) => "invalid b58 character",
            Error::BadChecksum(_, _) => "invalid b58ck checksum",
            Error::InvalidLength(_) => "invalid length for b58 type",
            Error::InvalidVersion(_) => "invalid version for b58 type",
            Error::TooShort(_) => "b58ck data less than 4 bytes",
            Error::Other(_) => "unknown b58 error"
        }
    }
}

static BASE58_CHARS: &'static [u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

static BASE58_DIGITS: [Option<u8>; 128] = [
    None,     None,     None,     None,     None,     None,     None,     None,     // 0-7
    None,     None,     None,     None,     None,     None,     None,     None,     // 8-15
    None,     None,     None,     None,     None,     None,     None,     None,     // 16-23
    None,     None,     None,     None,     None,     None,     None,     None,     // 24-31
    None,     None,     None,     None,     None,     None,     None,     None,     // 32-39
    None,     None,     None,     None,     None,     None,     None,     None,     // 40-47
    None,     Some(0),  Some(1),  Some(2),  Some(3),  Some(4),  Some(5),  Some(6),  // 48-55
    Some(7),  Some(8),  None,     None,     None,     None,     None,     None,     // 56-63
    None,     Some(9),  Some(10), Some(11), Some(12), Some(13), Some(14), Some(15), // 64-71
    Some(16), None,     Some(17), Some(18), Some(19), Some(20), Some(21), None,     // 72-79
    Some(22), Some(23), Some(24), Some(25), Some(26), Some(27), Some(28), Some(29), // 80-87
    Some(30), Some(31), Some(32), None,     None,     None,     None,     None,     // 88-95
    None,     Some(33), Some(34), Some(35), Some(36), Some(37), Some(38), Some(39), // 96-103
    Some(40), Some(41), Some(42), Some(43), None,     Some(44), Some(45), Some(46), // 104-111
    Some(47), Some(48), Some(49), Some(50), Some(51), Some(52), Some(53), Some(54), // 112-119
    Some(55), Some(56), Some(57), None,     None,     None,     None,     None,     // 120-127
];

/// Trait for objects which can be read as base58
pub trait FromBase58: Sized {
    /// Constructs an object from the byte-encoding (base 256)
    /// representation of its base58 format
    fn from_base58_layout(data: Vec<u8>) -> Result<Self, Error>;

    /// Obtain an object from its base58 encoding
    fn from_base58(data: &str) -> Result<Self, Error> {
        // 11/15 is just over log_256(58)
        let mut scratch = vec![0u8; 1 + data.len() * 11 / 15];
        // Build in base 256
        for d58 in data.bytes() {
            // Compute "X = X * 58 + next_digit" in base 256
            if d58 as usize > BASE58_DIGITS.len() {
                return Err(Error::BadByte(d58));
            }
            let mut carry = match BASE58_DIGITS[d58 as usize] {
                Some(d58) => d58 as u32,
                None => { return Err(Error::BadByte(d58)); }
            };
            for d256 in scratch.iter_mut().rev() {
                carry += *d256 as u32 * 58;
                *d256 = carry as u8;
                carry /= 256;
            }
            assert_eq!(carry, 0);
        }

        // Copy leading zeroes directly
        let mut ret: Vec<u8> = data.bytes().take_while(|&x| x == BASE58_CHARS[0])
                                           .map(|_| 0)
                                           .collect();
        // Copy rest of string
        ret.extend(scratch.into_iter().skip_while(|&x| x == 0));
        FromBase58::from_base58_layout(ret)
    }

    /// Obtain an object from its base58check encoding
    fn from_base58check(data: &str) -> Result<Self, Error> {
        let mut ret: Vec<u8> = try!(FromBase58::from_base58(data));
        if ret.len() < 4 {
            return Err(Error::TooShort(ret.len()));
        }
        let ck_start = ret.len() - 4;
        let expected = blake2_u32(&ret[..ck_start]);
        let actual = LittleEndian::read_u32(&ret[ck_start..(ck_start + 4)]);
        if expected != actual {
            return Err(Error::BadChecksum(expected, actual));
        }
  
          ret.truncate(ck_start);
        FromBase58::from_base58_layout(ret)
    }
}

/// Directly encode a slice as base58
pub fn base58_encode_slice(data: &[u8]) -> String {
    // 7/5 is just over log_58(256)
    let mut scratch = vec![0u8; 1 + data.len() * 7 / 5];
    // Build in base 58
    for &d256 in &data.base58_layout() {
        // Compute "X = X * 256 + next_digit" in base 58
        let mut carry = d256 as u32;
        for d58 in scratch.iter_mut().rev() {
            carry += (*d58 as u32) << 8;
            *d58 = (carry % 58) as u8;
            carry /= 58;
        }
        assert_eq!(carry, 0);
    }

    // Copy leading zeroes directly
    let mut ret: Vec<u8> = data.iter().take_while(|&&x| x == 0)
                                      .map(|_| BASE58_CHARS[0])
                                      .collect();
    // Copy rest of string
    ret.extend(scratch.into_iter().skip_while(|&x| x == 0)
                                  .map(|x| BASE58_CHARS[x as usize]));
    String::from_utf8(ret).unwrap()
}

/// Trait for objects which can be written as base58
pub trait ToBase58 {
    /// The serialization to be converted into base58
    fn base58_layout(&self) -> Vec<u8>;

    /// Obtain a string with the base58 encoding of the object
    fn to_base58(&self) -> String {
        base58_encode_slice(&self.base58_layout()[..])
    }

    /// Obtain a string with the base58check encoding of the object
    /// (Tack the first 4 256-digits of the object's Bitcoin hash onto the end.)
    fn to_base58check(&self) -> String {
        let mut data = self.base58_layout();
        let checksum = blake2_u32(&data);
        data.write_u32::<LittleEndian>(checksum).unwrap();
        base58_encode_slice(&data)
    }
}

// Trivial implementations for slices and vectors
impl<'a> ToBase58 for &'a [u8] {
    fn base58_layout(&self) -> Vec<u8> { self.to_vec() }
    fn to_base58(&self) -> String { base58_encode_slice(*self) }
}

impl<'a> ToBase58 for Vec<u8> {
    fn base58_layout(&self) -> Vec<u8> { self.clone() }
    fn to_base58(&self) -> String { base58_encode_slice(&self[..]) }
}

impl FromBase58 for Vec<u8> {
    fn from_base58_layout(data: Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(data)
    }
}
