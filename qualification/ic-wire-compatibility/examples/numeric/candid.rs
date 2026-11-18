//! Single-argument Candid integer list probe.
#![cfg(any(test, target_family = "wasm"))]
type Value = Vec<u32>;
fn prepare(words: Vec<u32>) -> Value {
    words
}
fn encode(value: &Value) -> Vec<u8> {
    candid::encode_one(value).unwrap()
}
fn decode(bytes: &[u8]) -> Value {
    candid::decode_one(bytes).unwrap()
}
include!("probe.inc");
