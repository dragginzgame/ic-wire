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
    fn serialize<S: Serializer>(&self, inner: S) -> Result<S::Ok, S::Error> {
        if self.depth.saturating_add(2) > self.budget.depth
            || !self.budget.enter(self.depth, size_of_val(self.value))
            || !self.budget.enter(self.depth, size_of_val(self.value))
            || !self.budget.enter(self.depth, 0)
            || !self.budget.enter(self.depth, 0)
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
    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_bool(value)
    }
    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i8(value)
    }
    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i16(value)
    }
    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i32(value)
    }
    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i64(value)
    }
    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i128(value)
    }
    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u8(value)
    }
    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u16(value)
    }
    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u32(value)
    }
    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u64(value)
    }
    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u128(value)
    }
    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_f32(value)
    }
    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_f64(value)
    }
    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_char(value)
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
impl<S: ser::SerializeSeq> ser::SerializeSeq for Compound<'_, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_element(&Bounded {
            value,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
impl<S: ser::SerializeTuple> ser::SerializeTuple for Compound<'_, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_element(&Bounded {
            value,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
impl<S: ser::SerializeTupleStruct> ser::SerializeTupleStruct for Compound<'_, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_field(&Bounded {
            value,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
impl<S: ser::SerializeTupleVariant> ser::SerializeTupleVariant for Compound<'_, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_field(&Bounded {
            value,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
impl<S: ser::SerializeStruct> ser::SerializeStruct for Compound<'_, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.inner.serialize_field(
            key,
            &Bounded {
                value,
                budget: self.budget,
                depth: self.depth,
            },
        )
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
impl<S: ser::SerializeStructVariant> ser::SerializeStructVariant for Compound<'_, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.inner.serialize_field(
            key,
            &Bounded {
                value,
                budget: self.budget,
                depth: self.depth,
            },
        )
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
impl<S: ser::SerializeMap> ser::SerializeMap for Compound<'_, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_key(&Bounded {
            value,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_value(&Bounded {
            value,
            budget: self.budget,
            depth: self.depth,
        })
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
