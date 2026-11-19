//! Ordinary integer list probe.
#![cfg_attr(all(target_arch = "wasm32", not(test)), no_main)]

mod wire_codec;

use wire_codec::{decode, encode};

include!("probe/entry.rs");

type Value = Vec<u32>;

fn prepare(words: Vec<u32>) -> Value {
    words
}
