use bn::Fr;

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};
use bincode::SizeLimit::Infinite;
use bincode::rustc_serialize::encode;
use blake2_rfc::blake2b::blake2b;

/// 512-bit BLAKE2b hash digest
pub struct Digest([u8; 64]);

impl Digest {
    pub fn to_string(&self) -> String {
        use rustc_serialize::hex::{ToHex};

        self.0.to_hex()
    }

    pub fn from_string(s: &str) -> Option<Digest> {
        use rustc_serialize::hex::{FromHex};

        match s.from_hex() {
            Ok(decoded) => {
                if decoded.len() == 64 {
                    let mut decoded_bytes: [u8; 64] = [0; 64];
                    decoded_bytes.copy_from_slice(&decoded);
                    Some(Digest(decoded_bytes))
                } else {
                    None
                }
            },
            Err(_) => {
                None
            }
        }
    }

    pub fn from<E: Encodable>(obj: &E) -> Option<Self> {
        let serialized = encode(obj, Infinite);
        match serialized {
            Ok(ref serialized) => {
                let mut buf: [u8; 64] = [0; 64];
                buf.copy_from_slice(&blake2b(64, &[], serialized).as_bytes());

                Some(Digest(buf))
            },
            Err(_) => None
        }
    }

    pub fn interpret(&self) -> Fr {
        Fr::interpret(&self.0)
    }
}

impl PartialEq for Digest {
    fn eq(&self, other: &Digest) -> bool {
        (&self.0[..]).eq(&other.0[..])
    }
}

impl Eq for Digest { }

impl Copy for Digest { }
impl Clone for Digest {
    fn clone(&self) -> Digest {
        *self
    }
}

impl Encodable for Digest {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        for i in 0..64 {
            try!(s.emit_u8(self.0[i]));
        }

        Ok(())
    }
}

impl Decodable for Digest {
    fn decode<S: Decoder>(s: &mut S) -> Result<Digest, S::Error> {
        let mut buf = [0; 64];

        for i in 0..64 {
            buf[i] = try!(s.read_u8());
        }

        Ok(Digest(buf))
    }
}
