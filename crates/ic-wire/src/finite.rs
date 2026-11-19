//! Explicit finite-float contract over the sole Serde/Postcard codec.
use crate::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! finite {
    ($name:ident, $primitive:ty) => {
        /// Finite IEEE-754 value. Preserves signed zero and subnormal bits.
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
        pub struct $name($primitive);
        impl $name {
            /// Admit a primitive without normalization.
            /// # Errors
            /// Returns `Error::Malformed` for NaN or infinity.
            pub fn new(value: $primitive) -> Result<Self, Error> {
                if value.is_finite() {
                    Ok(Self(value))
                } else {
                    Err(Error::Malformed)
                }
            }

            /// Return the exact admitted primitive.
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
                    .map_err(|_| serde::de::Error::custom("non-finite float"))
            }
        }
    };
}
finite!(FiniteF32, f32);
finite!(FiniteF64, f64);
