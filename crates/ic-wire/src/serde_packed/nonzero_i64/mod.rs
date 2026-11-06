//! Packed little-endian bytes for `Vec<core::num::NonZeroI64>`.
//! Generated clients use `BigInt64Array` and reject zero on both encode and decode.
use alloc::vec::Vec;
use core::num::NonZeroI64;
use serde::{Deserialize, Deserializer, Serializer};

/// Serialize the existing storage without constructing a temporary integer vector.
pub fn serialize<S: Serializer>(values: &[NonZeroI64], serializer: S) -> Result<S::Ok, S::Error> {
    // NonZeroI64 has no uninitialized bytes; all its patterns are valid i64.
    let words: &[i64] = bytemuck::must_cast_slice(values);
    crate::packed::serialize_words(words, serializer)
}

/// Decode owned words and validate every nonzero invariant.
pub fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<NonZeroI64>, D::Error> {
    crate::Packed::<i64>::deserialize(deserializer)?
        .into_vec()
        .into_iter()
        .map(NonZeroI64::new)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| serde::de::Error::custom("zero in nonzero list"))
}
