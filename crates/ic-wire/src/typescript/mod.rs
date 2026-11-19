//! One typed authority for supported JSON metadata.
//!
//! This imports explicit metadata; it does not infer arbitrary Serde layouts.
//! Manual projections remain trusted and require independent byte vectors.
//! Named Rust type collection and a matching subset runtime share this contract.

mod advice;
mod native;
mod render;

pub use advice::{BulkLintReport, BulkWarning};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec,
    vec::Vec,
};
#[cfg(feature = "derive")]
pub use ic_wire_derive::Wire;
#[doc(hidden)]
pub use native::{PackedField, SerdeBytesField};
pub use native::{SchemaContext, SchemaRegistry, Wire};
use serde_json::{Number, Value};

const MAX_SAFE_INTEGER: u64 = (1_u64 << 53) - 1;

/// Macro support for generated metadata; not a second schema API.
#[doc(hidden)]
pub mod __private {
    pub use serde_json::{Value, json};
}

/// Generated client modules require this metadata/runtime interface marker.
/// This labels tooling artifacts, never Wire payload bytes.
pub const RUNTIME_SCHEMA_FORMAT: &str = "ic-wire-json";

/// TypeScript interpreter for the admitted schema subset. Write this beside
/// generated client modules at their selected runtime import path.
/// Contains no transport, SDK, endpoint execution or deployment implementation.
pub const RUNTIME: &str = include_str!("runtime.ts");

/// Explicit browser payload allowances, owned by the consuming application.
///
/// Node/depth accounting follows the selected browser runtime, not Rust Serde
/// traversal. No native allocation allowance is translated into a JS heap limit.
#[derive(Clone, Copy, Debug)]
pub struct PayloadLimits {
    /// Maximum payload bytes, excluding any frame or transport envelope.
    pub bytes: u64,
    /// Maximum runtime value visits; must be nonzero.
    pub nodes: u64,
    /// Maximum runtime value depth; must be nonzero.
    pub depth: u64,
}

/// One typed request/response contract for generated framed clients.
///
/// Both byte allowances are payload-only; generation adds the existing 11-byte
/// header. Node/depth allowances retain browser runtime accounting.
#[derive(Clone, Copy, Debug)]
pub struct Method<'a> {
    /// Exported codec identifier. Does not implicitly determine the method ID.
    pub export: &'a str,
    /// Exact routing identity; use crate::method_id explicitly for named IDs.
    pub id: u64,
    /// Admitted request type name.
    pub request: &'a str,
    /// Admitted response type name.
    pub response: &'a str,
    /// Request payload allowances.
    pub request_limits: PayloadLimits,
    /// Response payload allowances.
    pub response_limits: PayloadLimits,
}

/// Invalid metadata or exhausted schema generation allowance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationError {
    /// Unsupported shape, invalid field, unresolved name or alias-only cycle.
    Schema,
    /// Metadata or output exceeds its schema allowance.
    Complexity,
}

/// Schema limits, independent of codec runtime allowances.
#[derive(Clone, Copy, Debug)]
pub struct SchemaLimits {
    /// Maximum number of named definitions plus inline nodes.
    pub nodes: usize,
    /// Maximum nested inline JSON depth; references are not expanded.
    pub depth: usize,
    /// Maximum total bytes of names, field/variant labels and check names retained.
    pub names_bytes: usize,
    /// Maximum bytes per emitted declaration or metadata artifact.
    pub output_bytes: usize,
}

impl Default for SchemaLimits {
    fn default() -> Self {
        Self {
            nodes: 65_536,
            depth: 64,
            names_bytes: 1_048_576,
            output_bytes: 1_048_576,
        }
    }
}

#[derive(Debug)]
struct ScalarBounds {
    min: Option<Number>,
    max: Option<Number>,
}

#[derive(Debug)]
struct Annotations {
    check: Option<String>,
    bounds: Option<ScalarBounds>,
}

