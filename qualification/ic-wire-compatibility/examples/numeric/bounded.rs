//! Bounded varint integer list probe.
#![cfg(any(test, target_family = "wasm"))]
type Value = wire::List<u32, 4096>;
fn prepare(words: Vec<u32>) -> Value {
    Value::new(words).unwrap()
}
include!("wire.inc");
include!("probe.inc");
