//! Bounded varint integer list probe.
pub(super) type Value = wire::List<u32, 4096>;

pub(super) fn prepare(words: Vec<u32>) -> Value {
    Value::new(words).unwrap()
}
