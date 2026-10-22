use alloc::boxed::Box;
use serde::Serialize;

/// Worst-case bounds for one finite value.
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    /// Maximum canonical Postcard payload bytes.
    pub bytes: usize,
    /// Maximum requested heap bytes owned by the decoded value.
    pub heap: usize,
    /// Maximum visited value nodes, including zero-byte values.
    pub nodes: usize,
    /// Maximum nested value depth.
    pub depth: usize,
}
impl Shape {
    /// A leaf with no allocation.
    pub const fn scalar(bytes: usize) -> Self {
        Self {
            bytes,
            heap: 0,
            nodes: 1,
            depth: 1,
        }
    }
    /// A record with these children. Overflow saturates and fails admission.
    pub const fn record(children: &[Self]) -> Self {
        let mut out = Self::scalar(0);
        let mut i = 0;
        while i < children.len() {
            let child = children[i];
            out.bytes = out.bytes.saturating_add(child.bytes);
            out.heap = out.heap.saturating_add(child.heap);
            out.nodes = out.nodes.saturating_add(child.nodes);
            let depth = child.depth.saturating_add(1);
            if depth > out.depth {
                out.depth = depth;
            }
            i += 1;
        }
        out
    }
    /// A tagged choice, using the largest bound in each dimension.
    pub const fn choice(tag_bytes: usize, alternatives: &[Self]) -> Self {
        let mut out = Self::scalar(0);
        let mut i = 0;
        while i < alternatives.len() {
            let child = alternatives[i];
            if child.bytes > out.bytes {
                out.bytes = child.bytes;
            }
            if child.heap > out.heap {
                out.heap = child.heap;
            }
            if child.nodes > out.nodes {
                out.nodes = child.nodes;
            }
            if child.depth > out.depth {
                out.depth = child.depth;
            }
            i += 1;
        }
        out.bytes = out.bytes.saturating_add(tag_bytes);
        out.nodes = out.nodes.saturating_add(1);
        out.depth = out.depth.saturating_add(1);
        out
    }
}

/// Finite values. Prefer the derive with explicit field identities.
///
/// Manual implementations must provide honest bounds, reject before excessive
/// allocation/work, and serialize deterministically. Unbounded Serde types have
/// no implementation. Recursive types are unsupported.
/// Decoding additionally requires `Deserialize` for the input lifetime; a Wire
/// value may borrow input or support encoding only.
pub trait Wire: Serialize {
    /// Internal whole-type guarantee of unchanged values and decoding boundaries.
    /// Reserved for supplied types and generated code; manual implementations
    /// must retain this default and the default `__canonical` method.
    #[doc(hidden)]
    const __CANONICAL_FIELDS: bool = false;

    /// Internal post-decode verification. Custom implementations retain exact comparison.
    #[doc(hidden)]
    fn __canonical(
        &self,
        canonical: &mut crate::__private::Canonical<'_>,
    ) -> Result<(), crate::Error> {
        canonical.compare(self)
    }

    /// Static resource authority for this value.
    const SHAPE: Shape;
    /// Static projection metadata; custom implementations must opt in honestly.
    const SCHEMA: crate::schema::Schema = crate::schema::Schema::Opaque;
}

/// Consumer-selected allowances for finite schema or explicit native admission.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum encoding bytes; framed APIs include the 11-byte header.
    pub bytes: usize,
    /// Maximum value heap plus codec scratch (excludes caller input).
    pub allocation: usize,
    /// Maximum visited value nodes, including unit elements.
    pub nodes: usize,
    /// Maximum nesting depth.
    pub depth: usize,
}
macro_rules! scalars {
    ($($ty:ty => ($bytes:expr, $schema:expr)),* $(,)?) => { $(
        impl Wire for $ty {
            const __CANONICAL_FIELDS: bool = true;
            const SHAPE: Shape = Shape::scalar($bytes);
            const SCHEMA: crate::schema::Schema = $schema;
        }
    )* };
}
use crate::schema::Schema;
scalars! { () => (0, Schema::Unit), bool => (1, Schema::Bool),
u8 => (1, Schema::Unsigned(8)), i8 => (1, Schema::Signed(8)),
u16 => (3, Schema::Unsigned(16)), i16 => (3, Schema::Signed(16)),
u32 => (5, Schema::Unsigned(32)), i32 => (5, Schema::Signed(32)),
u64 => (10, Schema::Unsigned(64)), i64 => (10, Schema::Signed(64)),
u128 => (19, Schema::Unsigned(128)), i128 => (19, Schema::Signed(128)) }
impl<T: Wire> Wire for Option<T> {
    const __CANONICAL_FIELDS: bool = T::__CANONICAL_FIELDS;
    const SHAPE: Shape = Shape::choice(1, &[T::SHAPE]);
    const SCHEMA: Schema = Schema::Option(&T::SCHEMA);
}
impl<T: Wire, E: Wire> Wire for Result<T, E> {
    const __CANONICAL_FIELDS: bool = T::__CANONICAL_FIELDS && E::__CANONICAL_FIELDS;
    const SHAPE: Shape = Shape::choice(1, &[T::SHAPE, E::SHAPE]);
    const SCHEMA: Schema = Schema::Result(&T::SCHEMA, &E::SCHEMA);
}

// Serde boxes are transparent: only owned heap storage changes. Saturation
// preserves the existing fail-closed admission rule, including nested boxes.
impl<T: Wire> Wire for Box<T> {
    const __CANONICAL_FIELDS: bool = T::__CANONICAL_FIELDS;
    const SHAPE: Shape = Shape {
        heap: core::mem::size_of::<T>().saturating_add(T::SHAPE.heap),
        ..T::SHAPE
    };
    const SCHEMA: Schema = T::SCHEMA;
}

impl<T: Wire, const N: usize> Wire for [T; N]
where
    [T; N]: Serialize,
{
    const __CANONICAL_FIELDS: bool = T::__CANONICAL_FIELDS;
    const SCHEMA: Schema = Schema::Array(&T::SCHEMA, N);
    const SHAPE: Shape = Shape {
        bytes: T::SHAPE.bytes.saturating_mul(N),
        heap: T::SHAPE.heap.saturating_mul(N),
        nodes: 1_usize.saturating_add(T::SHAPE.nodes.saturating_mul(N)),
        depth: T::SHAPE.depth.saturating_add(1),
    };
}
