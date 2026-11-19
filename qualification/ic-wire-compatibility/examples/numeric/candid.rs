//! Single-argument Candid integer list probe.
pub(super) type Value = Vec<u32>;

pub(super) fn prepare(words: Vec<u32>) -> Value {
    words
}

pub(super) fn encode(value: &Value) -> Vec<u8> {
    candid::encode_one(value).unwrap()
}

pub(super) fn decode(bytes: &[u8]) -> Value {
    candid::decode_one(bytes).unwrap()
}