#[derive(Debug)]
enum Node {
    Unit,
    Bool,
    Integer {
        bits: u8,
        signed: bool,
    },
    Float {
        bits: u8,
    },
    Text {
        max: Option<u64>,
    },
    Bytes {
        min: Option<u64>,
        max: Option<u64>,
    },
    Packed {
        word: &'static str,
        nonzero: bool,
        max: Option<u64>,
    },
    List {
        item: usize,
        max: Option<u64>,
    },
    Array {
        item: usize,
        len: u64,
    },
    Tuple(Vec<usize>),
    Result {
        ok: usize,
        err: usize,
    },
    Option(usize),
    Alias(usize),
    Ref(usize),
    Record(Vec<(String, usize)>),
    Enum(Vec<(String, u32, usize)>),
}

/// Immutable admitted registry with private node indices.
///
/// Supplied definition names are preserved.
/// Input metadata can be mutated or dropped after construction without changing
/// the graph. All rendering consumes this graph, never the original JSON.
#[derive(Debug)]
pub struct ValidatedRegistry {
    names: Vec<String>,
    roots: Vec<usize>,
    nodes: Vec<Node>,
    annotations: BTreeMap<usize, Annotations>,
    output_bytes: usize,
}

impl ValidatedRegistry {
    /// Import explicit definitions in the Wire metadata grammar.
    ///
    /// Supported kinds: unit, bool, u/i, float, text, bytes, packed, list,
    /// array, tuple, result, option, newtype, ref, record and enum. Unknown keys reject;
    /// manual metadata must match the consumer's actual Serde representation.
    ///
    /// # Errors
    /// Returns Schema for invalid metadata and Complexity for exceeded bounds.
    /// Failure returns no partially usable registry. Input JSON allocation is
    /// caller-owned; limits apply to graph import, not parsing external JSON text.
    pub fn from_metadata(
        types: &BTreeMap<String, Value>,
        limits: SchemaLimits,
    ) -> Result<Self, GenerationError> {
        if limits.depth > 64 || types.len() > limits.nodes {
            return Err(GenerationError::Complexity);
        }
        let mut importer = Importer {
            nodes: Vec::new(),
            annotations: BTreeMap::new(),
            names: BTreeMap::new(),
            remaining: limits.nodes - types.len(),
            name_bytes: limits.names_bytes,
            depth: limits.depth,
        };
        let mut names = Vec::new();
        for (index, name) in types.keys().enumerate() {
            identifier(name)?;
            let name = importer.name(name)?;
            importer.names.insert(name.clone(), index);
            names.push(name);
        }
        let mut roots = Vec::new();
        for value in types.values() {
            roots.push(importer.parse(value, 0)?);
        }
        let graph = Self {
            names,
            roots,
            nodes: importer.nodes,
            annotations: importer.annotations,
            output_bytes: limits.output_bytes,
        };
        graph.validate_aliases()?;
        Ok(graph)
    }

    // Linear worklist over transparent edges. Productive recursive records,
    // lists and options terminate an alias chain rather than expanding it.
    fn validate_aliases(&self) -> Result<(), GenerationError> {
        let mut state = vec![0_u8; self.nodes.len()];
        let mut path = Vec::new();
        for start in 0..self.nodes.len() {
            let mut current = start;
            path.clear();
            loop {
                match state[current] {
                    1 => return Err(GenerationError::Schema),
                    2 => break,
                    _ => {}
                }
                state[current] = 1;
                path.push(current);
                match self.nodes[current] {
                    Node::Alias(next) => current = next,
                    Node::Ref(root) => current = self.roots[root],
                    _ => break,
                }
            }
            for node in &path {
                state[*node] = 2;
            }
        }
        Ok(())
    }
}

struct Importer {
    nodes: Vec<Node>,
    annotations: BTreeMap<usize, Annotations>,
    names: BTreeMap<String, usize>,
    remaining: usize,
    name_bytes: usize,
    depth: usize,
}

impl Importer {
    fn name(&mut self, name: &str) -> Result<String, GenerationError> {
        self.name_bytes = self
            .name_bytes
            .checked_sub(name.len())
            .ok_or(GenerationError::Complexity)?;
        Ok(name.to_string())
    }

