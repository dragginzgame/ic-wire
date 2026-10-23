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
        #[cfg(target_endian = "little")]
        {
            serializer.serialize_bytes(bytemuck::must_cast_slice(&self.0))
        }
        #[cfg(target_endian = "big")]
        {
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(self.0.len() * 2)
                .map_err(|_| serde::ser::Error::custom("allocation"))?;
            bytes.extend(self.0.iter().flat_map(|value| value.to_le_bytes()));
            serializer.serialize_bytes(&bytes)
        }
    }
}
impl<'de, const N: usize> Deserialize<'de> for PackedI16<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<const N: usize>;
        impl<const N: usize> de::Visitor<'_> for Visitor<N> {
            type Value = PackedI16<N>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bounded packed little-endian i16 words")
            }
            fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                // Reject incomplete words and the element bound before allocation.
                // Every complete LE word is valid; canonical length checking is
                // still provided by the native codec's exact reserialization.
                if !bytes.len().is_multiple_of(2) || bytes.len() / 2 > N {
                    return Err(E::custom("invalid packed length"));
                }
                let mut values = Vec::new();
                values
                    .try_reserve_exact(bytes.len() / 2)
                    .map_err(|_| E::custom("allocation"))?;
                values.extend(
                    bytes
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|word| i16::from_le_bytes(*word)),
                );
                Ok(PackedI16(values))
            }
        }
        deserializer.deserialize_bytes(Visitor::<N>)
    }
}
