//! Explicit Serde field adapters for domain containers using [`crate::Packed`] bytes.
//!
//! Select with `#[serde(with = "ic_wire::serde_packed::nonzero_i64")]` or
//! `#[serde(with = "ic_wire::serde_packed::heap_u64")]`. Encoding borrows existing
//! storage on little-endian hosts; decoding remains owned. Wire's metadata derive
//! recognizes these exact paths (also with a leading `::`).
//! [`bools`] provides the same explicit selection for `Vec<bool>`, with checked
//! byte-buffer decoding and `boolean[]` browser values on every endian target.
//! [`finite_f32`] and [`finite_f64`] explicitly encode primitive float vectors
//! as finite little-endian words, with typed-array browser values. They reject
//! NaNs/infinities before decode allocation and preserve signed zero/subnormals.
//! A singleton f32 prefix changes from `01` to `04`; f64 changes `01` to `08`.
//! Preserve consumer field bounds and regenerate clients before adoption.
//!
//! Adoption changes ordinary list bytes and traversal admission: `[-1]` as
//! `Vec<NonZeroI64>` changes from `01 01` to `08 ff ff ff ff ff ff ff ff`;
//! heap `[1]` changes from `01 01` to `08 01 00 00 00 00 00 00 00`.
//! Bool list bytes stay identical (`02 00 01` for `[false, true]`), while their
//! traversal admission changes to one buffer. Existing lists are never rewritten.
//! Coordinate contract identities and consumers, regenerate clients and requalify
//! limits. Existing ordinary lists and `Packed` bytes/acceptance are unchanged.
//! Field adapters can charge less encoding allowance than an equivalent `Packed`
//! wrapper because Serde passes a reference-sized adapter to the parent serializer.
//! Neither representation guarantees canonical heap order: retain original bytes
//! when retries or signatures depend on them.

pub mod bools;
mod finite;
pub mod finite_f32;
pub mod finite_f64;
pub mod heap_u64;
pub mod nonzero_i64;
