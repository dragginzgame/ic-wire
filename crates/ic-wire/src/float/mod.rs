use crate::{Error, Shape, Wire, schema::Schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

macro_rules! finite {
    ($name:ident, $primitive:ty, $bits:literal) => {
        /// A finite IEEE-754 value encoded in Postcard's native little-endian form.
        ///
        /// NaN and infinity are rejected. All finite bits, including negative zero
        /// and subnormals, are preserved. Consumer normalization remains separate.
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
        pub struct $name($primitive);
        impl $name {
            /// Admit a finite value; non-finite input returns Error::Malformed.
            pub fn new(value: $primitive) -> Result<Self, Error> {
                if value.is_finite() {
                    Ok(Self(value))
                } else {
                    Err(Error::Malformed)
                }
            }
            /// Recover the exact admitted primitive value.
            pub fn get(self) -> $primitive {
                self.0
            }
        }
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Self::new(<$primitive>::deserialize(deserializer)?)
                    .map_err(|_| de::Error::custom("non-finite float"))
            }
        }
        impl Wire for $name {
            const __CANONICAL_FIELDS: bool = true;
            const SHAPE: Shape = Shape::scalar($bits / 8);
            const SCHEMA: Schema = Schema::Float($bits);
        }
    };
}
finite!(FiniteF32, f32, 32);
finite!(FiniteF64, f64, 64);
