//! Bounded Serde/Postcard contracts with optional schema generation.
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

/// Experimental sparse extensions and explicit open variants.
#[cfg(feature = "evolution")]
pub mod evolution;

/// Optional schema admission and TypeScript declarations.
#[cfg(feature = "typescript")]
pub mod typescript;

pub use decode::decode;
pub use encode::{encode, encode_into, encode_to_slice, encoded_len};
pub use error::{Error, LimitKind};
pub use fields::{Bytes, BytesRef, List, Text, TextRef};
pub use finite::{FiniteF32, FiniteF64};
pub use framing::{Direction, Encoding, method_id, validate_request_header};
#[cfg(feature = "evolution-derive")]
pub use ic_wire_derive::Wire;
pub use limits::Limits;
pub use packed::{Packed, PackedElement};

/// Published Serde bulk byte types; ByteArray includes a length prefix, unlike
/// an ordinary `[u8; N]`. Changing field representation requires consumer review.
pub use serde_bytes::{ByteArray, ByteBuf};
