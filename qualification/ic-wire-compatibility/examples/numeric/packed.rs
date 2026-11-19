//! Bounded fixed-width integer list probe.
#![cfg_attr(all(target_arch = "wasm32", not(test)), no_main)]

mod probe;
mod wire_codec;

use wire_codec::{decode, encode};

#[cfg(all(not(test), not(target_arch = "wasm32")))]
use probe::host::allocations as main;

type Value = wire::Packed<u32, 4096>;

fn prepare(words: Vec<u32>) -> Value {
    Value::new(words).unwrap()
}
