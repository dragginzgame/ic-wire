//! Fixed-width bulk fields within the single Serde/Postcard codec.
use alloc::vec::Vec;
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
};
use serde::Serialize;

/// Supported fixed-width elements of [`Packed`]. Sealed to the qualified types.
/// Custom types, floats and platform-sized integers are deliberately excluded.
pub trait PackedElement: sealed::Word {}
mod sealed {
    use super::*;
    pub trait Word: bytemuck::Pod + Debug + Eq {
        #[cfg(feature = "typescript")]
        const SCHEMA: &'static str;
        fn read_le(bytes: &[u8]) -> Self;
        #[cfg(any(target_endian = "big", test))]
        fn append_le(self, bytes: &mut Vec<u8>);
    }
    // These concrete readers cross the crate boundary when a consumer instantiates
    // Packed<T>. Expose their bodies for specialization of the bulk copy loop.
    macro_rules! words {
    ($($ty:ty),*) => {$(
        impl super::PackedElement for $ty {}
        impl Word for $ty {
            #[cfg(feature = "typescript")]
            const SCHEMA: &'static str = stringify!($ty);
            #[inline]
            fn read_le(bytes: &[u8]) -> Self {
                Self::from_le_bytes(bytes.try_into().unwrap())
            }
            #[cfg(any(target_endian = "big", test))]
            #[inline]
            fn append_le(self, bytes: &mut Vec<u8>) {
                bytes.extend_from_slice(&self.to_le_bytes());
            }
        }
    )*};
}
    words!(i16, u16, i32, u32, i64, u64);
    impl super::PackedElement for [u8; 32] {}
    impl Word for [u8; 32] {
        #[cfg(feature = "typescript")]
        const SCHEMA: &'static str = "hash32";
        #[inline]
        fn read_le(bytes: &[u8]) -> Self {
            bytes.try_into().unwrap()
        }
        #[cfg(any(target_endian = "big", test))]
        #[inline]
        fn append_le(self, bytes: &mut Vec<u8>) {
            bytes.extend_from_slice(&self);
        }
    }
}

/// Owned fixed-width words serialized as one Postcard byte buffer.
///
/// Supports i16/u16/i32/u32/i64/u64 and 32-byte hashes. Runtime [`crate::Limits`]
/// bound admission; this type has no per-field element limit. Small integers
/// may use more bytes than ordinary lists. Switching from Vec requires coordinated
/// consumers; see the packed-field guide.
///
/// ```compile_fail
/// let unsupported = ic_wire::Packed::new(vec![1.0_f32]);
/// fn serializable(_: impl serde::Serialize) {}
/// serializable(unsupported);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Packed<T>(Vec<T>);
impl<T> Packed<T> {
    /// Wrap owned words without copying. Admission happens at the codec boundary.
    pub fn new(values: Vec<T>) -> Self {
        Self(values)
    }
    /// Borrow the words.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    /// Recover the owned words without copying.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}
impl<T: PackedElement> Serialize for Packed<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Opt-in field layout: element count + per-element encoding becomes
        // byte count + fixed LE words. [32767_i16]: 01 fe ff 03 -> 02 ff 7f.
        // One 32-byte hash changes its prefix 01 -> 20; hash bytes are unchanged.
        // Adoption requires coordinated schemas/clients and retained retry bytes;
        // per-element traversal becomes byte-buffer accounting. Packed<T>
        // has no field-specific N; consumers requiring one need an explicit
        // field constraint. Ordinary Vec encoding is unchanged.
        #[cfg(target_endian = "little")]
        {
            serializer.serialize_bytes(bytemuck::must_cast_slice(&self.0))
        }
        #[cfg(target_endian = "big")]
        {
            let mut bytes = Vec::new();
            let len = self
                .0
                .len()
                .checked_mul(size_of::<T>())
                .ok_or_else(|| serde::ser::Error::custom("length overflow"))?;
            bytes
                .try_reserve_exact(len)
                .map_err(|_| serde::ser::Error::custom("allocation"))?;
            for value in &self.0 {
                value.append_le(&mut bytes);
            }
            serializer.serialize_bytes(&bytes)
        }
    }
}
impl<'de, T: PackedElement> serde::Deserialize<'de> for Packed<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_bytes(PackedVisitor {
                marker: PhantomData,
            })
            .map(Self)
    }
}
// Only the word type is part of the field type. No per-field N.
// Native admission checks the byte buffer before this visitor allocates;
// unlike the prior prototype, the field relies on runtime Limits like Vec<T>.
struct PackedVisitor<T> {
    marker: PhantomData<T>,
}
impl<T: PackedElement> serde::de::Visitor<'_> for PackedVisitor<T> {
    type Value = Vec<T>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("packed little-endian words")
    }
    #[inline(never)]
    fn visit_bytes<E: serde::de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        let width = size_of::<T>();
        if !bytes.len().is_multiple_of(width) {
            return Err(E::custom("invalid packed length"));
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(bytes.len() / width)
            .map_err(|_| E::custom("allocation"))?;
        values.extend(bytes.chunks_exact(width).map(T::read_le));
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn endian_append_matches_wire_bytes() {
        use super::sealed::Word;
        use alloc::vec::Vec;
        macro_rules! check {
            ($($ty:ty),*) => {$(
                let words = [<$ty>::MIN, 1, <$ty>::MAX];
                let mut bytes = Vec::new();
                for word in words { word.append_le(&mut bytes); }
                let expected: Vec<_> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
                assert_eq!(bytes, expected);
                for (chunk, word) in bytes.chunks_exact(size_of::<$ty>()).zip(words) {
                    assert_eq!(<$ty>::read_le(chunk), word);
                }
            )*};
        }
        check!(i16, u16, i32, u32, i64, u64);
        let hash = core::array::from_fn::<_, 32, _>(|i| i as u8);
        let mut bytes = Vec::new();
        hash.append_le(&mut bytes);
        assert_eq!(bytes, hash);
        assert_eq!(<[u8; 32]>::read_le(&bytes), hash);
    }
}
