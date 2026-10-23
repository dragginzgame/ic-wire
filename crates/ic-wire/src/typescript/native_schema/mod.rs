//! Host-only projection of ordinary native Serde layouts.
use super::GenerationError;
use alloc::{boxed::Box, string::String, vec::Vec};
use serde_json::{Value, json};

/// Metadata for a native Postcard serialization layout, not a runtime resource bound.
///
/// Derive this only alongside ordinary Serde derives. A procedural macro cannot
/// inspect a separately implemented serializer. Manual implementations assert an
/// exact match to both Serialize and Deserialize, including domain validation;
/// qualify them with Rust/TypeScript byte vectors. Never infer a custom layout
/// from its Rust fields. Child metadata must use [`SchemaContext::describe`].
pub trait Wire: serde::Serialize {
    /// Describe the serialized shape using the existing registry metadata grammar.
    fn native_schema(context: &mut SchemaContext) -> Result<Value, GenerationError>;
}

mod byte_field {
    pub trait Sealed {}
    impl Sealed for alloc::vec::Vec<u8> {}
    impl Sealed for [u8] {}
    impl Sealed for serde_bytes::ByteBuf {}
    impl Sealed for serde_bytes::Bytes {}
    impl<T: Sealed + ?Sized> Sealed for &T {}
}

/// Qualified inputs to the standard `serde_bytes` field adapter.
/// Sealed to prevent assuming arbitrary custom adapters use a byte-buffer layout.
///
/// ```compile_fail
/// fn require_bytes<T: ic_wire::typescript::SerdeBytesField>() {}
/// require_bytes::<Vec<u16>>();
/// ```
#[doc(hidden)]
pub trait SerdeBytesField: byte_field::Sealed {}
impl<T: byte_field::Sealed + ?Sized> SerdeBytesField for T {}

/// Bounded traversal used by derived and explicitly implemented metadata.
/// Constructed by [`native_schema`]; it does not infer codec resource allowances.
pub struct SchemaContext {
    depth: usize,
    remaining: usize,
}
impl SchemaContext {
    /// Describe a qualified `serde_bytes` field while retaining traversal checks.
    #[doc(hidden)]
    pub fn describe_bytes<T: SerdeBytesField + ?Sized>(
        &mut self,
    ) -> Result<Value, GenerationError> {
        self.describe::<serde_bytes::Bytes>()
    }
    /// Describe a child, rejecting recursive or excessively large inline schemas.
    pub fn describe<T: Wire + ?Sized>(&mut self) -> Result<Value, GenerationError> {
        if self.depth >= 64 || self.remaining == 0 {
            return Err(GenerationError::Complexity);
        }
        self.depth += 1;
        self.remaining -= 1;
        let result = T::native_schema(self);
        self.depth -= 1;
        result
    }
}

/// Describe a concrete native Serde type for [`super::generate_registry`].
///
/// Generic parameters must be instantiated (for example `Page<User>`). Schemas
/// are inlined, with a 64-level/65,536-node traversal ceiling; recursive types
/// return Complexity. Method identity and runtime budgets remain caller-owned.
pub fn native_schema<T: Wire + ?Sized>() -> Result<Value, GenerationError> {
    SchemaContext {
        depth: 0,
        remaining: 65536,
    }
    .describe::<T>()
}

macro_rules! scalar {
    ($($ty:ty => $node:expr),* $(,)?) => {$(
        impl Wire for $ty {
            fn native_schema(_: &mut SchemaContext) -> Result<Value, GenerationError> {
                Ok($node)
            }
        }
    )*};
}
scalar! {
    () => json!({"k":"unit"}), bool => json!({"k":"bool"}),
    str => json!({"k":"text"}), String => json!({"k":"text"}),
    // Unlike Vec<u8>'s list metadata, byte-oriented fields project to Uint8Array.
    // Postcard bytes stay length + raw bytes (03 00 7f ff for [0,127,255]);
    // traversal budgets change from per-element to bulk. Regenerate client
    // bindings and qualify limits when changing a consumer field representation.
    serde_bytes::ByteBuf => json!({"k":"bytes"}),
    serde_bytes::Bytes => json!({"k":"bytes"}),
    u8 => json!({"k":"u","bits":8}), u16 => json!({"k":"u","bits":16}),
    u32 => json!({"k":"u","bits":32}), u64 => json!({"k":"u","bits":64}),
    u128 => json!({"k":"u","bits":128}),
    i8 => json!({"k":"i","bits":8}), i16 => json!({"k":"i","bits":16}),
    i32 => json!({"k":"i","bits":32}), i64 => json!({"k":"i","bits":64}),
    i128 => json!({"k":"i","bits":128}),
    crate::FiniteF32 => json!({"k":"float","bits":32}),
    crate::FiniteF64 => json!({"k":"float","bits":64}),
}
// Deliberately omit pointer-sized integers and raw floats: their platform/range
// contracts cannot be faithfully inferred from the TS runtime.
impl<T: Wire> Wire for Vec<T> {
    fn native_schema(c: &mut SchemaContext) -> Result<Value, GenerationError> {
        <[T] as Wire>::native_schema(c)
    }
}
// Serde sequences share a length-prefixed layout regardless of Vec/slice custody.
// Forward directly (not through describe) so existing Vec metadata traversal
// depth/node accounting and generated clients remain unchanged.
impl<T: Wire> Wire for [T] {
    fn native_schema(c: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"list","item":c.describe::<T>()?}))
    }
}
impl<T: Wire> Wire for Option<T> {
    fn native_schema(c: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"option","item":c.describe::<T>()?}))
    }
}
impl<T: Wire, E: Wire> Wire for Result<T, E> {
    fn native_schema(c: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"result","ok":c.describe::<T>()?,"err":c.describe::<E>()?}))
    }
}
impl<T: Wire + ?Sized> Wire for Box<T> {
    fn native_schema(c: &mut SchemaContext) -> Result<Value, GenerationError> {
        c.describe::<T>()
    }
}
impl<T: Wire + ?Sized> Wire for &T {
    fn native_schema(c: &mut SchemaContext) -> Result<Value, GenerationError> {
        c.describe::<T>()
    }
}
impl<T: Wire, const N: usize> Wire for [T; N]
where
    [T; N]: serde::Serialize,
{
    fn native_schema(c: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"array","max":N,"item":c.describe::<T>()?}))
    }
}
macro_rules! tuple {
    ($($t:ident),+) => {
        impl<$($t: Wire),+> Wire for ($($t,)+) {
            fn native_schema(c: &mut SchemaContext) -> Result<Value, GenerationError> {
                Ok(json!({"k":"tuple","items":[$(c.describe::<$t>()?),+]}))
            }
        }
    };
}
tuple!(A);
tuple!(A, B);
tuple!(A, B, C);
tuple!(A, B, C, D);

macro_rules! bounded_scalar {
    ($ty:ty, $kind:literal) => {
        impl<const N: usize> Wire for $ty {
            fn native_schema(_: &mut SchemaContext) -> Result<Value, GenerationError> {
                Ok(json!({"k": $kind, "max": N}))
            }
        }
    };
}
bounded_scalar!(crate::PackedI16<N>, "packed-i16");
bounded_scalar!(crate::Bytes<N>, "bytes");
bounded_scalar!(crate::BytesRef<'_, N>, "bytes");
bounded_scalar!(crate::Text<N>, "text");
bounded_scalar!(crate::TextRef<'_, N>, "text");
impl<T: Wire, const N: usize> Wire for crate::List<T, N> {
    fn native_schema(context: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"list", "max":N, "item":context.describe::<T>()?}))
    }
}
