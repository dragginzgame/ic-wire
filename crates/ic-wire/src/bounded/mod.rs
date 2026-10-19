use crate::{Error, Shape, Wire};
use alloc::{string::String, vec::Vec};
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// A sequence with a compile-time element bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct List<T, const N: usize>(Vec<T>);
impl<T, const N: usize> List<T, N> {
    /// Admit an owned sequence without copying.
    pub fn new(values: Vec<T>) -> Result<Self, Error> {
        if values.len() > N {
            Err(Error::Limit)
        } else {
            Ok(Self(values))
        }
    }
    /// Borrow the elements.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
    /// Recover the admitted sequence.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}
impl<T: Serialize, const N: usize> Serialize for List<T, N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}
impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for List<T, N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor<T, const N: usize>(core::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>, const N: usize> de::Visitor<'de> for Visitor<T, N> {
            type Value = List<T, N>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bounded sequence")
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                // Postcard hides its count when remaining bytes are fewer than
                // elements (including legitimate unit values). Bound both paths.
                let count = seq.size_hint();
                if count.is_some_and(|count| count > N) {
                    return Err(de::Error::custom("element limit"));
                }
                let mut values = Vec::new();
                values
                    .try_reserve_exact(count.unwrap_or(0))
                    .map_err(|_| de::Error::custom("allocation"))?;
                for _ in 0..N {
                    let Some(value) = seq.next_element()? else {
                        return Ok(List(values));
                    };
                    if values.len() == values.capacity() {
                        // Hidden lengths need amortized growth, capped by the
                        // declared element bound rather than Vec's growth policy.
                        let next = values.len().saturating_mul(2).max(1).min(N);
                        values
                            .try_reserve_exact(next - values.len())
                            .map_err(|_| de::Error::custom("allocation"))?;
                    }
                    values.push(value);
                }
                // Detect an excess element without invoking its decoder.
                struct Reject;
                impl<'de> de::DeserializeSeed<'de> for Reject {
                    type Value = ();
                    fn deserialize<D: Deserializer<'de>>(self, _: D) -> Result<(), D::Error> {
                        Err(de::Error::custom("element limit"))
                    }
                }
                seq.next_element_seed(Reject)?;
                Ok(List(values))
            }
        }
        d.deserialize_seq(Visitor::<T, N>(core::marker::PhantomData))
    }
}
impl<T: Wire, const N: usize> Wire for List<T, N> {
    const SCHEMA: crate::schema::Schema = crate::schema::Schema::List(&T::SCHEMA, N);
    const SHAPE: Shape = Shape {
        bytes: 10_usize.saturating_add(T::SHAPE.bytes.saturating_mul(N)),
        heap: N.saturating_mul(core::mem::size_of::<T>().saturating_add(T::SHAPE.heap)),
        nodes: 1_usize.saturating_add(N.saturating_mul(T::SHAPE.nodes)),
        depth: T::SHAPE.depth.saturating_add(1),
    };
}

/// Bounded opaque bytes; consumers validate principal bytes using their existing
/// principal type. IC Wire defines no new identity type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bytes<const N: usize>(Vec<u8>);
impl<const N: usize> Bytes<N> {
    /// Admit bytes without copying.
    pub fn new(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() > N {
            Err(Error::Limit)
        } else {
            Ok(Self(bytes))
        }
    }
    /// Borrow the bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    /// Recover the admitted bytes without copying or allocating.
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}
impl<const N: usize> Serialize for Bytes<N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(&self.0)
    }
}
impl<'de, const N: usize> Deserialize<'de> for Bytes<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor<const N: usize>;
        impl<const N: usize> de::Visitor<'_> for Visitor<N> {
            type Value = Bytes<N>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bounded bytes")
            }
            fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                if bytes.len() > N {
                    return Err(E::custom("byte limit"));
                }
                let mut out = Vec::new();
                out.try_reserve_exact(bytes.len())
                    .map_err(|_| E::custom("allocation"))?;
                out.extend_from_slice(bytes);
                Ok(Bytes(out))
            }
        }
        d.deserialize_bytes(Visitor::<N>)
    }
}
impl<const N: usize> Wire for Bytes<N> {
    const SCHEMA: crate::schema::Schema = crate::schema::Schema::Bytes(N);
    const SHAPE: Shape = Shape {
        bytes: N.saturating_add(10),
        heap: N,
        nodes: 1,
        depth: 1,
    };
}

/// UTF-8 text bounded in bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Text<const N: usize>(String);
impl<const N: usize> Text<N> {
    /// Admit owned text without copying.
    pub fn new(text: String) -> Result<Self, Error> {
        if text.len() > N {
            Err(Error::Limit)
        } else {
            Ok(Self(text))
        }
    }
    /// Borrow the text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Recover the admitted text without copying or allocating.
    pub fn into_string(self) -> String {
        self.0
    }
}
impl<const N: usize> Serialize for Text<N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let bytes = Bytes::<N>::deserialize(d)?;
        let text = String::from_utf8(bytes.0).map_err(|_| de::Error::custom("invalid UTF-8"))?;
        Ok(Self(text))
    }
}
impl<const N: usize> Wire for Text<N> {
    const SCHEMA: crate::schema::Schema = crate::schema::Schema::Text(N);
    const SHAPE: Shape = Bytes::<N>::SHAPE;
}
