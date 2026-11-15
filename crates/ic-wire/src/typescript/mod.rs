//! One typed authority for an initial subset of reference JSON metadata.
//!
//! This imports explicit metadata; it does not infer arbitrary Serde layouts.
//! Manual projections remain trusted and require independent byte vectors.
//! Runtime implementation and automatic Rust type collection are not included.

mod render;

/// Generated payload modules require this metadata/runtime interface version.
/// This labels tooling artifacts, never Wire payload bytes.
pub const RUNTIME_SCHEMA_FORMAT: &str = "ic-wire-json/1";

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

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec,
    vec::Vec,
};
use serde_json::Value;

/// Invalid metadata or exhausted host generation allowance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationError {
    /// Unsupported shape, invalid field, unresolved name or alias-only cycle.
    Schema,
    /// Metadata or output exceeds its host-only allowance.
    Complexity,
}

/// Host schema limits, independent of codec runtime allowances.
#[derive(Clone, Copy, Debug)]
pub struct SchemaLimits {
    /// Maximum number of named definitions plus inline nodes.
    pub nodes: usize,
    /// Maximum nested inline JSON depth; references are not expanded.
    pub depth: usize,
    /// Maximum total bytes of names and field/variant labels retained.
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
enum Node {
    Unit,
    Bool,
    Integer { bits: u8, signed: bool },
    Text { max: Option<u64> },
    Bytes { min: Option<u64>, max: Option<u64> },
    PackedI16 { max: u64 },
    List { item: usize, max: Option<u64> },
    Option(usize),
    Alias(usize),
    Ref(usize),
    Record(Vec<(String, usize)>),
    Enum(Vec<(String, u32, usize)>),
}

/// Immutable admitted registry with private node indices.
///
/// Supplied definition names are preserved, including reference-generated names.
/// Input metadata can be mutated or dropped after construction without changing
/// the graph. All rendering consumes this graph, never the original JSON.
#[derive(Debug)]
pub struct ValidatedRegistry {
    names: Vec<String>,
    roots: Vec<usize>,
    nodes: Vec<Node>,
    output_bytes: usize,
}

impl ValidatedRegistry {
    /// Import explicit definitions in the reference metadata grammar.
    ///
    /// Supported kinds: unit, bool, u/i, text, bytes, packed-i16, list, option,
    /// newtype, ref, record and enum. Unknown keys and unsupported kinds reject;
    /// this subset is not a replacement for the reference's full schema surface.
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
            "unit" | "bool" => &["k"],
            "u" | "i" => &["k", "bits"],
            "text" | "packed-i16" => &["k", "max"],
            "bytes" => &["k", "min", "max"],
            "list" => &["k", "item", "max"],
            "option" | "newtype" => &["k", "item"],
            "ref" => &["k", "name"],
            "record" => &["k", "fields"],
            "enum" => &["k", "variants"],
            _ => return Err(GenerationError::Schema),
        };
        keys(value, allowed)?;
        let node = match kind {
            "unit" => Node::Unit,
            "bool" => Node::Bool,
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
            "packed-i16" => {
                let max = bound(value, "max")?.ok_or(GenerationError::Schema)?;
                if max > ((1_u64 << 53) - 1) / 2 {
                    return Err(GenerationError::Complexity);
                }
                Node::PackedI16 { max }
            }
            "list" => Node::List {
                max: bound(value, "max")?,
                item: self.parse(&value["item"], depth + 1)?,
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
            if n > (1_u64 << 53) - 1 {
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
