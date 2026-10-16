//! Static metadata for projections of the maintained wire contract.

/// A field in ascending wire-ID order.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    /// Rust field name, without a raw-identifier prefix.
    pub name: &'static str,
    /// Explicit field identity; not a transmitted tag.
    pub id: u32,
    /// Field encoding and bounds.
    pub value: &'static Schema,
}
/// One explicitly tagged enum alternative.
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    /// Source variant name.
    pub name: &'static str,
    /// Transmitted Postcard u32 tag.
    pub id: u32,
    /// Unit or the single payload's encoding.
    pub value: &'static Schema,
}
/// Finite type metadata, emitted by Wire derives and bounded built-ins.
/// Manual Wire implementations default to Opaque and cannot generate bindings.
#[derive(Clone, Copy, Debug)]
pub enum Schema {
    /// No trustworthy generated projection is available.
    Opaque,
    /// Zero-byte unit.
    Unit,
    /// A strict boolean byte.
    Bool,
    /// Unsigned integer width in bits.
    Unsigned(u8),
    /// Signed integer width in bits (i8 is a raw two's-complement byte).
    Signed(u8),
    /// Bytes with a maximum length.
    Bytes(usize),
    /// UTF-8 text with a maximum byte length.
    Text(usize),
    /// Element schema and maximum count.
    List(&'static Schema, usize),
    /// Element schema and exact count, without a length prefix.
    Array(&'static Schema, usize),
    /// Optional value, with explicit None/Some tags.
    Option(&'static Schema),
    /// Ok and Err payloads respectively.
    Result(&'static Schema, &'static Schema),
    /// Fields in their explicit wire order.
    Record(&'static [Field]),
    /// Alternatives selected by explicit transmitted IDs.
    Enum(&'static [Variant]),
}
