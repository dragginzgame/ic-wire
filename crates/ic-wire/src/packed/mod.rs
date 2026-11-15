//! Explicit bounded little-endian i16 byte fields.
// Contract and minimal algorithm adapted from pinned IC Wire; see docs/design/packed.md.

use crate::Error;
use alloc::vec::Vec;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[cfg(test)]
mod tests;

/// Owned signed 16-bit words, bounded in elements and encoded as a byte buffer.
///
/// The wire prefix counts bytes followed by fixed-width little-endian words.
/// Unlike ordinary lists, this does not use ZigZag varints. Switching a field
/// changes its representation and requires coordinated consumers. Small values
/// can require more bytes than ordinary lists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackedI16<const N: usize>(Vec<i16>);

impl<const N: usize> PackedI16<N> {
    /// Admit owned words without copying.
    ///
    /// # Errors
    /// Returns [`Error::Limit`] when the element count exceeds N.
    pub fn new(words: Vec<i16>) -> Result<Self, Error> {
        if words.len() > N {
            Err(Error::Limit)
        } else {
            Ok(Self(words))
        }
    }

    /// Borrow the admitted words.
    #[must_use]
    pub fn as_slice(&self) -> &[i16] {
        &self.0
    }

    /// Recover the owned words without copying.
    #[must_use]
    pub fn into_vec(self) -> Vec<i16> {
        self.0
    }
}

impl<const N: usize> Serialize for PackedI16<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[cfg(target_endian = "little")]
        {
            serializer.serialize_bytes(bytemuck::must_cast_slice(&self.0))
        }
        #[cfg(target_endian = "big")]
        {
            let bytes = little_endian_bytes::<S::Error>(&self.0)?;
            serializer.serialize_bytes(&bytes)
        }
    }
}

// Compiled in native tests as well as on big-endian targets, so conversion is
// exercised without claiming a native big-endian execution result.
#[cfg(any(target_endian = "big", test))]
fn little_endian_bytes<E: serde::ser::Error>(words: &[i16]) -> Result<Vec<u8>, E> {
    let length = words
        .len()
        .checked_mul(2)
        .ok_or_else(|| E::custom("length overflow"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| E::custom("allocation"))?;
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    Ok(bytes)
}

impl<'de, const N: usize> Deserialize<'de> for PackedI16<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_bytes(PackedVisitor { limit: N })
            .map(Self)
    }
}

// Keep the reference's Vec result size for traversal admission. The runtime
// bound keeps one allocation/copy body rather than duplicating it per N.
struct PackedVisitor {
    limit: usize,
}

impl de::Visitor<'_> for PackedVisitor {
    type Value = Vec<i16>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded packed little-endian i16 words")
    }

    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        let (words, remainder) = bytes.as_chunks::<2>();
        if !remainder.is_empty() || words.len() > self.limit {
            return Err(E::custom("invalid packed length"));
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(words.len())
            .map_err(|_| E::custom("allocation"))?;
        values.extend(words.iter().map(|word| i16::from_le_bytes(*word)));
        Ok(values)
    }
}
