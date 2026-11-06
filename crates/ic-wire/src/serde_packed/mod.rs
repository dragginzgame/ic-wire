//! Explicit Serde field adapters for domain containers using [`crate::Packed`] bytes.
//!
//! Select with `#[serde(with = "ic_wire::serde_packed::nonzero_i64")]` or
//! `#[serde(with = "ic_wire::serde_packed::heap_u64")]`. Encoding borrows existing
//! storage on little-endian hosts; decoding remains owned. Wire's metadata derive
//! recognizes these exact paths (also with a leading `::`).
//!
//! Adoption changes ordinary list bytes and traversal admission: `[-1]` as
//! `Vec<NonZeroI64>` changes from `01 01` to `08 ff ff ff ff ff ff ff ff`;
//! heap `[1]` changes from `01 01` to `08 01 00 00 00 00 00 00 00`.
//! Coordinate contract identities and consumers, regenerate clients and requalify
//! limits. Existing ordinary lists and `Packed` bytes/acceptance are unchanged.
//! Field adapters can charge less encoding allowance than an equivalent `Packed`
//! wrapper because Serde passes a reference-sized adapter to the parent serializer.
//! Neither representation guarantees canonical heap order: retain original bytes
//! when retries or signatures depend on them.

pub mod heap_u64;
pub mod nonzero_i64;