    fn parse(&mut self, value: &Value, depth: usize) -> Result<usize, GenerationError> {
        if depth > self.depth {
            return Err(GenerationError::Complexity);
        }
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(GenerationError::Complexity)?;
        let kind = text(value, "k")?;
        let allowed: &[&str] = match kind {
            "unit" | "bool" => &["k", "check"],
            "u" | "i" | "float" => &["k", "check", "bits", "min", "max"],
            "text" => &["k", "check", "max"],
            "packed" => &["k", "check", "word", "nonzero", "max"],
            "bytes" => &["k", "check", "min", "max"],
            "list" | "array" => &["k", "check", "item", "max"],
            "tuple" => &["k", "check", "items"],
            "result" => &["k", "check", "ok", "err"],
            "option" | "newtype" => &["k", "check", "item"],
            "ref" => &["k", "check", "name"],
            "record" => &["k", "check", "fields"],
            "enum" => &["k", "check", "variants"],
            _ => return Err(GenerationError::Schema),
        };
        keys(value, allowed)?;
        let node = match kind {
            "unit" => Node::Unit,
            "bool" => Node::Bool,
            "float" => {
                let bits = value["bits"].as_u64().ok_or(GenerationError::Schema)?;
                if ![32, 64].contains(&bits) {
                    return Err(GenerationError::Schema);
                }
                Node::Float { bits: bits as u8 }
            }
            "u" | "i" => {
                let bits = value["bits"].as_u64().ok_or(GenerationError::Schema)?;
                if ![8, 16, 32, 64, 128].contains(&bits) {
                    return Err(GenerationError::Schema);
                }
                Node::Integer {
                    bits: bits as u8,
                    signed: kind == "i",
                }
            }
            "text" => Node::Text {
                max: bound(value, "max")?,
            },
            "bytes" => {
                let min = bound(value, "min")?;
                let max = bound(value, "max")?;
                if min.zip(max).is_some_and(|(min, max)| min > max) {
                    return Err(GenerationError::Schema);
                }
                Node::Bytes { min, max }
            }
            "packed" => {
                let word = text(value, "word")?;
                let word = [
                    "i8",
                    "i16",
                    "u16",
                    "i32",
                    "u32",
                    "i64",
                    "u64",
                    "hash16",
                    "hash32",
                    "bool",
                    "finite-f32",
                    "finite-f64",
                ]
                .into_iter()
                .find(|candidate| *candidate == word)
                .ok_or(GenerationError::Schema)?;
                let nonzero = match value.get("nonzero") {
                    None => false,
                    Some(Value::Bool(true)) if word == "i64" => true,
                    _ => return Err(GenerationError::Schema),
                };
                let max = bound(value, "max")?;
                let width = match word {
                    "bool" | "i8" => 1,
                    "i16" | "u16" => 2,
                    "i32" | "u32" | "finite-f32" => 4,
                    "i64" | "u64" | "finite-f64" => 8,
                    "hash16" => 16,
                    _ => 32,
                };
                if max.is_some_and(|n| n > MAX_SAFE_INTEGER / width) {
                    return Err(GenerationError::Complexity);
                }
                Node::Packed { word, nonzero, max }
            }
            "list" => Node::List {
                max: bound(value, "max")?,
                item: self.parse(&value["item"], depth + 1)?,
            },
            "array" => Node::Array {
                len: bound(value, "max")?.ok_or(GenerationError::Schema)?,
                item: self.parse(&value["item"], depth + 1)?,
            },
            "tuple" => {
                let items = value["items"].as_array().ok_or(GenerationError::Schema)?;
                if items.len() > self.remaining {
                    return Err(GenerationError::Complexity);
                }
                Node::Tuple(
                    items
                        .iter()
                        .map(|item| self.parse(item, depth + 1))
                        .collect::<Result<_, _>>()?,
                )
            }
            "result" => Node::Result {
                ok: self.parse(&value["ok"], depth + 1)?,
                err: self.parse(&value["err"], depth + 1)?,
            },
            "option" => Node::Option(self.parse(&value["item"], depth + 1)?),
            "newtype" => Node::Alias(self.parse(&value["item"], depth + 1)?),
            "ref" => Node::Ref(
                *self
                    .names
                    .get(text(value, "name")?)
                    .ok_or(GenerationError::Schema)?,
            ),
            "record" | "enum" => {
                let is_enum = kind == "enum";
                let fields = value[if is_enum { "variants" } else { "fields" }]
                    .as_array()
                    .ok_or(GenerationError::Schema)?;
                if is_enum && fields.is_empty() {
                    return Err(GenerationError::Schema);
                }
                if fields.len() > self.remaining {
                    return Err(GenerationError::Complexity);
                }
                let mut seen = BTreeSet::new();
                let mut ids = BTreeSet::new();
                let mut record = Vec::new();
                let mut variants = Vec::new();
                for field in fields {
                    keys(
                        field,
                        if is_enum {
                            &["name", "id", "value"]
                        } else {
                            &["name", "value"]
                        },
                    )?;
                    let name = text(field, "name")?;
                    if !seen.insert(name) {
                        return Err(GenerationError::Schema);
                    }
                    let name = self.name(name)?;
                    let child = self.parse(&field["value"], depth + 1)?;
                    if is_enum {
                        let id = field["id"]
                            .as_u64()
                            .and_then(|id| u32::try_from(id).ok())
                            .ok_or(GenerationError::Schema)?;
                        if !ids.insert(id) {
                            return Err(GenerationError::Schema);
                        }
                        variants.push((name, id, child));
                    } else {
                        record.push((name, child));
                    }
                }
                if is_enum {
                    Node::Enum(variants)
                } else {
                    Node::Record(record)
                }
            }
            _ => return Err(GenerationError::Schema),
        };
        let id = self.nodes.len();
        let check = value
            .get("check")
            .map(|check| self.name(check.as_str().ok_or(GenerationError::Schema)?))
            .transpose()?;
        let bounds = if matches!(kind, "u" | "i" | "float")
            && (value.get("min").is_some() || value.get("max").is_some())
        {
            Some(scalar_bounds(value, kind != "float")?)
        } else {
            None
        };
        if check.is_some() || bounds.is_some() {
            self.annotations.insert(id, Annotations { check, bounds });
        }
        self.nodes.push(node);
        Ok(id)
    }
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, GenerationError> {
    value[key].as_str().ok_or(GenerationError::Schema)
}

fn keys(value: &Value, allowed: &[&str]) -> Result<(), GenerationError> {
    if value
        .as_object()
        .ok_or(GenerationError::Schema)?
        .keys()
        .any(|key| !allowed.contains(&key.as_str()))
    {
        return Err(GenerationError::Schema);
    }
    Ok(())
}

fn bound(value: &Value, key: &str) -> Result<Option<u64>, GenerationError> {
    value
        .get(key)
        .map(|n| {
            let n = n.as_u64().ok_or(GenerationError::Schema)?;
            if n > MAX_SAFE_INTEGER {
                return Err(GenerationError::Complexity);
            }
            Ok(n)
        })
        .transpose()
}

fn identifier(name: &str) -> Result<(), GenerationError> {
    if name.is_empty()
        || !name
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
    {
        return Err(GenerationError::Schema);
    }
    if RESERVED.contains(&name) {
        return Err(GenerationError::Schema);
    }
    Ok(())
}

const RESERVED: &[&str] = &[
    "as",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "await",
    "any",
    "boolean",
    "bigint",
    "never",
    "number",
    "object",
    "string",
    "symbol",
    "undefined",
    "unknown",
    "eval",
    "arguments",
];

fn scalar_bounds(value: &Value, integer: bool) -> Result<ScalarBounds, GenerationError> {
    let read = |key| -> Result<Option<Number>, GenerationError> {
        value
            .get(key)
            .map(|value| {
                let number = value.as_number().ok_or(GenerationError::Schema)?;
                let n = number
                    .as_f64()
                    .filter(|n| n.is_finite())
                    .ok_or(GenerationError::Schema)?;
                if integer {
                    if n % 1.0 != 0.0 {
                        return Err(GenerationError::Schema);
                    }
                    if n.abs() > 9_007_199_254_740_991.0 {
                        return Err(GenerationError::Complexity);
                    }
                }
                Ok(number.clone())
            })
            .transpose()
    };
    let bounds = ScalarBounds {
        min: read("min")?,
        max: read("max")?,
    };
    if bounds
        .min
        .as_ref()
        .zip(bounds.max.as_ref())
        .is_some_and(|(min, max)| min.as_f64() > max.as_f64())
    {
        return Err(GenerationError::Schema);
    }
    Ok(bounds)
}
