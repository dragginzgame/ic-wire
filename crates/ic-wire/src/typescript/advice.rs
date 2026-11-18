//! Rooted advice over the admitted graph. Reference contracts: docs/design/parity.md.
use super::{GenerationError, Node, ValidatedRegistry};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
    vec::Vec,
};
use core::fmt;

// Keep the width-specific type name and the selection rule in the same diagnostic.
macro_rules! integer_list {
    ($word:literal) => {
        Some((
            $word,
            concat!(
                "Start with an ordinary list for mostly small values. Try Packed<",
                $word,
                ", N> for wide values or an IC instruction bottleneck. ",
                "Compare encoded_len on typical and worst-case inputs, preserve the field bound, ",
                "and choose one layout for the schema."
            ),
        ))
    };
}

/// A primitive-list use site requiring an explicit representation review.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkWarning {
    /// JSON-pointer path rooted in a public consumer label.
    pub path: String,
    /// Suggested representation or resource policy; never an automatic rewrite.
    pub representation: &'static str,
}
impl fmt::Display for BulkWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.representation)
    }
}
/// Complete bounded advice. Warnings never change generated source or wire bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct BulkLintReport {
    /// Candidates without an exact acknowledgement.
    pub warnings: Vec<BulkWarning>,
    /// Acknowledgements that no longer identify a candidate.
    pub stale_acknowledgements: Vec<String>,
}
impl BulkLintReport {
    /// True only when every candidate is acknowledged and no stale path remains.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty() && self.stale_acknowledgements.is_empty()
    }
}
impl fmt::Display for BulkLintReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut shown = 0;
        for warning in self.warnings.iter().take(8) {
            writeln!(f, "warning: {warning}")?;
            shown += 1;
        }
        for path in self.stale_acknowledgements.iter().take(8 - shown) {
            writeln!(f, "warning: {path}: stale acknowledgement")?;
            shown += 1;
        }
        let omitted = self.warnings.len() + self.stale_acknowledgements.len() - shown;
        if omitted > 0 {
            writeln!(
                f,
                "warning: {omitted} more bulk diagnostics; inspect BulkLintReport for all paths"
            )?;
        }
        Ok(())
    }
}
pub(super) fn escape(name: &str) -> String {
    name.replace('~', "~0").replace('/', "~1")
}
impl ValidatedRegistry {
    /// Review public labels mapped to admitted type names, independent of generation.
    /// References retain use-site paths; recursion stops only on the active path.
    /// Exact acknowledgements need nonempty reasons. Bounds and domain checks
    /// remain consumer-owned; no wildcard suppression or representation rewriting.
    ///
    /// # Errors
    /// Schema for unknown roots or empty reasons; Complexity for depth >=64,
    /// more than 65,536 visits/root or 1,000,000 visits/report, paths over 4 KiB
    /// or retained diagnostic paths over 16 MiB. No partial report is returned.
    pub fn lint_bulk_roots(
        &self,
        roots: &BTreeMap<String, String>,
        acknowledgements: &BTreeMap<String, String>,
    ) -> Result<BulkLintReport, GenerationError> {
        let roots = roots.iter().map(|(label, name)| {
            if label.len() > 4095 {
                return Err(GenerationError::Complexity);
            }
            Ok((format!("/{}", escape(label)), self.root_index(name)?))
        });
        self.lint_roots(roots, acknowledgements)
    }
    pub(super) fn root_index(&self, name: &str) -> Result<usize, GenerationError> {
        self.names
            .binary_search_by(|candidate| candidate.as_str().cmp(name))
            .map_err(|_| GenerationError::Schema)
    }
    pub(super) fn lint_roots(
        &self,
        roots: impl IntoIterator<Item = Result<(String, usize), GenerationError>>,
        acks: &BTreeMap<String, String>,
    ) -> Result<BulkLintReport, GenerationError> {
        if acks.len() > 1_000_000 {
            return Err(GenerationError::Complexity);
        }
        if acks.values().any(|reason| reason.trim().is_empty()) {
            return Err(GenerationError::Schema);
        }
        let mut walk = Walk {
            graph: self,
            active: BTreeSet::new(),
            remaining: 1_000_000,
            root_remaining: 65_536,
            path: String::new(),
            bytes: 16 * 1024 * 1024,
            warnings: Vec::new(),
        };
        for entry in roots {
            let (path, root) = entry?;
            walk.root_remaining = 65_536;
            walk.path = path;
            // Model the public named root as a reference, just like nested refs.
            walk.charge(0)?;
            walk.active.insert(root);
            walk.visit(self.roots[root], 1)?;
            walk.active.remove(&root);
        }
        let paths: BTreeSet<_> = walk
            .warnings
            .iter()
            .map(|warning| warning.path.as_str())
            .collect();
        let mut stale = Vec::new();
        for path in acks.keys().filter(|path| !paths.contains(path.as_str())) {
            if path.len() > 4096 {
                return Err(GenerationError::Complexity);
            }
            walk.bytes = walk
                .bytes
                .checked_sub(path.len())
                .ok_or(GenerationError::Complexity)?;
            stale.push(path.clone());
        }
        walk.warnings
            .retain(|warning| !acks.contains_key(&warning.path));
        Ok(BulkLintReport {
            warnings: walk.warnings,
            stale_acknowledgements: stale,
        })
    }
    fn representation(
        &self,
        mut id: usize,
    ) -> Result<Option<(&'static str, &'static str)>, GenerationError> {
        let mut hash = None;
        for _ in 0..64 {
            match &self.nodes[id] {
                Node::Ref(root) => id = self.roots[*root],
                Node::Alias(item) => id = *item,
                Node::Array { item, len } if hash.is_none() && [16, 32].contains(len) => {
                    hash = Some(*len);
                    id = *item;
                }
                node => {
                    if let Some(width) = hash {
                        return Ok(matches!(
                            node,
                            Node::Integer {
                                bits: 8,
                                signed: false,
                                ..
                            }
                        )
                        .then_some(if width == 16 {
                            ("hash16", "Packed<[u8; 16]>")
                        } else {
                            ("hash32", "Packed<[u8; 32]>")
                        }));
                    }
                    return Ok(match node {
                        Node::Integer {
                            bits: 8,
                            signed: false,
                            ..
                        } => Some((
                            "u8",
                            "For opaque bytes use Bytes<N> for a field bound, ByteBuf for owned bytes, or serde_bytes on an existing byte field.",
                        )),
                        Node::Integer {
                            bits: 8,
                            signed: true,
                            ..
                        } => Some((
                            "i8",
                            "Use Packed<i8, N> for bulk signed bytes when element traversal costs matter. Preserve the element bound.",
                        )),
                        Node::Integer {
                            bits: 16,
                            signed: false,
                            ..
                        } => integer_list!("u16"),
                        Node::Integer {
                            bits: 16,
                            signed: true,
                            ..
                        } => integer_list!("i16"),
                        Node::Integer {
                            bits: 32,
                            signed: false,
                            ..
                        } => integer_list!("u32"),
                        Node::Integer {
                            bits: 32,
                            signed: true,
                            ..
                        } => integer_list!("i32"),
                        Node::Integer {
                            bits: 64,
                            signed: false,
                            ..
                        } => integer_list!("u64"),
                        Node::Integer {
                            bits: 64,
                            signed: true,
                            ..
                        } => integer_list!("i64"),
                        Node::Integer {
                            bits: 128,
                            signed: false,
                            ..
                        } => Some((
                            "u128",
                            "bounded u128 lists and workload-specific node limits; Packed<u128> is unsupported",
                        )),
                        Node::Integer {
                            bits: 128,
                            signed: true,
                            ..
                        } => Some((
                            "i128",
                            "bounded i128 lists and workload-specific node limits; Packed<i128> is unsupported",
                        )),
                        Node::Bool => Some((
                            "bool",
                            "the explicit ic_wire::serde_packed::bools field adapter; preserve field bounds and review buffer admission",
                        )),
                        Node::Float { bits: 32, .. } => Some((
                            "f32",
                            "the explicit finite_f32 adapter for primitive Vec<f32> fields; preserve bounds and review changed bytes/buffer admission; Packed<f32> remains unsupported",
                        )),
                        Node::Float { bits: 64, .. } => Some((
                            "f64",
                            "the explicit finite_f64 adapter for primitive Vec<f64> fields; preserve bounds and review changed bytes/buffer admission; Packed<f64> remains unsupported",
                        )),
                        Node::Unit => Some(("unit", ZERO)),
                        Node::Record(fields) if fields.is_empty() => Some(("empty-record", ZERO)),
                        Node::Tuple(items) if items.is_empty() => Some(("empty-tuple", ZERO)),
                        Node::Array { len: 0, .. } => Some(("empty-array", ZERO)),
                        _ => None,
                    });
                }
            }
        }
        Err(GenerationError::Complexity)
    }
}
const ZERO: &str =
    "an explicit element bound and node limit; zero-byte elements still require traversal";
