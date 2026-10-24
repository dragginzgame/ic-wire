//! Optional host-side advice; never changes a schema or emitted bytes.
use super::{GenerationError, metadata};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
    vec::Vec,
};
use serde_json::Value;

/// A length-prefixed primitive list that may benefit from bulk admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkWarning {
    /// Stable path: root, field/variant names and container positions, JSON-pointer escaped.
    pub path: String,
    /// Suggested field representation. Review constraints and small-value size tradeoffs.
    pub representation: &'static str,
}

/// Advisory results suitable for printing locally or enforcing in consumer CI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkLintReport {
    /// Candidates without a matching acknowledgement.
    pub warnings: Vec<BulkWarning>,
    /// Acknowledged paths no longer identifying a candidate; remove or update them.
    pub stale_acknowledgements: Vec<String>,
}
impl BulkLintReport {
    /// True when every candidate is acknowledged and no acknowledgement is stale.
    /// A consumer may assert this in CI; linting itself never fails on warnings.
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty() && self.stale_acknowledgements.is_empty()
    }
}

/// Find ordinary primitive lists in resolved Wire metadata, on the host only.
///
/// Pass named schema roots (including request/response schemas not otherwise in
/// the registry). Named references are inspected at their definition, once;
/// Rust aliases already resolve through `native_schema`. Nested lists, records,
/// variants, tuples and options are traversed. Fixed arrays, packed/byte fields,
/// unsupported words and lists of general records are not candidates.
///
/// Acknowledgements map exact diagnostic paths to nonempty reasons. Unmatched
/// entries are reported as stale, preventing broad suppression after a schema
/// change. Warnings are advisory for all lengths: metadata cannot predict actual
/// payload sizes or a 10x slowdown. Preserve field bounds/custom checks when
/// adopting a suggestion; primitive newtypes may carry consumer constraints.
///
/// This does not alter generator output, encoding, budgets or client types.
/// Consumers choose whether to print warnings or require `report.is_clean()`.
/// Invalid schemas/reasons return Schema; traversal uses generation's limits.
pub fn lint_bulk_fields(
    types: &BTreeMap<String, Value>,
    acknowledgements: &BTreeMap<String, String>,
) -> Result<BulkLintReport, GenerationError> {
    let mut remaining = 65536;
    for node in types.values() {
        metadata::ty(node, types, 0, &mut remaining)?;
    }
    metadata::validate_aliases(types)?;
    if acknowledgements
        .values()
        .any(|reason| reason.trim().is_empty())
    {
        return Err(GenerationError::Schema);
    }
    let mut warnings = Vec::new();
    for (name, node) in types {
        walk(node, &format!("/{}", escape(name)), types, &mut warnings)?;
    }
    let paths: BTreeSet<_> = warnings.iter().map(|warning| &warning.path).collect();
    let stale_acknowledgements = acknowledgements
        .keys()
        .filter(|path| !paths.contains(*path))
        .cloned()
        .collect();
    warnings.retain(|warning| !acknowledgements.contains_key(&warning.path));
    Ok(BulkLintReport {
        warnings,
        stale_acknowledgements,
    })
}

fn escape(name: &str) -> String {
    name.replace('~', "~0").replace('/', "~1")
}

fn representation<'a>(
    mut node: &'a Value,
    types: &'a BTreeMap<String, Value>,
) -> Result<Option<&'static str>, GenerationError> {
    // Follow scalar aliases only. Structural recursion is visited at its named
    // definition, so recursive records never expand into an unbounded walk.
    for _ in 0..64 {
        match node["k"].as_str().unwrap() {
            "ref" => node = &types[node["name"].as_str().unwrap()],
            "newtype" => node = &node["item"],
            kind => {
                return Ok(match (kind, node["bits"].as_u64()) {
                    ("u", Some(8)) => Some("ByteBuf / Bytes<N> / serde_bytes"),
                    ("i", Some(8)) => Some("Packed<i8>"),
                    ("i", Some(16)) => Some("Packed<i16>"),
                    ("u", Some(16)) => Some("Packed<u16>"),
                    ("i", Some(32)) => Some("Packed<i32>"),
                    ("u", Some(32)) => Some("Packed<u32>"),
                    ("i", Some(64)) => Some("Packed<i64>"),
                    ("u", Some(64)) => Some("Packed<u64>"),
                    _ => None,
                });
            }
        }
    }
    Err(GenerationError::Complexity)
}

fn walk(
    node: &Value,
    path: &str,
    types: &BTreeMap<String, Value>,
    warnings: &mut Vec<BulkWarning>,
) -> Result<(), GenerationError> {
    match node["k"].as_str().unwrap() {
        "list" => {
            if let Some(representation) = representation(&node["item"], types)? {
                warnings.push(BulkWarning {
                    path: path.into(),
                    representation,
                });
            }
            walk(&node["item"], &format!("{path}/item"), types, warnings)?;
        }
        "array" | "option" | "newtype" => {
            walk(&node["item"], &format!("{path}/item"), types, warnings)?
        }
        "record" | "enum" => {
            let key = if node["k"] == "record" {
                "fields"
            } else {
                "variants"
            };
            for field in node[key].as_array().unwrap() {
                walk(
                    &field["value"],
                    &format!("{path}/{key}/{}", escape(field["name"].as_str().unwrap())),
                    types,
                    warnings,
                )?;
            }
        }
        "tuple" => {
            for (index, item) in node["items"].as_array().unwrap().iter().enumerate() {
                walk(item, &format!("{path}/items/{index}"), types, warnings)?;
            }
        }
        "result" => {
            for key in ["ok", "err"] {
                walk(&node[key], &format!("{path}/{key}"), types, warnings)?;
            }
        }
        _ => {} // Scalars, bulk fields, and references to separately visited definitions.
    }
    Ok(())
}
