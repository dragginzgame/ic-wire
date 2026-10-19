//! Input-backed finite byte and text values. Deserializers must lend their input.
use super::{Bytes, Text};
use crate::{Error, Shape, Wire, schema::Schema};
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Opaque bytes bounded in length, borrowing their backing storage.
/// Decoding accepts borrowed input only; the value cannot outlive that input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BytesRef<'a, const N: usize>(&'a [u8]);
impl<'a, const N: usize> BytesRef<'a, N> {
    /// Admit a borrowed slice without copying or allocating.
    pub fn new(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() > N {
            Err(Error::Limit)
        } else {
            Ok(Self(bytes))
        }
    }
    /// Borrow the admitted bytes for their original storage lifetime.
    pub fn as_slice(&self) -> &'a [u8] {
        self.0
    }
    pub(super) fn from_owned(bytes: &'a Bytes<N>) -> Self {
        Self(bytes.as_slice())
    }
}
impl<const N: usize> Serialize for BytesRef<'_, N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(self.0)
    }
}
impl<'de: 'a, 'a, const N: usize> Deserialize<'de> for BytesRef<'a, N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor<const N: usize>;
        impl<'de, const N: usize> de::Visitor<'de> for Visitor<N> {
            type Value = BytesRef<'de, N>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bounded borrowed bytes")
            }
            fn visit_borrowed_bytes<E: de::Error>(
                self,
                bytes: &'de [u8],
            ) -> Result<Self::Value, E> {
                BytesRef::new(bytes).map_err(|_| E::custom("byte limit"))
            }
        }
        d.deserialize_bytes(Visitor::<N>)
    }
}
impl<const N: usize> Wire for BytesRef<'_, N> {
    const SHAPE: Shape = Shape {
        heap: 0,
        ..Bytes::<N>::SHAPE
    };
    const SCHEMA: Schema = Bytes::<N>::SCHEMA;
}

/// UTF-8 text bounded in bytes, borrowing its backing storage.
/// Decoding accepts borrowed input only, without an owned fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextRef<'a, const N: usize>(&'a str);
impl<'a, const N: usize> TextRef<'a, N> {
    /// Admit borrowed text without copying or allocating.
    pub fn new(text: &'a str) -> Result<Self, Error> {
        if text.len() > N {
            Err(Error::Limit)
        } else {
            Ok(Self(text))
        }
    }
    /// Borrow the admitted text for its original storage lifetime.
    pub fn as_str(&self) -> &'a str {
        self.0
    }
    pub(super) fn from_owned(text: &'a Text<N>) -> Self {
        Self(text.as_str())
    }
}
impl<const N: usize> Serialize for TextRef<'_, N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.0)
    }
}
impl<'de: 'a, 'a, const N: usize> Deserialize<'de> for TextRef<'a, N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor<const N: usize>;
        impl<'de, const N: usize> de::Visitor<'de> for Visitor<N> {
            type Value = TextRef<'de, N>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bounded borrowed UTF-8 text")
            }
            fn visit_borrowed_str<E: de::Error>(self, text: &'de str) -> Result<Self::Value, E> {
                TextRef::new(text).map_err(|_| E::custom("text limit"))
            }
        }
        d.deserialize_str(Visitor::<N>)
    }
}
impl<const N: usize> Wire for TextRef<'_, N> {
    const SHAPE: Shape = Shape {
        heap: 0,
        ..Text::<N>::SHAPE
    };
    const SCHEMA: Schema = Text::<N>::SCHEMA;
}
