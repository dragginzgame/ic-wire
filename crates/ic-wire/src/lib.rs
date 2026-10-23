//! Bounded Postcard contracts for controlled IC endpoints.
//!
//! Transport, authentication, semantic validation and replay policy belong to
//! consumers. See the runtime guide for maintained wire rules.
#![no_std]
#![doc = include_str!("../README.md")]
extern crate alloc;
mod bounded;
mod contract;
mod float;
pub mod native;
/// Optional host-side TypeScript generation; never required by a canister.
#[cfg(feature = "typescript")]
pub mod typescript;
pub use bounded::{Bytes, BytesRef, List, PackedI16, Text, TextRef};
pub use float::{FiniteF32, FiniteF64};
pub use ic_wire_derive::Wire;

/// Implementation dependencies used by generated contracts.
#[doc(hidden)]
pub mod __private {
    pub use serde;
    #[cfg(feature = "typescript")]
    pub use serde_json;
}

/// Typed codec/admission failure, distinct from transport and application errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Wire, collection or text allowance exceeded.
    Limit,
    /// Allocation failed within an admitted allowance.
    Allocation,
    /// Invalid or truncated Postcard representation.
    Malformed,
    /// Bytes remain after the expected value.
    Trailing,
    /// A valid value used a noncanonical representation.
    Noncanonical,
    /// Envelope marker, method identity or direction differs.
    Identity,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "wire {self:?}")
    }
}
impl core::error::Error for Error {}

/// Consumer-selected allowances for Serde traversal.
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
