//! Explicit fixed-width little-endian packed fields with optional element bounds.
// Contract and minimal algorithm adapted from pinned IC Wire; see docs/design/packed.md.

use crate::Error;
use alloc::vec::Vec;
use core::fmt;
use core::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[cfg(test)]
mod tests;

/// Qualified fixed-width packed words. Sealed; arbitrary-bit casts never admit
/// booleans, nonzero integers or floating-point domain values.
pub trait PackedElement: word::Word {}
mod word {
    // Readers receive exact-width chunks after visitor admission. Keep the
    // checked conversion simple: a custom expect message retains a per-word
    // Result::expect call under opt-z (see the codec benchmark investigation).
    pub trait Word: bytemuck::Pod + core::fmt::Debug + Eq {
        const SCHEMA: &'static str;
        fn read(bytes: &[u8]) -> Self;
        #[cfg(any(target_endian = "big", test))]
        fn append(self, bytes: &mut alloc::vec::Vec<u8>);
    }
    macro_rules! integer {
        ($($ty:ty),+) => {$(
            impl super::PackedElement for $ty {}
            impl Word for $ty {
                const SCHEMA: &'static str = stringify!($ty);
                fn read(bytes: &[u8]) -> Self { Self::from_le_bytes(bytes.try_into().unwrap()) }
                #[cfg(any(target_endian = "big", test))]
                fn append(self, bytes: &mut alloc::vec::Vec<u8>) { bytes.extend_from_slice(&self.to_le_bytes()); }
            }
        )+};
    }
    integer!(i8, i16, u16, i32, u32, i64, u64);
    macro_rules! opaque {
        ($n:literal,$schema:literal) => {
            impl super::PackedElement for [u8; $n] {}
            impl Word for [u8; $n] {
                const SCHEMA: &'static str = $schema;
                fn read(bytes: &[u8]) -> Self {
                    bytes.try_into().unwrap()
                }
                #[cfg(any(target_endian = "big", test))]
                fn append(self, bytes: &mut alloc::vec::Vec<u8>) {
                    bytes.extend_from_slice(&self);
                }
            }
        };
    }
    opaque!(16, "hash16");
    opaque!(32, "hash32");
}

/// Explicit fixed-width little-endian byte buffer. Prefix counts bytes, not
/// elements. Ordinary vectors retain their original Serde representation.
/// N limits the number of words; the default is unbounded apart from codec limits.
///
/// ```compile_fail
/// let value = ic_wire::Packed::<f32>::new(vec![1.0_f32]).unwrap();
/// fn serializable(_: impl serde::Serialize) {}
/// serializable(value);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Packed<T, const N: usize = { usize::MAX }>(Vec<T>);
impl<T, const N: usize> Packed<T, N> {
    /// Admit owned words without copying; codec limits also apply on traversal.
    /// # Errors
    /// Returns [`Error::Limit`] when the element count exceeds N.
    pub fn new(words: Vec<T>) -> Result<Self, Error> {
        if words.len() > N {
            Err(Error::Limit)
        } else {
            Ok(Self(words))
        }
    }
    /// Borrow the words.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    /// Recover storage without copying.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}
impl<T: PackedElement, const N: usize> Serialize for Packed<T, N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_words(&self.0, serializer)
    }
}
impl<'de, T: PackedElement, const N: usize> Deserialize<'de> for Packed<T, N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_bytes(PackedVisitor::<T> {
                limit: N,
                marker: PhantomData,
            })
            .map(Self)
    }
}

pub(crate) fn serialize_words<T: PackedElement, S: Serializer>(
    words: &[T],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    #[cfg(target_endian = "little")]
    {
        serializer.serialize_bytes(bytemuck::must_cast_slice(words))
    }
    #[cfg(target_endian = "big")]
    {
        let bytes = little_endian_bytes::<T, S::Error>(words)?;
        serializer.serialize_bytes(&bytes)
    }
}

// Compiled in native tests as well as on big-endian targets, so conversion is
// exercised without claiming a native big-endian execution result.
#[cfg(any(target_endian = "big", test))]
fn little_endian_bytes<T: PackedElement, E: serde::ser::Error>(words: &[T]) -> Result<Vec<u8>, E> {
    let length = words
        .len()
        .checked_mul(size_of::<T>())
        .ok_or_else(|| E::custom("length overflow"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| E::custom("allocation"))?;
    for word in words {
        word.append(&mut bytes);
    }
    Ok(bytes)
}

// Returning Vec keeps traversal accounting independent of the wrapper. The runtime
// bound keeps one allocation/copy body rather than duplicating it per N.
struct PackedVisitor<T> {
    limit: usize,
    marker: PhantomData<T>,
}

impl<T: PackedElement> de::Visitor<'_> for PackedVisitor<T> {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("packed little-endian words")
    }

    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        let width = size_of::<T>();
        if !bytes.len().is_multiple_of(width) || bytes.len() / width > self.limit {
            return Err(E::custom("invalid packed length"));
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(bytes.len() / width)
            .map_err(|_| E::custom("allocation"))?;
        values.extend(bytes.chunks_exact(width).map(T::read));
        Ok(values)
    }
}
