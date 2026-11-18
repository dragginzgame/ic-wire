//! Ordinary integer list probe.
#![cfg(any(test, target_family = "wasm"))]
type Value = Vec<u32>;
fn prepare(words: Vec<u32>) -> Value {
    words
}
include!("wire.inc");
include!("probe.inc");
