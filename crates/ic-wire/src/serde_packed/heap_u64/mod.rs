//! Packed little-endian bytes for `alloc::collections::BinaryHeap<u64>`.
//! Encoding preserves the heap's backing order. Decoding accepts arbitrary order
//! and duplicates, rebuilding the heap; re-encoding may therefore differ.
//! Generated clients use `BigUint64Array`, without heap normalization.
use alloc::collections::BinaryHeap;
use serde::{Deserialize, Deserializer, Serializer};

/// Serialize the heap's backing slice without copying into a temporary vector.
pub fn serialize<S: Serializer>(
    values: &BinaryHeap<u64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    crate::packed::serialize_words(values.as_slice(), serializer)
}

/// Decode owned words and rebuild the heap.
pub fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BinaryHeap<u64>, D::Error> {
    Ok(BinaryHeap::from(
        crate::Packed::<u64>::deserialize(deserializer)?.into_vec(),
    ))
}
