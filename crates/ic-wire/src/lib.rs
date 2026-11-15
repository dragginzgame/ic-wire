//! Portable IC Wire contract evaluation.
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
mod framing;
mod limits;
mod meter;
mod packed;

/// Optional host-only schema admission and TypeScript declarations.
#[cfg(all(feature = "typescript", not(target_family = "wasm")))]
pub mod typescript;

pub use decode::decode;
pub use encode::{encode, encode_into, encode_to_slice, encoded_len};
pub use error::Error;
pub use fields::{Bytes, BytesRef, List, Text, TextRef};
pub use framing::{Direction, Encoding, method_id, validate_request_header};
pub use limits::Limits;
pub use packed::PackedI16;
