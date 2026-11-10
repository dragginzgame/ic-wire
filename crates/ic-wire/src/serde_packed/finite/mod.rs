//! Shared source for the two concrete finite-float adapters, not consumer types.
macro_rules! adapter {
    ($float:ty, $width:expr; $($index:expr),+; $($pair:expr),+) => {
        use alloc::vec::Vec;
        use serde::{Deserializer, Serializer, de};

        // Callers establish whole-word lengths. Bounded unrolling reduces loop
        // overhead without generating a codec for each consumer or field bound.
        #[inline(never)]
        fn finite(bytes: &[u8]) -> bool {
            if $width == 4 {
                // Each u64 holds two independent f32 exponents. Adding one to
                // the masked exponent sets its sign bit only for 0xff (NaN or
                // infinity); neither lane can carry into the other. Preserve
                // the same finite-only acceptance, bytes and admission limits.
                let mut chunks = bytes.chunks_exact(32);
                for chunk in chunks.by_ref() {
                    let mut exponent_carries = 0_u64;
                    $(let bits = u64::from_le_bytes(chunk[$pair * 8..($pair + 1) * 8].try_into().unwrap());
                    exponent_carries |= (bits & 0x7f80_0000_7f80_0000) + 0x0080_0000_0080_0000;)+
                    if exponent_carries & 0x8000_0000_8000_0000 != 0 { return false; }
                }
                return chunks.remainder().chunks_exact(4).all(|word|
                    f32::from_le_bytes(word.try_into().unwrap()).is_finite());
            }
            let mut chunks = bytes.chunks_exact($width * 8);
            for chunk in chunks.by_ref() {
                let mut valid = true;
                $(valid &= <$float>::from_le_bytes(chunk[$index * $width..($index + 1) * $width].try_into().unwrap()).is_finite();)+
                if !valid { return false; }
            }
            chunks.remainder().chunks_exact($width)
                .all(|word| <$float>::from_le_bytes(word.try_into().unwrap()).is_finite())
        }

        /// Serialize finite values as a little-endian byte buffer.
        pub fn serialize<S: Serializer>(values: &[$float], serializer: S) -> Result<S::Ok, S::Error> {
            #[cfg(target_endian = "little")]
            {
                let bytes = bytemuck::must_cast_slice(values);
                if !finite(bytes) { return Err(serde::ser::Error::custom("non-finite float")); }
                serializer.serialize_bytes(bytes)
            }
            #[cfg(target_endian = "big")]
            {
                if values.iter().any(|value| !value.is_finite()) {
                    return Err(serde::ser::Error::custom("non-finite float"));
                }
                let length = values.len().checked_mul($width)
                    .ok_or_else(|| serde::ser::Error::custom("length overflow"))?;
                let mut bytes = Vec::new();
                bytes.try_reserve_exact(length).map_err(|_| serde::ser::Error::custom("allocation"))?;
                for value in values { bytes.extend_from_slice(&value.to_le_bytes()); }
                serializer.serialize_bytes(&bytes)
            }
        }

        /// Reject partial or non-finite words before allocating owned output.
        pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<$float>, D::Error> {
            struct Visitor;
            impl de::Visitor<'_> for Visitor {
                type Value = Vec<$float>;
                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("finite little-endian float bytes")
                }
                fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                    if !bytes.len().is_multiple_of($width) || !finite(bytes) {
                        return Err(E::custom("invalid finite float bytes"));
                    }
                    let mut values = Vec::new();
                    values.try_reserve_exact(bytes.len() / $width).map_err(|_| E::custom("allocation"))?;
                    values.extend(bytes.chunks_exact($width).map(|word|
                        <$float>::from_le_bytes(word.try_into().unwrap())));
                    Ok(values)
                }
            }
            deserializer.deserialize_bytes(Visitor)
        }
    };
}
pub(super) use adapter;
