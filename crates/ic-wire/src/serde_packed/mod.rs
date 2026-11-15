//! Explicit bulk field adapters within the single Serde/Postcard codec.
//!
//! Select using `#[serde(with = "ic_wire::serde_packed::bools")]` (or another
//! adapter below). Ordinary fields are never rewritten. Adoption changes traversal
//! accounting and, except for bools, ordinary-list bytes. Preserve field bounds,
//! coordinate clients and retain original retry/hash bytes. Reference contracts
//! and qualification are recorded in docs/design/parity.md.

/// Checked boolean bytes, projected as a browser boolean array.
pub mod bools {
    use alloc::vec::Vec;
    use serde::{Deserializer, Serializer, de};
    /// Borrow boolean storage as bytes; only 0/1 patterns can occur.
    /// # Errors
    /// Propagates serializer refusal.
    pub fn serialize<S: Serializer>(values: &[bool], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(bytemuck::must_cast_slice(values))
    }
    /// Validate every byte before reserving owned output.
    /// # Errors
    /// Rejects invalid boolean patterns and propagates admission/allocation errors.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<bool>, D::Error> {
        struct Visitor;
        impl de::Visitor<'_> for Visitor {
            type Value = Vec<bool>;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("boolean bytes")
            }
            fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                let words: &[bool] = bytemuck::checked::try_cast_slice(bytes)
                    .map_err(|_| E::custom("invalid boolean byte"))?;
                let mut output = Vec::new();
                output
                    .try_reserve_exact(words.len())
                    .map_err(|_| E::custom("allocation"))?;
                output.extend_from_slice(words);
                Ok(output)
            }
        }
        deserializer.deserialize_bytes(Visitor)
    }
}

/// Packed signed words retaining the nonzero domain rule.
pub mod nonzero_i64 {
    use alloc::vec::Vec;
    use core::num::NonZeroI64;
    use serde::{Deserialize, Deserializer, Serializer};
    /// Encode borrowed nonzero storage without a temporary integer vector.
    /// # Errors
    /// Propagates serializer refusal.
    pub fn serialize<S: Serializer>(
        values: &[NonZeroI64],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        crate::packed::serialize_words::<i64, S>(bytemuck::must_cast_slice(values), serializer)
    }
    /// Decode packed words and enforce nonzero values, as in the reference.
    /// # Errors
    /// Rejects zero and propagates packed decoding failures. Validation follows
    /// owned word decoding; it is not a pre-allocation validation guarantee.
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
}

/// Packed heap backing order; decoding reconstructs the heap.
/// Re-encoding may reorder accepted input. Retain original signed/retry bytes.
pub mod heap_u64 {
    use alloc::collections::BinaryHeap;
    use serde::{Deserialize, Deserializer, Serializer};
    /// Encode the existing backing slice without copying.
    /// # Errors
    /// Propagates serializer refusal.
    pub fn serialize<S: Serializer>(
        values: &BinaryHeap<u64>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        crate::packed::serialize_words(values.as_slice(), serializer)
    }
    /// Rebuild a heap from owned packed words, accepting duplicates and any order.
    /// # Errors
    /// Propagates packed decoding failures.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BinaryHeap<u64>, D::Error> {
        Ok(BinaryHeap::from(
            crate::Packed::<u64>::deserialize(deserializer)?.into_vec(),
        ))
    }
}

// A shared source pattern for two domain adapters. No consumer-specific codec
// generation or inherited unrolled/scanner optimization without new measurements.
macro_rules! finite {
    ($module:ident,$float:ty,$width:literal) => {
        /// Finite little-endian float byte fields. Preserves signed zero and subnormals.
        pub mod $module {
            use alloc::vec::Vec;
            use serde::{Deserializer, Serializer, de};
            fn valid(bytes: &[u8]) -> bool {
                if !bytes.len().is_multiple_of($width) {
                    return false;
                }
                let mut tail = bytes;
                if $width == 4 {
                    // Independent f32 exponent lanes: adding one exponent unit
                    // reaches each sign bit exactly for exponent 0xff. Masking
                    // first prevents mantissa/sign bits and cross-lane carries.
                    let mut pairs = bytes.chunks_exact(8);
                    if !pairs.by_ref().all(|pair| {
                        let bits = u64::from_le_bytes(pair.try_into().unwrap());
                        ((bits & 0x7f80_0000_7f80_0000) + 0x0080_0000_0080_0000)
                            & 0x8000_0000_8000_0000
                            == 0
                    }) {
                        return false;
                    }
                    tail = pairs.remainder();
                }
                tail.chunks_exact($width).all(|word| {
                    <$float>::from_le_bytes(word.try_into().expect("whole float word")).is_finite()
                })
            }
            /// Encode finite values as one byte buffer without narrowing finite bits.
            /// # Errors
            /// Rejects NaN/infinity and propagates serializer/allocation failures.
            pub fn serialize<S: Serializer>(
                values: &[$float],
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                #[cfg(target_endian = "little")]
                {
                    let bytes = bytemuck::must_cast_slice(values);
                    if !valid(bytes) {
                        return Err(serde::ser::Error::custom("non-finite float"));
                    }
                    serializer.serialize_bytes(bytes)
                }
                #[cfg(target_endian = "big")]
                {
                    serializer.serialize_bytes(&little_endian::<S::Error>(values)?)
                }
            }
            #[cfg(any(target_endian = "big", test))]
            pub(super) fn little_endian<E: serde::ser::Error>(
                values: &[$float],
            ) -> Result<Vec<u8>, E> {
                if values.iter().any(|value| !value.is_finite()) {
                    return Err(E::custom("non-finite float"));
                }
                let length = values
                    .len()
                    .checked_mul($width)
                    .ok_or_else(|| E::custom("length overflow"))?;
                let mut bytes = Vec::new();
                bytes
                    .try_reserve_exact(length)
                    .map_err(|_| E::custom("allocation"))?;
                for value in values {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
                Ok(bytes)
            }
            /// Reject partial/nonfinite words before allocating owned float storage.
            /// # Errors
            /// Rejects invalid spans and propagates admission/allocation failures.
            pub fn deserialize<'de, D: Deserializer<'de>>(
                deserializer: D,
            ) -> Result<Vec<$float>, D::Error> {
                struct Visitor;
                impl de::Visitor<'_> for Visitor {
                    type Value = Vec<$float>;
                    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        f.write_str("finite little-endian float bytes")
                    }
                    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                        if !valid(bytes) {
                            return Err(E::custom("invalid finite float bytes"));
                        }
                        let mut values = Vec::new();
                        values
                            .try_reserve_exact(bytes.len() / $width)
                            .map_err(|_| E::custom("allocation"))?;
                        values.extend(bytes.chunks_exact($width).map(|word| {
                            <$float>::from_le_bytes(
                                word.try_into().expect("validated whole float word"),
                            )
                        }));
                        Ok(values)
                    }
                }
                deserializer.deserialize_bytes(Visitor)
            }
        }
    };
}
finite!(finite_f32, f32, 4);
finite!(finite_f64, f64, 8);
#[cfg(test)]
mod tests;
