//! Host-side advice; never changes a schema or emitted bytes.
use super::{GenerationError, SCHEMA_DEPTH, SCHEMA_WORK, metadata};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
    vec::Vec,
};
use serde_json::Value;

// Previously one 65,536-visit counter covered all expanded roots, refusing
// ordinary large composed services. Keep that ceiling per root and allow at
// most 1,000,000 expanded visits for the entire report. This broadens host
// generation capacity only: schemas, wire bytes and runtime admission stay
// unchanged. Consumers can regenerate without changing deployed contracts.
const ROOTED_WORK: usize = 1_000_000;

/// A length-prefixed primitive list that may benefit from bulk admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkWarning {
    /// Stable path: root, field/variant names and container positions, JSON-pointer escaped.
    pub path: String,
    /// Suggested field representation. Review constraints and small-value size tradeoffs.
    pub representation: &'static str,
}

impl core::fmt::Display for BulkWarning {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}: consider {}; ordinary lists can have high per-element encoding and truncated-input rejection costs; preserve constraints and measure small-value size tradeoffs",
            self.path, self.representation
        )
    }
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

impl core::fmt::Display for BulkLintReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Keep default diagnostics bounded even for large generated registries.
        // The complete vectors and exact acknowledgement paths remain available.
        const LIMIT: usize = 8;
        for warning in self.warnings.iter().take(LIMIT) {
            writeln!(f, "warning: {warning}")?;
        }
        let shown = self.warnings.len().min(LIMIT);
        for path in self.stale_acknowledgements.iter().take(LIMIT - shown) {
            writeln!(f, "warning: {path}: stale acknowledgement")?;
        }
        let omitted = self.warnings.len() + self.stale_acknowledgements.len()
            - (shown + self.stale_acknowledgements.len().min(LIMIT - shown));
        if omitted > 0 {
            writeln!(
                f,
                "warning: {omitted} more bulk diagnostics; inspect BulkLintReport for all paths"
            )?;
        }
        Ok(())
    }
}

/// Review primitive lists at explicit public use sites, resolving named references.
///
/// Root labels are consumer-owned identities; values may be inline schemas or
/// registry refs. Reference names never enter diagnostic paths. Each reference
/// is expanded per use site, stopping when it is already active on that path;
/// sibling uses remain independent. Paths end in /list/u32 (or the other word)
/// so changing a candidate's primitive width makes an old acknowledgement stale.
/// Bounds/custom checks still require consumer review; they are not inferred.
/// Empty reasons and invalid schemas return Schema; depth/work exhaustion returns
/// Complexity without a partial report. Byte/packed fields are not candidates.
/// Expanded advice allows 65,536 visits per root and 1,000,000 per report,
/// with depth strictly below 64. Definitions and roots are pre-validated under
/// a separate shared 65,536-node allowance. No use-site warning is truncated.
/// Runtime bytes, budgets and generator output are unchanged.
pub fn lint_bulk_roots(
    roots: &BTreeMap<String, Value>,
    types: &BTreeMap<String, Value>,
    acknowledgements: &BTreeMap<String, String>,
) -> Result<BulkLintReport, GenerationError> {
    let mut remaining = SCHEMA_WORK;
    for node in types.values() {
        metadata::ty(node, types, 0, &mut remaining, false)?;
    }
    metadata::validate_aliases(types)?;
    for node in roots.values() {
        metadata::ty(node, types, 0, &mut remaining, false)?;
    }
    lint_validated(
        roots
            .iter()
            .map(|(name, node)| (format!("/{}", escape(name)), node)),
        types,
        acknowledgements,
    )
}

// Generation has already validated all metadata. Reuse the bounded traversal
// without copying schemas or repeating the generator's validation pass.
pub(super) fn lint_generated(
    types: &BTreeMap<String, Value>,
    methods: &[Value],
    acknowledgements: &BTreeMap<String, String>,
) -> Result<BulkLintReport, GenerationError> {
    if methods.is_empty() {
        lint_validated(
            types
                .iter()
                .map(|(name, node)| (format!("/types/{}", escape(name)), node)),
            types,
            acknowledgements,
        )
    } else {
        lint_validated(
            methods.iter().flat_map(|method| {
                ["request", "response"].map(|direction| {
                    (
                        format!(
                            "/methods/{}/{direction}",
                            escape(method["name"].as_str().unwrap())
                        ),
                        &method[direction],
                    )
                })
            }),
            types,
            acknowledgements,
        )
    }
}

