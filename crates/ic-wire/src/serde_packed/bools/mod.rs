//! Bulk byte-buffer encoding for `Vec<bool>`, with `boolean[]` browser values.
//!
//! Select explicitly with `#[serde(with = "ic_wire::serde_packed::bools")]`.
//! Valid bytes match ordinary bool lists (`[false, true]` stays `02 00 01`),
//! but admission changes from element visits to one byte buffer. Regenerate
//! clients and review field bounds and runtime limits before adoption.
use alloc::vec::Vec;
use core::fmt;
use serde::{Deserializer, Serializer, de};

/// Serialize borrowed boolean storage without a temporary byte vector.
pub fn serialize<S: Serializer>(values: &[bool], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(bytemuck::must_cast_slice(values))
}

/// Validate every byte before allocating the owned boolean vector.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<bool>, D::Error> {
    struct Visitor;
    impl de::Visitor<'_> for Visitor {
        type Value = Vec<bool>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("boolean bytes")
        }
        fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
            // bool has one-byte alignment but only 00/01 are valid. The safe
            // checked cast rejects all other patterns before allocation/copy.
            let words: &[bool] = bytemuck::checked::try_cast_slice(bytes)
                .map_err(|_| E::custom("invalid boolean byte"))?;
            let mut values = Vec::new();
            values
                .try_reserve_exact(words.len())
                .map_err(|_| E::custom("allocation"))?;
            values.extend_from_slice(words);
            Ok(values)
        }
    }
    deserializer.deserialize_bytes(Visitor)
}
