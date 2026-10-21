//! Admission before native payload decoding and visitor allocation.
// Adapted from IcyDB native admission; see ../LICENSE-MIT.

use super::Budget;
use alloc::{string::String, vec::Vec};
use core::fmt;
use serde::{
    Deserializer,
    de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor},
};

pub(super) struct Bounded<'a, D> {
    pub(super) inner: D,
    pub(super) budget: &'a Budget,
    pub(super) depth: usize,
}
struct CheckedVisitor<'a, V> {
    inner: V,
    budget: &'a Budget,
    depth: usize,
}
struct Access<'a, A> {
    inner: A,
    budget: &'a Budget,
    depth: usize,
}
struct Seed<'a, T> {
    inner: T,
    budget: &'a Budget,
    depth: usize,
}

impl<'de, T: DeserializeSeed<'de>> DeserializeSeed<'de> for Seed<'_, T> {
    type Value = T::Value;
    fn deserialize<D: Deserializer<'de>>(self, inner: D) -> Result<Self::Value, D::Error> {
        if !self.budget.enter(self.depth, size_of::<T::Value>()) {
            return Err(de::Error::custom("payload resource allowance exceeded"));
        }
        self.inner.deserialize(Bounded {
            inner,
            budget: self.budget,
            depth: self.depth,
        })
    }
}

// Preserve the exact admission and visitor-depth rule in one expansion.
macro_rules! admit_visitor {
    ($($method:ident),* $(,)?) => {$(
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            if !self.budget.enter(self.depth, size_of::<V::Value>()) {
                return Err(de::Error::custom("payload resource allowance exceeded"));
            }
            self.inner.$method(CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth + 1,
            })
        }
    )*};
}

macro_rules! visit_scalar {
    ($($method:ident: $ty:ty),* $(,)?) => {$(
        fn $method<E: de::Error>(self, value: $ty) -> Result<Self::Value, E> {
            self.inner.$method(value)
        }
    )*};
}

impl<'de, D: Deserializer<'de>> Deserializer<'de> for Bounded<'_, D> {
    type Error = D::Error;
    admit_visitor! {
        deserialize_any,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,

        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_u128,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
    }
    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if !self.budget.enter(self.depth, size_of::<V::Value>()) {
            return Err(de::Error::custom("payload resource allowance exceeded"));
        }
        self.inner.deserialize_unit_struct(
            name,
            CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth + 1,
            },
        )
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if !self.budget.enter(self.depth, size_of::<V::Value>()) {
            return Err(de::Error::custom("payload resource allowance exceeded"));
        }
        self.inner.deserialize_newtype_struct(
            name,
            CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth + 1,
            },
        )
    }
    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if !self.budget.enter(self.depth, size_of::<V::Value>()) {
            return Err(de::Error::custom("payload resource allowance exceeded"));
        }
        self.inner.deserialize_tuple(
            len,
            CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth + 1,
            },
        )
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if !self.budget.enter(self.depth, size_of::<V::Value>()) {
            return Err(de::Error::custom("payload resource allowance exceeded"));
        }
        self.inner.deserialize_tuple_struct(
            name,
            len,
            CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth + 1,
            },
        )
    }
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if !self.budget.enter(self.depth, size_of::<V::Value>()) {
            return Err(de::Error::custom("payload resource allowance exceeded"));
        }
        self.inner.deserialize_struct(
            name,
            fields,
            CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth + 1,
            },
        )
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if !self.budget.enter(self.depth, size_of::<V::Value>()) {
            return Err(de::Error::custom("payload resource allowance exceeded"));
        }
        self.inner.deserialize_enum(
            name,
            variants,
            CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth + 1,
            },
        )
    }
    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}

