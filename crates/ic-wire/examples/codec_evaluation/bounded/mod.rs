//! Bounds for the finite evaluation schemas, not a public codec API.

use candid::CandidType;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{fmt, marker::PhantomData};

use crate::Error;

pub(super) const MAX_TEXT: usize = 64;
pub(super) const MAX_BLOB: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, CandidType)]
#[serde(transparent)]
pub(super) struct Text(String);

impl Text {
    pub(super) fn new(value: &str) -> Result<Self, Error> {
        if value.len() > MAX_TEXT {
            return Err(Error::Limit);
        }
        Ok(Self(value.to_owned()))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TextVisitor;
        impl de::Visitor<'_> for TextVisitor {
            type Value = Text;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("UTF-8 text of at most 64 bytes")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Text, E> {
                Text::new(value).map_err(E::custom)
            }
        }
        // Request a borrowed/string-slice visit. Ciborium uses its bounded
        // scratch space, avoiding its allocating deserialize_string path.
        deserializer.deserialize_str(TextVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType)]
pub(super) struct Bytes(Vec<u8>);

impl Bytes {
    pub(super) fn new(value: &[u8]) -> Result<Self, Error> {
        if value.len() > MAX_BLOB {
            return Err(Error::Limit);
        }
        Ok(Self(value.to_vec()))
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for Bytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BytesVisitor;
        impl de::Visitor<'_> for BytesVisitor {
            type Value = Bytes;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a byte string of at most 4096 bytes")
            }
            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Bytes, E> {
                Bytes::new(value).map_err(E::custom)
            }
        }
        deserializer.deserialize_bytes(BytesVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, CandidType)]
#[serde(transparent)]
pub(super) struct List<T, const N: usize>(Vec<T>);

impl<T, const N: usize> List<T, N> {
    pub(super) fn new(value: Vec<T>) -> Result<Self, Error> {
        if value.len() > N {
            return Err(Error::Limit);
        }
        Ok(Self(value))
    }

    pub(super) fn as_slice(&self) -> &[T] {
        &self.0
    }
}

// When an indefinite sequence reaches its limit, reject the next element
// before asking its type to decode or allocate. End-of-sequence still succeeds.
struct ExcessElement;

impl<'de> Deserialize<'de> for ExcessElement {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(de::Error::custom("element limit exceeded"))
    }
}

impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for List<T, N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ListVisitor<T, const N: usize>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>, const N: usize> de::Visitor<'de> for ListVisitor<T, N> {
            type Value = List<T, N>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a sequence of at most {N} elements")
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                if seq.size_hint().is_some_and(|len| len > N) {
                    return Err(de::Error::custom("element limit exceeded"));
                }
                let mut values = Vec::new();
                // Reserve the finite schema bound, never an untrusted count.
                values.try_reserve_exact(N).map_err(de::Error::custom)?;
                while values.len() < N {
                    let Some(value) = seq.next_element()? else {
                        return Ok(List(values));
                    };
                    values.push(value);
                }
                let _: Option<ExcessElement> = seq.next_element()?;
                Ok(List(values))
            }
        }
        deserializer.deserialize_seq(ListVisitor::<T, N>(PhantomData))
    }
}
