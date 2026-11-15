//! Thin benchmark-only mappings to the two public request APIs.
use serde::{Deserialize, Serialize};

pub const METHOD: u64 = 7;
pub const BYTES: usize = 64 * 1024 * 1024;
pub const ALLOCATION: usize = 512 * 1024 * 1024;
pub const NODES: usize = 32 * 1024 * 1024;
pub const DEPTH: usize = 64;

pub trait Supplier {
    const NAME: &'static str;
    fn len<T: Serialize>(value: &T) -> usize;
    fn encode<T: Serialize>(value: &T) -> Vec<u8>;
    fn reuse<T: Serialize>(value: &T, output: &mut Vec<u8>);
    fn slice<T: Serialize>(value: &T, output: &mut [u8]) -> usize;
    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, ()>;
}

pub struct Candidate;
pub struct Reference;
const NEW_LIMITS: candidate::Limits = candidate::Limits {
    bytes: BYTES,
    allocation: ALLOCATION,
    nodes: NODES,
    depth: DEPTH,
};
const OLD_LIMITS: reference::Limits = reference::Limits {
    bytes: BYTES,
    allocation: ALLOCATION,
    nodes: NODES,
    depth: DEPTH,
};
const ENCODING: candidate::Encoding = candidate::Encoding::Frame {
    method: METHOD,
    direction: candidate::Direction::Request,
};

impl Supplier for Candidate {
    const NAME: &'static str = "candidate";
    fn len<T: Serialize>(value: &T) -> usize {
        candidate::encoded_len(value, ENCODING, NEW_LIMITS).unwrap()
    }
    fn encode<T: Serialize>(value: &T) -> Vec<u8> {
        candidate::encode(value, ENCODING, NEW_LIMITS).unwrap()
    }
    fn reuse<T: Serialize>(value: &T, output: &mut Vec<u8>) {
        candidate::encode_into(value, ENCODING, NEW_LIMITS, output).unwrap();
    }
    fn slice<T: Serialize>(value: &T, output: &mut [u8]) -> usize {
        candidate::encode_to_slice(value, ENCODING, NEW_LIMITS, output)
            .unwrap()
            .len()
    }
    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, ()> {
        candidate::decode(bytes, ENCODING, NEW_LIMITS).map_err(|_| ())
    }
}
impl Supplier for Reference {
    const NAME: &'static str = "reference";
    fn len<T: Serialize>(value: &T) -> usize {
        reference::native::encoded_frame_len(value, OLD_LIMITS).unwrap()
    }
    fn encode<T: Serialize>(value: &T) -> Vec<u8> {
        reference::native::encode_request(METHOD, value, OLD_LIMITS).unwrap()
    }
    fn reuse<T: Serialize>(value: &T, output: &mut Vec<u8>) {
        reference::native::encode_request_into(METHOD, value, OLD_LIMITS, output).unwrap();
    }
    fn slice<T: Serialize>(value: &T, output: &mut [u8]) -> usize {
        reference::native::encode_request_to_slice(METHOD, value, OLD_LIMITS, output)
            .unwrap()
            .len()
    }
    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, ()> {
        reference::native::decode_request(METHOD, bytes, OLD_LIMITS).map_err(|_| ())
    }
}