impl<'de, V: Visitor<'de>> Visitor<'de> for CheckedVisitor<'_, V> {
    type Value = V::Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
    visit_scalar! {
        visit_bool: bool,
        visit_i8: i8,
        visit_i16: i16,
        visit_i32: i32,
        visit_i64: i64,
        visit_i128: i128,
        visit_u8: u8,
        visit_u16: u16,
        visit_u32: u32,
        visit_u64: u64,
        visit_u128: u128,
        visit_f32: f32,
        visit_f64: f64,
        visit_char: char,
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        if !self.budget.bytes(value.len()) {
            return Err(de::Error::custom("payload allocation allowance exceeded"));
        }
        self.inner.visit_str(value)
    }
    fn visit_borrowed_str<E: de::Error>(self, value: &'de str) -> Result<Self::Value, E> {
        if !self.budget.bytes(value.len()) {
            return Err(de::Error::custom("payload allocation allowance exceeded"));
        }
        self.inner.visit_borrowed_str(value)
    }
    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        if !self.budget.bytes(value.len()) {
            return Err(de::Error::custom("payload allocation allowance exceeded"));
        }
        self.inner.visit_string(value)
    }
    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        if !self.budget.bytes(value.len()) {
            return Err(de::Error::custom("payload allocation allowance exceeded"));
        }
        self.inner.visit_bytes(value)
    }
    fn visit_borrowed_bytes<E: de::Error>(self, value: &'de [u8]) -> Result<Self::Value, E> {
        if !self.budget.bytes(value.len()) {
            return Err(de::Error::custom("payload allocation allowance exceeded"));
        }
        self.inner.visit_borrowed_bytes(value)
    }
    fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Self::Value, E> {
        if !self.budget.bytes(value.len()) {
            return Err(de::Error::custom("payload allocation allowance exceeded"));
        }
        self.inner.visit_byte_buf(value)
    }
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        self.inner.visit_none()
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        self.inner.visit_unit()
    }
    fn visit_some<D: Deserializer<'de>>(self, inner: D) -> Result<Self::Value, D::Error> {
        self.inner.visit_some(Bounded {
            inner,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn visit_newtype_struct<D: Deserializer<'de>>(self, inner: D) -> Result<Self::Value, D::Error> {
        self.inner.visit_newtype_struct(Bounded {
            inner,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn visit_seq<A: SeqAccess<'de>>(self, inner: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_seq(Access {
            inner,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn visit_map<A: MapAccess<'de>>(self, inner: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_map(Access {
            inner,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn visit_enum<A: EnumAccess<'de>>(self, inner: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_enum(Access {
            inner,
            budget: self.budget,
            depth: self.depth,
        })
    }
}

impl<'de, A: SeqAccess<'de>> SeqAccess<'de> for Access<'_, A> {
    type Error = A::Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        self.inner.next_element_seed(Seed {
            inner: seed,
            budget: self.budget,
            depth: self.depth,
        })
    }
    // A malicious count must not reserve storage before element admission.
    fn size_hint(&self) -> Option<usize> {
        None
    }
}
impl<'de, A: MapAccess<'de>> MapAccess<'de> for Access<'_, A> {
    type Error = A::Error;
    fn next_key_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        self.inner.next_key_seed(Seed {
            inner: seed,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn next_value_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        self.inner.next_value_seed(Seed {
            inner: seed,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn size_hint(&self) -> Option<usize> {
        None
    }
}
impl<'a, 'de, A: EnumAccess<'de>> EnumAccess<'de> for Access<'a, A> {
    type Error = A::Error;
    type Variant = Access<'a, A::Variant>;
    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let (value, inner) = self.inner.variant_seed(Seed {
            inner: seed,
            budget: self.budget,
            depth: self.depth,
        })?;
        Ok((
            value,
            Access {
                inner,
                budget: self.budget,
                depth: self.depth,
            },
        ))
    }
}
impl<'de, A: VariantAccess<'de>> VariantAccess<'de> for Access<'_, A> {
    type Error = A::Error;
    fn unit_variant(self) -> Result<(), Self::Error> {
        self.inner.unit_variant()
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        self.inner.newtype_variant_seed(Seed {
            inner: seed,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn tuple_variant<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.inner.tuple_variant(
            len,
            CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth,
            },
        )
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.inner.struct_variant(
            fields,
            CheckedVisitor {
                inner: visitor,
                budget: self.budget,
                depth: self.depth,
            },
        )
    }
}
