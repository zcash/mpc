use bn::Fr;

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::encode;
use blake2_rfc::blake2b::blake2b;
use blake2_rfc::blake2s::blake2s;

macro_rules! digest_impl {
    ($name:ident, $bytes:expr, $hash:ident) => {
        pub struct $name([u8; $bytes]);

        impl $name {
            pub fn from<E: Encodable>(obj: &E) -> Option<Self> {
                let serialized = encode(obj, Infinite);
                match serialized {
                    Ok(ref serialized) => {
                        let mut buf: [u8; $bytes] = [0; $bytes];
                        buf.copy_from_slice(&$hash($bytes, &[], serialized).as_bytes());

                        Some($name(buf))
                    },
                    Err(_) => None
                }
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &$name) -> bool {
                (&self.0[..]).eq(&other.0[..])
            }
        }

        impl Eq for $name { }

        impl Copy for $name { }
        impl Clone for $name {
            fn clone(&self) -> $name {
                *self
            }
        }

        impl Encodable for $name {
            fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
                for i in 0..$bytes {
                    try!(s.emit_u8(self.0[i]));
                }

                Ok(())
            }
        }

        impl Decodable for $name {
            fn decode<S: Decoder>(s: &mut S) -> Result<$name, S::Error> {
                let mut buf = [0; $bytes];

                for i in 0..$bytes {
                    buf[i] = try!(s.read_u8());
                }

                Ok($name(buf))
            }
        }
    }
}

digest_impl!(Digest512, 64, blake2b);
digest_impl!(Digest256, 32, blake2s);

impl Digest512 {
    pub fn interpret(&self) -> Fr {
        Fr::interpret(&self.0)
    }
}

impl Digest256 {
    pub fn to_string(&self) -> String {
        use rustc_serialize::hex::{ToHex};

        self.0.to_hex()
    }

    pub fn from_string(s: &str) -> Option<Digest256> {
        use rustc_serialize::hex::{FromHex};

        match s.from_hex() {
            Ok(decoded) => {
                if decoded.len() == 32 {
                    let mut decoded_bytes: [u8; 32] = [0; 32];
                    decoded_bytes.copy_from_slice(&decoded);
                    Some(Digest256(decoded_bytes))
                } else {
                    None
                }
            },
            Err(_) => {
                None
            }
        }
    }
}