pub(super) fn lint_validated<'a>(
    roots: impl Iterator<Item = (String, &'a Value)>,
    types: &BTreeMap<String, Value>,
    acknowledgements: &BTreeMap<String, String>,
) -> Result<BulkLintReport, GenerationError> {
    if acknowledgements
        .values()
        .any(|reason| reason.trim().is_empty())
    {
        return Err(GenerationError::Schema);
    }
    let mut warnings = Vec::new();
    let mut traversal = Traversal {
        active: BTreeSet::new(),
        remaining: ROOTED_WORK,
        root_remaining: SCHEMA_WORK,
    };
    for (path, node) in roots {
        traversal.root_remaining = SCHEMA_WORK;
        walk(node, &path, types, &mut warnings, &mut traversal, 0)?;
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
) -> Result<Option<(&'static str, &'static str)>, GenerationError> {
    // Follow aliases and at most one hash array without recursive probing.
    // Advice changes only: schemas, emitted codec bytes and admission stay intact.
    let mut hash = None;
    for _ in 0..SCHEMA_DEPTH {
        match node["k"].as_str().unwrap() {
            "ref" => node = &types[node["name"].as_str().unwrap()],
            "newtype" => node = &node["item"],
            "array" if hash.is_none() && matches!(node["max"].as_u64(), Some(16 | 32)) => {
                hash = node["max"].as_u64();
                node = &node["item"];
            }
            kind => {
                if let Some(width) = hash {
                    return Ok((kind == "u" && node["bits"].as_u64() == Some(8)).then_some(
                        if width == 16 {
                            ("hash16", "Packed<[u8; 16]>")
                        } else {
                            ("hash32", "Packed<[u8; 32]>")
                        },
                    ));
                }
                return Ok(match (kind, node["bits"].as_u64()) {
                    ("u", Some(8)) => Some(("u8", "ByteBuf / Bytes<N> / serde_bytes")),
                    ("i", Some(8)) => Some(("i8", "Packed<i8>")),
                    ("i", Some(16)) => Some(("i16", "Packed<i16>")),
                    ("u", Some(16)) => Some(("u16", "Packed<u16>")),
                    ("i", Some(32)) => Some(("i32", "Packed<i32>")),
                    ("u", Some(32)) => Some(("u32", "Packed<u32>")),
                    ("i", Some(64)) => Some(("i64", "Packed<i64>")),
                    ("u", Some(64)) => Some(("u64", "Packed<u64>")),
                    ("i", Some(128)) => Some((
                        "i128",
                        "bounded i128 lists and workload-specific node limits; Packed<i128> is unsupported",
                    )),
                    ("u", Some(128)) => Some((
                        "u128",
                        "bounded u128 lists and workload-specific node limits; Packed<u128> is unsupported",
                    )),
                    ("float", Some(32)) => Some((
                        "f32",
                        "the explicit finite_f32 adapter for primitive Vec<f32> fields; preserve bounds and review changed bytes/buffer admission; Packed<f32> remains unsupported",
                    )),
                    ("float", Some(64)) => Some((
                        "f64",
                        "the explicit finite_f64 adapter for primitive Vec<f64> fields; preserve bounds and review changed bytes/buffer admission; Packed<f64> remains unsupported",
                    )),
                    ("bool", _) => Some((
                        "bool",
                        "the explicit ic_wire::serde_packed::bools field adapter; preserve field bounds and review buffer admission",
                    )),
                    ("unit", _) => Some((
                        "unit",
                        "an explicit element bound and node limit; zero-byte elements still require traversal",
                    )),
                    ("record", _) if node["fields"].as_array().is_some_and(Vec::is_empty) => {
                        Some((
                            "empty-record",
                            "an explicit element bound and node limit; zero-byte elements still require traversal",
                        ))
                    }
                    ("tuple", _) if node["items"].as_array().is_some_and(Vec::is_empty) => Some((
                        "empty-tuple",
                        "an explicit element bound and node limit; zero-byte elements still require traversal",
                    )),
                    ("array", _) if node["max"].as_u64() == Some(0) => Some((
                        "empty-array",
                        "an explicit element bound and node limit; zero-byte elements still require traversal",
                    )),
                    _ => None,
                });
            }
        }
    }
    Err(GenerationError::Complexity)
}

struct Traversal {
    active: BTreeSet<String>,
    remaining: usize,
    root_remaining: usize,
}

fn walk(
    node: &Value,
    path: &str,
    types: &BTreeMap<String, Value>,
    warnings: &mut Vec<BulkWarning>,
    traversal: &mut Traversal,
    depth: usize,
) -> Result<(), GenerationError> {
    if depth >= SCHEMA_DEPTH || traversal.remaining == 0 || traversal.root_remaining == 0 {
        return Err(GenerationError::Complexity);
    }
    traversal.remaining -= 1;
    traversal.root_remaining -= 1;
    match node["k"].as_str().unwrap() {
        "ref" => {
            let name = node["name"].as_str().unwrap();
            if traversal.active.insert(name.into()) {
                walk(&types[name], path, types, warnings, traversal, depth + 1)?;
                traversal.active.remove(name);
            }
        }
        "list" => {
            if let Some((word, representation)) = representation(&node["item"], types)? {
                warnings.push(BulkWarning {
                    path: format!("{path}/list/{word}"),
                    representation,
                });
            }
            walk(
                &node["item"],
                &format!("{path}/item"),
                types,
                warnings,
                traversal,
                depth + 1,
            )?;
        }
        "array" | "option" | "newtype" => walk(
            &node["item"],
            &format!("{path}/item"),
            types,
            warnings,
            traversal,
            depth + 1,
        )?,
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
                    traversal,
                    depth + 1,
                )?;
            }
        }
        "tuple" => {
            for (index, item) in node["items"].as_array().unwrap().iter().enumerate() {
                walk(
                    item,
                    &format!("{path}/items/{index}"),
                    types,
                    warnings,
                    traversal,
                    depth + 1,
                )?;
            }
        }
        "result" => {
            for key in ["ok", "err"] {
                walk(
                    &node[key],
                    &format!("{path}/{key}"),
                    types,
                    warnings,
                    traversal,
                    depth + 1,
                )?;
            }
        }
        _ => {} // Scalars and bulk fields.
    }
    Ok(())
}
