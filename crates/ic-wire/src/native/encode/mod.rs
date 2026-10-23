//! Admission before native payload serialization.
// Adapted from IcyDB native admission; see ../LICENSE-MIT.

use super::Budget;
use serde::{Serialize, Serializer, ser};

pub(super) struct Bounded<'a, T: ?Sized> {
    pub(super) value: &'a T,
    pub(super) budget: &'a Budget,
    pub(super) depth: usize,
}

impl<T: Serialize + ?Sized> Serialize for Bounded<'_, T> {
    #[cfg_attr(target_arch = "wasm32", inline(always))]
    fn serialize<S: Serializer>(&self, inner: S) -> Result<S::Ok, S::Error> {
        if self.depth.saturating_add(2) > self.budget.depth
            // Preserve four node charges and two value-size charges, but admit
            // them together. Refusal is sticky; no serializer runs on failure.
            || !size_of_val(self.value)
                .checked_mul(2)
                .is_some_and(|size| self.budget.enter_many(self.depth, size, 4))
        {
            self.budget.refused.set(true);
            return Err(ser::Error::custom("payload resource allowance exceeded"));
        }
        self.value.serialize(Checked {
            inner,
            budget: self.budget,
            depth: self.depth,
        })
    }
}

struct Checked<'a, S> {
    inner: S,
    budget: &'a Budget,
    depth: usize,
}
struct Compound<'a, S> {
    inner: S,
    budget: &'a Budget,
    depth: usize,
}

// Scalar forwarding needs no additional admission; allocation checks stay explicit.
macro_rules! forward_scalar {
    ($($method:ident: $ty:ty),* $(,)?) => {$(
        fn $method(self, value: $ty) -> Result<Self::Ok, Self::Error> {
            self.inner.$method(value)
        }
    )*};
}

impl<'a, S: Serializer> Serializer for Checked<'a, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = Compound<'a, S::SerializeSeq>;
    type SerializeTuple = Compound<'a, S::SerializeTuple>;
    type SerializeTupleStruct = Compound<'a, S::SerializeTupleStruct>;
    type SerializeTupleVariant = Compound<'a, S::SerializeTupleVariant>;
    type SerializeMap = Compound<'a, S::SerializeMap>;
    type SerializeStruct = Compound<'a, S::SerializeStruct>;
    type SerializeStructVariant = Compound<'a, S::SerializeStructVariant>;
    forward_scalar! {
        serialize_bool: bool,
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u128: u128,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_char: char,
    }
    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        // This private adapter only wraps Postcard serializers (including sizing
        // and canonical comparison). Below 128, u64 and u8 emit the same byte.
        // Avoid the temporary varint buffer without changing value admission,
        // frame bytes, canonicality or retry hashes. No consumer migration.
        if value < 128 {
            self.inner.serialize_u8(value as u8)
        } else {
            self.inner.serialize_u64(value)
        }
    }
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        if !self.budget.bytes(value.len()) {
            return Err(ser::Error::custom("payload allocation allowance exceeded"));
        }
        self.inner.serialize_str(value)
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        if !self.budget.bytes(value.len()) {
            return Err(ser::Error::custom("payload allocation allowance exceeded"));
        }
        self.inner.serialize_bytes(value)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_none()
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit()
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit_struct(name)
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit_variant(name, index, variant)
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_some(&Bounded {
            value,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_newtype_struct(
            name,
            &Bounded {
                value,
                budget: self.budget,
                depth: self.depth + 1,
            },
        )
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_newtype_variant(
            name,
            index,
            variant,
            &Bounded {
                value,
                budget: self.budget,
                depth: self.depth + 1,
            },
        )
    }
    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(Compound {
            inner: self.inner.serialize_seq(len)?,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Compound {
            inner: self.inner.serialize_tuple(len)?,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Compound {
            inner: self.inner.serialize_tuple_struct(name, len)?,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(Compound {
            inner: self
                .inner
                .serialize_tuple_variant(name, index, variant, len)?,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(Compound {
            inner: self.inner.serialize_map(len)?,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(Compound {
            inner: self.inner.serialize_struct(name, len)?,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(Compound {
            inner: self
                .inner
                .serialize_struct_variant(name, index, variant, len)?,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
}
// Each compound keeps its existing depth; child values own their admission.
macro_rules! compound {
    ($trait:ident, $($method:ident $(($key:ident))?),+ $(,)?) => {
        impl<S: ser::$trait> ser::$trait for Compound<'_, S> {
            type Ok = S::Ok;
            type Error = S::Error;
            $(#[cfg_attr(target_arch = "wasm32", inline(always))]
            fn $method<T: Serialize + ?Sized>(
                &mut self,
                $($key: &'static str,)?
                value: &T,
            ) -> Result<(), Self::Error> {
                self.inner.$method(
                    $($key,)?
                    &Bounded {
                        value,
                        budget: self.budget,
                        depth: self.depth,
                    },
                )
            })+
            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.inner.end()
            }
        }
    };
}
compound!(SerializeSeq, serialize_element);
compound!(SerializeTuple, serialize_element);
compound!(SerializeTupleStruct, serialize_field);
compound!(SerializeTupleVariant, serialize_field);
compound!(SerializeStruct, serialize_field(key));
compound!(SerializeStructVariant, serialize_field(key));
compound!(SerializeMap, serialize_key, serialize_value);
