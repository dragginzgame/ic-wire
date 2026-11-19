//! Single-argument Candid integer list probe.
#![cfg_attr(all(target_arch = "wasm32", not(test)), no_main)]

mod probe;

#[cfg(all(not(test), not(target_arch = "wasm32")))]
use probe::host::allocations as main;

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
