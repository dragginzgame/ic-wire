//! Host-side TypeScript projection of Serde layouts.
mod bulk_lint;
pub use bulk_lint::{BulkLintReport, BulkWarning, lint_bulk_fields, lint_bulk_roots};
mod limits;
mod metadata;
pub use limits::{method_limits, payload_limits};
mod native_schema;
pub use metadata::{generate_payload_registry, generate_registry, validate_payload_schema};
#[doc(hidden)]
pub use native_schema::SerdeBytesField;
pub use native_schema::{SchemaContext, SchemaRegistry, Wire, native_schema};

// Inline projection, named collection (including collision rechecks), finish
// validation, bulk pre-validation, and metadata validation/rendering each keep
// their existing 65,536-work allowance. References remain leaves in validation;
// rooted advice instead expands them at every use site (see bulk_lint).
// All host ceilings are independent of runtime Limits and wire acceptance.
const SCHEMA_WORK: usize = 65_536;
const SCHEMA_DEPTH: usize = 64;

// JavaScript's safe-integer ceiling for generated numeric bounds.
const MAX_SAFE_INTEGER: u64 = (1_u64 << 53) - 1;

/// Shared bounded Postcard implementation. Write beside generated codec modules
/// as `wire-runtime.ts`; it imports no SDK and owns no transport/auth policy.
pub const RUNTIME: &str = include_str!("runtime.ts");

/// A contract cannot be faithfully projected into a TypeScript module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationError {
    /// Invalid codec resource allowances.
    Limits,
    /// Missing/custom metadata, duplicate identities, or unsupported widths.
    Schema,
    /// Metadata is too large, too deep, or outside exact JavaScript integer range.
    Complexity,
}
