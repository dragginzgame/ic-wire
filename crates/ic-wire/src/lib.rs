//! Bounded Serde/Postcard contracts with optional host schema generation.
//!
//! Bounded encoding and decoding, method identity and request-header
//! admission. Header admission is
//! neither authentication nor payload/schema/resource validation.
#![no_std]
extern crate alloc;

mod decode;
mod encode;
mod error;
mod fields;
mod finite;
mod framing;
mod limits;
mod meter;
mod packed;
pub mod serde_packed;

/// Optional host-only schema admission and TypeScript declarations.
#[cfg(all(feature = "typescript", not(target_family = "wasm")))]
pub mod typescript;

/// Schema derive for DTOs shared with host tooling.
#[cfg(all(feature = "derive", target_family = "wasm"))]
pub mod typescript {
    pub use ic_wire_derive::Wire;
}

pub use decode::decode;
pub use encode::{encode, encode_into, encode_to_slice, encoded_len};
pub use error::Error;
pub use fields::{Bytes, BytesRef, List, Text, TextRef};
pub use finite::{FiniteF32, FiniteF64};
pub use framing::{Direction, Encoding, method_id, validate_request_header};
pub use limits::Limits;
pub use packed::{Packed, PackedElement};

/// Published Serde bulk byte types; ByteArray includes a length prefix, unlike
/// an ordinary `[u8; N]`. Changing field representation requires consumer review.
pub use serde_bytes::{ByteArray, ByteBuf};
