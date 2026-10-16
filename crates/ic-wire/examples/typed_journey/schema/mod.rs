//! Experimental scalar schema declaration. Its field list is the wire order.
//! This generates ordinary Rust fields and Serde tuple implementations together;
//! reordering this schema is a contract change, not a Rust-only refactor.
macro_rules! scalar {
    (u64) => {};
    (u32) => {};
    (bool) => {};
    ($other:ident) => {
        compile_error!("wire_record supports only u64, u32 and bool fields");
    };
}

macro_rules! wire_record {
    (struct $name:ident { $($field:ident: $ty:ident),+ $(,)? }) => {
        $(scalar!($ty);)*
        #[derive(Clone, Debug, PartialEq)]
        struct $name { $($field: $ty),+ }
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serde::Serialize::serialize(&($(self.$field,)+), serializer)
            }
        }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let ($($field,)+): ($($ty,)+) = serde::Deserialize::deserialize(deserializer)?;
                Ok(Self { $($field),+ })
            }
        }
    };
}
