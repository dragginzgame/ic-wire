use serde::{Serialize, de::DeserializeOwned};

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
pub trait Wire: Serialize + DeserializeOwned {
    /// Static resource authority for this value.
    const SHAPE: Shape;
}

/// Consumer-selected allowances, checked against the schema before decoding.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum complete envelope bytes, including its 11-byte header.
    pub bytes: usize,
    /// Maximum value heap plus codec scratch (excludes caller input).
    pub allocation: usize,
    /// Maximum visited value nodes, including unit elements.
    pub nodes: usize,
    /// Maximum nesting depth.
    pub depth: usize,
}
macro_rules! scalars {
    ($($ty:ty => $bytes:expr),* $(,)?) => { $(
        impl Wire for $ty { const SHAPE: Shape = Shape::scalar($bytes); }
    )* };
}
scalars! { () => 0, bool => 1, u8 => 1, i8 => 1, u16 => 3, i16 => 3,
u32 => 5, i32 => 5, u64 => 10, i64 => 10, u128 => 19, i128 => 19 }
impl<T: Wire> Wire for Option<T> {
    const SHAPE: Shape = Shape::choice(1, &[T::SHAPE]);
}
impl<T: Wire, E: Wire> Wire for Result<T, E> {
    const SHAPE: Shape = Shape::choice(1, &[T::SHAPE, E::SHAPE]);
}

impl<T: Wire, const N: usize> Wire for [T; N]
where
    [T; N]: Serialize + DeserializeOwned,
{
    const SHAPE: Shape = Shape {
        bytes: T::SHAPE.bytes.saturating_mul(N),
        heap: T::SHAPE.heap.saturating_mul(N),
        nodes: 1_usize.saturating_add(T::SHAPE.nodes.saturating_mul(N)),
        depth: T::SHAPE.depth.saturating_add(1),
    };
}
