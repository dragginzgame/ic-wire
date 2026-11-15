//! Bounded inline metadata production for the existing JSON admission boundary.
use super::{GenerationError, SchemaLimits, ValidatedRegistry};
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};
use serde_json::{Value, json};

/// A trusted description of a type's Serde representation.
///
/// This does not implement Serialize/Deserialize. Manual implementations must
/// independently qualify that their projection agrees with their Serde code.
pub trait Wire {
    /// Produce metadata using `context.describe` for every nested Rust type.
    ///
    /// # Errors
    /// Return Schema for unsupported layouts or Complexity for exhausted work.
    fn schema(context: &mut SchemaContext) -> Result<Value, GenerationError>;
}

/// Construction allowance for inline Rust metadata, separate from graph admission.
///
/// Created by `ValidatedRegistry::from_type`. Calls and active type depth are
/// bounded; recursion rejects rather than infinitely expanding. Manual projection
/// code is trusted and may allocate/work outside this accounting, like custom
/// Serde code. Final JSON admission independently checks names and exact nodes.
#[derive(Debug)]
pub struct SchemaContext {
    remaining: usize,
    depth: usize,
    active: Vec<&'static str>,
    failed: bool,
}

impl SchemaContext {
    /// Describe a nested type without bypassing construction admission.
    ///
    /// # Errors
    /// Schema rejects recursive automatic projection or a previously failed
    /// context; Complexity rejects exhausted construction calls or depth.
    pub fn describe<T: Wire + ?Sized>(&mut self) -> Result<Value, GenerationError> {
        if self.failed {
            return Err(GenerationError::Schema);
        }
        self.failed = true;
        let name = core::any::type_name::<T>();
        if self.active.contains(&name) {
            return Err(GenerationError::Schema);
        }
        if self.active.len() > self.depth {
            return Err(GenerationError::Complexity);
        }
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(GenerationError::Complexity)?;
        self.active.push(name);
        self.failed = false;
        let result = T::schema(self);
        self.active.pop();
        // A manual projection cannot hide a failed nested registration.
        if self.failed && result.is_ok() {
            return Err(GenerationError::Schema);
        }
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

impl ValidatedRegistry {
    /// Project one Rust root and admit its inline metadata into the shared graph.
    ///
    /// Nested declarations are inline; no generated-name algorithm is substituted
    /// for the reference named collector. Recursive automatic projection rejects;
    /// use explicit named metadata for recursive graphs until collection qualifies.
    ///
    /// # Errors
    /// Returns Schema for invalid/recursive projections and Complexity for
    /// exceeded construction or graph limits. No partial registry is returned.
    pub fn from_type<T: Wire + ?Sized>(
        root: &str,
        limits: SchemaLimits,
    ) -> Result<Self, GenerationError> {
        super::identifier(root)?;
        if limits.depth > 64 || root.len() > limits.names_bytes {
            return Err(GenerationError::Complexity);
        }
        let mut context = SchemaContext {
            remaining: limits.nodes,
            depth: limits.depth,
            active: Vec::new(),
            failed: false,
        };
        let schema = context.describe::<T>()?;
        Self::from_metadata(&BTreeMap::from([(String::from(root), schema)]), limits)
    }
}

macro_rules! scalar {
    ($($ty:ty => $value:expr),* $(,)?) => { $(
        impl Wire for $ty {
            fn schema(_: &mut SchemaContext) -> Result<Value, GenerationError> { Ok($value) }
        }
    )* };
}
scalar! {
    () => json!({"k":"unit"}), bool => json!({"k":"bool"}),
    u8 => json!({"k":"u","bits":8}), u16 => json!({"k":"u","bits":16}),
    u32 => json!({"k":"u","bits":32}), u64 => json!({"k":"u","bits":64}), u128 => json!({"k":"u","bits":128}),
    i8 => json!({"k":"i","bits":8}), i16 => json!({"k":"i","bits":16}),
    i32 => json!({"k":"i","bits":32}), i64 => json!({"k":"i","bits":64}), i128 => json!({"k":"i","bits":128}),
    str => json!({"k":"text"}), String => json!({"k":"text"}),
}
impl<T: Wire> Wire for Option<T> {
    fn schema(context: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"option","item":context.describe::<T>()?}))
    }
}
impl<T: Wire> Wire for Vec<T> {
    fn schema(context: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"list","item":context.describe::<T>()?}))
    }
}
impl<T: Wire + ?Sized> Wire for Box<T> {
    fn schema(context: &mut SchemaContext) -> Result<Value, GenerationError> {
        context.describe::<T>()
    }
}
impl<T: Wire + ?Sized> Wire for &T {
    fn schema(context: &mut SchemaContext) -> Result<Value, GenerationError> {
        context.describe::<T>()
    }
}
impl<T: Wire, const N: usize> Wire for crate::List<T, N> {
    fn schema(context: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"list","max":N,"item":context.describe::<T>()?}))
    }
}
macro_rules! bounded {
    ($($ty:ident => $kind:literal),* $(,)?) => { $(
        impl<const N: usize> Wire for crate::$ty<N> {
            fn schema(_: &mut SchemaContext) -> Result<Value, GenerationError> { Ok(json!({"k":$kind,"max":N})) }
        }
    )* };
}
bounded! { Bytes => "bytes", Text => "text", PackedI16 => "packed-i16" }
impl<const N: usize> Wire for crate::BytesRef<'_, N> {
    fn schema(_: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"bytes","max":N}))
    }
}
impl<const N: usize> Wire for crate::TextRef<'_, N> {
    fn schema(_: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"text","max":N}))
    }
}
