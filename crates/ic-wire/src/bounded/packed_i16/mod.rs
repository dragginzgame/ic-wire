use crate::Error;
use alloc::vec::Vec;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Bounded signed 16-bit words encoded as little-endian bytes.
///
/// Unlike `List<i16, N>` or `Vec<i16>`, this uses a byte count and fixed-width
/// words instead of an element count and ZigZag varints. Switching a field
/// requires coordinated consumers. Small-magnitude values take more space.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackedI16<const N: usize>(Vec<i16>);
impl<const N: usize> PackedI16<N> {
    /// Admit an owned sequence without copying. `N` counts elements, not bytes.
    pub fn new(values: Vec<i16>) -> Result<Self, Error> {
        if values.len() > N {
            Err(Error::Limit)
        } else {
            Ok(Self(values))
        }
    }
    /// Borrow the admitted words.
    pub fn as_slice(&self) -> &[i16] {
        &self.0
    }
    /// Recover the admitted sequence without copying.
    pub fn into_vec(self) -> Vec<i16> {
        self.0
    }
}
impl<const N: usize> Serialize for PackedI16<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Opt-in field layout: [32767] changes from 01 fe ff 03 (Vec<i16>)
        // to 02 ff 7f (byte count + LE words). Ordinary lists are unchanged;
        // consumers switching this field must coordinate schemas and retry bytes.
        // Share the sealed LE writer only. Decoders retain their own bound and
        // allocation/copy paths; bytes and admission are unchanged.
        crate::packed::serialize_words(&self.0, serializer)
    }
}
impl<'de, const N: usize> Deserialize<'de> for PackedI16<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_bytes(PackedVisitor { limit: N })
            .map(Self)
    }
}

// Share decoding across bounds instead of specializing the allocation/copy loop
// for each N. The field's exact element limit is still checked before allocation;
// byte layout, primitive validation and budget charges remain unchanged.
struct PackedVisitor {
    limit: usize,
}
impl de::Visitor<'_> for PackedVisitor {
    type Value = Vec<i16>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded packed little-endian i16 words")
    }
    #[inline(never)]
    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        crate::packed::decode_words!(
            E,
            bytes,
            2,
            bytes.len() / 2 > self.limit,
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|word| i16::from_le_bytes(*word))
        )
    }
}
