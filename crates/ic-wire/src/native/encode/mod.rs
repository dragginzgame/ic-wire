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
        serialize_i32: i32,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u128: u128,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_char: char,
    }
    // The private adapter only wraps Postcard serializers, including sizing and
    // canonical comparison. Previously i16 used its temporary varint buffer;
    // emit the same zigzag varint through a prefix-free tuple of raw u8 values.
    // Admission still charges the original i16, not the implementation
    // bytes. Payloads (-64 -> 7f, 64 -> 80 01, 32767 -> fe ff 03), canonical
    // acceptance, limits, hashes and retry bytes are unchanged; no migration.
    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        let value = ((value << 1) ^ (value >> 15)) as u16;
        if value < 128 {
            return self.inner.serialize_u8(value as u8);
        }
        let mut tuple = self
            .inner
            .serialize_tuple(if value < 16384 { 2 } else { 3 })?;
        ser::SerializeTuple::serialize_element(&mut tuple, &((value as u8) | 128))?;
        if value < 16384 {
            ser::SerializeTuple::serialize_element(&mut tuple, &((value >> 7) as u8))?;
        } else {
            ser::SerializeTuple::serialize_element(&mut tuple, &(((value >> 7) as u8) | 128))?;
            ser::SerializeTuple::serialize_element(&mut tuple, &((value >> 14) as u8))?;
        }
        ser::SerializeTuple::end(tuple)
    }
    // Previously i64 used Postcard's separate signed varint writer. Apply its
    // identical zigzag mapping, then reuse our u64 writer and one-byte shortcut.
    // Original signed-type admission is already done; bytes and canonical
    // acceptance are unchanged (-64 -> 7f, 64 -> 80 01). No consumer migration
    // or retry/hash change. i8 deliberately retains its raw-byte representation.
    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(((value << 1) ^ (value >> 63)) as u64)
    }
    // Postcard encodes u16/u32 values below 128 identically to u8. As with
    // u64 below, use its byte writer while retaining the original value's
    // admission charges. Bytes, canonical acceptance and retries are unchanged;
    // consumers need no migration (127 -> 7f, 128 -> 80 01 for both types).
    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        if value < 128 {
            self.inner.serialize_u8(value as u8)
        } else {
            self.inner.serialize_u16(value)
        }
    }
    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        if value < 128 {
            self.inner.serialize_u8(value as u8)
        } else {
            self.inner.serialize_u32(value)
        }
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
