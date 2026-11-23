//! Named production into the sole JSON/typed-graph admission boundary.
use super::{GenerationError, SchemaContext, SchemaLimits, ValidatedRegistry, Wire};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
    vec::Vec,
};
use serde_json::{Value, json};

type Projection = fn(&mut SchemaContext) -> Result<Value, GenerationError>;

#[derive(Debug)]
pub(super) struct Names {
    definitions: BTreeMap<String, Value>,
    labels: BTreeMap<&'static str, String>,
    used: BTreeSet<String>,
    checks: Vec<(&'static str, Projection)>,
    remaining_names: usize,
    verifying: bool,
}

/// Bounded named Rust collection, including shared and recursive projections.
///
/// Explicit root aliases are stable consumer names. Nested generated names retain
/// a deterministic algorithm over compiler type spelling, not a protocol identity.
/// A failed registration invalidates the collector. Finishing rechecks every
/// encountered projection before admitting one immutable typed graph.
#[derive(Debug)]
pub struct SchemaRegistry {
    context: SchemaContext,
    limits: SchemaLimits,
}

impl SchemaRegistry {
    /// Named graph label for this concrete Rust type under the current toolchain.
    #[must_use]
    pub fn type_label<T: ?Sized>() -> String {
        generated_name(core::any::type_name::<T>())
    }

    /// Start collection with explicit host allowances.
    ///
    /// # Errors
    /// Returns Complexity when inline depth exceeds the supported ceiling of 64.
    pub fn new(limits: SchemaLimits) -> Result<Self, GenerationError> {
        if limits.depth > 64 {
            return Err(GenerationError::Complexity);
        }
        Ok(Self {
            context: SchemaContext {
                remaining: limits.nodes,
                depth: limits.depth,
                active: Vec::new(),
                failed: false,
                names: Some(Box::new(Names {
                    definitions: BTreeMap::new(),
                    labels: BTreeMap::new(),
                    used: BTreeSet::new(),
                    checks: Vec::new(),
                    remaining_names: limits.names_bytes,
                    verifying: false,
                })),
            },
            limits,
        })
    }

    /// Register a concrete root under a public alias; duplicate aliases reject.
    ///
    /// # Errors
    /// Schema rejects invalid/duplicate names or a failed collector. Complexity
    /// rejects exhausted construction allowances. Any error invalidates collection.
    pub fn register<T: Wire + ?Sized>(&mut self, root: &str) -> Result<(), GenerationError> {
        if self.context.failed {
            return Err(GenerationError::Schema);
        }
        self.context.failed = true;
        super::super::identifier(root)?;
        let names = self.context.names.as_mut().ok_or(GenerationError::Schema)?;
        names.reserve(root)?;
        let root = String::from(root);
        self.context.failed = false;
        // describe owns sticky failure, including swallowed nested errors.
        let schema = self.context.describe::<T>()?;
        self.context
            .names
            .as_mut()
            .ok_or(GenerationError::Schema)?
            .definitions
            .insert(root, schema);
        Ok(())
    }

    /// Verify deterministic projections, then seal the shared typed graph.
    ///
    /// # Errors
    /// Schema rejects failed/inconsistent collection, unresolved references or
    /// unproductive cycles. Complexity rejects exhausted work or graph limits.
    pub fn finish(mut self) -> Result<ValidatedRegistry, GenerationError> {
        if self.context.failed {
            return Err(GenerationError::Schema);
        }
        let names = self.context.names.as_mut().ok_or(GenerationError::Schema)?;
        names.verifying = true;
        let checks = core::mem::take(&mut names.checks);
        for (key, project) in checks {
            self.context.remaining = self
                .context
                .remaining
                .checked_sub(1)
                .ok_or(GenerationError::Complexity)?;
            let actual = project(&mut self.context)?;
            if self.context.failed {
                return Err(GenerationError::Schema);
            }
            let names = self.context.names.as_ref().ok_or(GenerationError::Schema)?;
            let name = names.labels.get(key).ok_or(GenerationError::Schema)?;
            if names.definitions.get(name) != Some(&actual) {
                return Err(GenerationError::Schema);
            }
        }
        let names = self.context.names.ok_or(GenerationError::Schema)?;
        ValidatedRegistry::from_metadata(&names.definitions, self.limits)
    }
}

impl Names {
    fn reserve(&mut self, name: &str) -> Result<(), GenerationError> {
        self.remaining_names = self
            .remaining_names
            .checked_sub(name.len())
            .ok_or(GenerationError::Complexity)?;
        if !self.used.insert(String::from(name)) {
            return Err(GenerationError::Schema);
        }
        Ok(())
    }
}

impl SchemaContext {
    pub(super) fn named<T: Wire + ?Sized>(&mut self) -> Result<Value, GenerationError> {
        let key = core::any::type_name::<T>();
        let names = self.names.as_mut().ok_or(GenerationError::Schema)?;
        if names.verifying {
            let name = names.labels.get(key).ok_or(GenerationError::Schema)?;
            return Ok(json!({"k":"ref","name":name}));
        }
        // Keep each encountered callback, not just one per diagnostic label.
        // Distinct Rust types can have identical type_name spellings.
        names.checks.push((key, T::schema));
        if let Some(name) = names.labels.get(key) {
            return Ok(json!({"k":"ref","name":name}));
        }
        let name = generated_name(key);
        names.reserve(&name)?;
        names.labels.insert(key, name.clone());
        let schema = T::schema(self)?;
        if self.failed {
            return Err(GenerationError::Schema);
        }
        self.names
            .as_mut()
            .ok_or(GenerationError::Schema)?
            .definitions
            .insert(name.clone(), schema);
        Ok(json!({"k":"ref","name":name}))
    }
}

// Source provenance: revision 2dc434b10bec2fca92345d940dbfedc05993e9d5,
// typescript/native_schema/registry/mod.rs. Retains source-name compatibility.
fn generated_name(key: &str) -> String {
    let base = key
        .split('<')
        .next()
        .unwrap_or(key)
        .rsplit("::")
        .next()
        .unwrap_or(key);
    let stem: String = base
        .chars()
        .take(48)
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let hash = key.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    });
    format!("Wire_{stem}_{hash:016x}")
}
