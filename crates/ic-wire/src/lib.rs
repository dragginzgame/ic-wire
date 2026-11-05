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
mod packed;
#[cfg(all(feature = "platform-types", not(target_family = "wasm")))]
mod platform_types;
pub use packed::{Packed, PackedElement};
/// Bounded Serde/Postcard encoding, decoding and request/response framing.
pub mod native;
/// Optional TypeScript generation on native hosts.
/// Forwarding the feature on Wasm retains only the runtime codec.
#[cfg(all(feature = "typescript", not(target_family = "wasm")))]
pub mod typescript;
pub use bounded::{Bytes, BytesRef, List, PackedI16, Text, TextRef};
pub use float::{FiniteF32, FiniteF64};
#[cfg(feature = "typescript")]
pub use ic_wire_derive::Wire;

/// Owned binary data using Serde's bulk byte-buffer representation.
///
/// This is the same type as `serde_bytes::ByteBuf`, available without a separate
/// consumer dependency. Use [`Bytes<N>`] when the field needs a byte limit.
/// Native [`Limits`] still apply; TypeScript metadata projects to `Uint8Array`.
/// Compared with `Vec<u8>`, bytes are identical but admission is bulk rather
/// than per-element. Existing ByteBuf bytes, limits and primitive checks are
/// unchanged by this re-export; no migration is needed from serde_bytes::ByteBuf.
pub use serde_bytes::ByteBuf;

/// Fixed-width binary data using Serde's bulk byte representation.
///
/// This re-exports `serde_bytes::ByteArray<N>`; no consumer dependency is needed.
/// TypeScript projects it to `Uint8Array` with length exactly `N`.
/// Unlike `[u8; N]`, its Postcard bytes include a length prefix: `[0, 255]`
/// is `00 ff` as an array and `02 00 ff` as `ByteArray<2>`. Adopting this type
/// changes the contract and bulk admission; coordinate consumers, regenerate
/// clients and qualify limits. Existing array layouts remain unchanged.
pub use serde_bytes::ByteArray;

/// Implementation dependencies used by generated contracts.
#[doc(hidden)]
pub mod __private {
    pub use serde;
    #[cfg(all(feature = "typescript", not(target_family = "wasm")))]
    pub use serde_json;
}

/// Typed codec/admission failure, distinct from transport and application errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Wire, collection, text or caller-output allowance exceeded.
    Limit,
    /// Allocation failed within an admitted allowance.
    Allocation,
    /// Invalid or truncated Postcard representation.
    Malformed,
    /// Bytes remain after the expected value.
    Trailing,
    /// Envelope marker, method identity or direction differs.
    Identity,
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Display must survive builds that strip derived Debug implementations.
        // Text changes from e.g. "wire Malformed" to "wire malformed"; enum
        // meanings, protocol bytes and admission rules remain unchanged.
        f.write_str(match self {
            Self::Limit => "wire limit",
            Self::Allocation => "wire allocation",
            Self::Malformed => "wire malformed",
            Self::Trailing => "wire trailing",
            Self::Identity => "wire identity",
        })
    }
}
impl core::error::Error for Error {}

/// Consumer-selected allowances for Serde traversal.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum encoding bytes; framed APIs include the 11-byte header.
    pub bytes: usize,
    /// Conservative traversal allowance in bytes, not measured heap usage.
    /// Reserves the entire [`Self::bytes`] allowance, then charges eight times
    /// visited value sizes and text/byte lengths. Excludes caller input.
    /// Encoding and decoding can require different allowances; see the crate guide.
    pub allocation: usize,
    /// Maximum visited value nodes, including unit elements.
    pub nodes: usize,
    /// Maximum nesting depth.
    pub depth: usize,
}