struct Walk<'a> {
    graph: &'a ValidatedRegistry,
    active: BTreeSet<usize>,
    remaining: usize,
    root_remaining: usize,
    path: String,
    bytes: usize,
    warnings: Vec<BulkWarning>,
}
impl Walk<'_> {
    fn charge(&mut self, depth: usize) -> Result<(), GenerationError> {
        if depth >= 64 || self.path.len() > 4096 {
            return Err(GenerationError::Complexity);
        }
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(GenerationError::Complexity)?;
        self.root_remaining = self
            .root_remaining
            .checked_sub(1)
            .ok_or(GenerationError::Complexity)?;
        Ok(())
    }
    fn child(&mut self, item: usize, suffix: &str, depth: usize) -> Result<(), GenerationError> {
        let length = self.path.len();
        if length + suffix.len() > 4096 {
            return Err(GenerationError::Complexity);
        }
        self.path.push_str(suffix);
        let result = self.visit(item, depth + 1);
        self.path.truncate(length);
        result
    }
    fn visit(&mut self, id: usize, depth: usize) -> Result<(), GenerationError> {
        self.charge(depth)?;
        match &self.graph.nodes[id] {
            Node::Ref(root) => {
                if self.active.insert(*root) {
                    self.visit(self.graph.roots[*root], depth + 1)?;
                    self.active.remove(root);
                }
            }
            Node::List { item, .. } => {
                if let Some((word, representation)) = self.graph.representation(*item)? {
                    let length = self.path.len() + 6 + word.len();
                    if length > 4096 {
                        return Err(GenerationError::Complexity);
                    }
                    self.bytes = self
                        .bytes
                        .checked_sub(length)
                        .ok_or(GenerationError::Complexity)?;
                    self.warnings.push(BulkWarning {
                        path: format!("{}/list/{word}", self.path),
                        representation,
                    });
                }
                self.child(*item, "/item", depth)?;
            }
            Node::Array { item, .. } | Node::Option(item) | Node::Alias(item) => {
                self.child(*item, "/item", depth)?
            }
            Node::Record(fields) => {
                for (name, item) in fields {
                    self.child(*item, &format!("/fields/{}", escape(name)), depth)?;
                }
            }
            Node::Enum(variants) => {
                for (name, _, item) in variants {
                    self.child(*item, &format!("/variants/{}", escape(name)), depth)?;
                }
            }
            Node::Tuple(items) => {
                for (index, item) in items.iter().enumerate() {
                    self.child(*item, &format!("/items/{index}"), depth)?;
                }
            }
            Node::Result { ok, err } => {
                self.child(*ok, "/ok", depth)?;
                self.child(*err, "/err", depth)?;
            }
            _ => {}
        }
        Ok(())
    }
}
