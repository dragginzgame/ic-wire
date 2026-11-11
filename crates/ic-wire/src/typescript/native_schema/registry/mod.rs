//! Finite named projection; names are host metadata, never wire identities.
use super::{GenerationError, SchemaContext, Wire};
use crate::typescript::{SCHEMA_WORK, metadata};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
    vec::Vec,
};
use serde_json::{Value, json};

type Projection = fn(&mut SchemaContext) -> Result<Value, GenerationError>;
#[derive(Default)]
pub(super) struct Names {
    aliases: BTreeMap<&'static str, String>,
    definitions: BTreeMap<String, Value>,
    active: BTreeSet<&'static str>,
    used: BTreeSet<String>,
    checks: Vec<(&'static str, Projection)>,
    verifying: bool,
}

/// Collect concrete native types into the existing named TypeScript grammar.
///
/// Register each root once, reuse its returned reference for methods, then finish.
/// Derived nested declarations receive generated names; generic instantiations
/// remain distinct. Names depend on compiler type names, not traversal, and are not
/// stable contract identities. Manual Wire implementations still own their layout.
/// Projection callbacks must be deterministic: finish rechecks encountered shapes
/// so diagnostic type-name collisions cannot silently substitute a definition.
/// Any failed registration invalidates this collector; start a new one.
pub struct SchemaRegistry {
    context: SchemaContext,
    failed: bool,
}
impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl SchemaRegistry {
    /// Start one bounded collection, with the same ceilings as inline projection.
    pub fn new() -> Self {
        Self {
            context: SchemaContext {
                depth: 0,
                remaining: SCHEMA_WORK,
                names: Some(Box::default()),
            },
            failed: false,
        }
    }
    /// Register a root under a valid TypeScript name, returning its ref node.
    /// Duplicate names reject, even if equal. Already collected types get aliases.
    pub fn register<T: Wire + ?Sized>(&mut self, name: &str) -> Result<Value, GenerationError> {
        if self.failed {
            return Err(GenerationError::Schema);
        }
        self.failed = true;
        metadata::identifier(name)?;
        if self.context.remaining == 0 {
            return Err(GenerationError::Complexity);
        }
        let names = self.context.names.as_mut().unwrap();
        if !names.used.insert(name.into()) {
            return Err(GenerationError::Schema);
        }
        // Explicit roots are aliases too: registering a type as a root before
        // or after another root discovers it must not rename its nested uses.
        let reference = self.context.describe::<T>()?;
        self.context
            .names
            .as_mut()
            .unwrap()
            .definitions
            .insert(name.into(), reference);
        self.failed = false;
        Ok(json!({"k":"ref", "name":name}))
    }
    /// Validate all shapes/references and return the registry for generate_registry.
    pub fn finish(mut self) -> Result<BTreeMap<String, Value>, GenerationError> {
        if self.failed {
            return Err(GenerationError::Schema);
        }
        let names = self.context.names.as_mut().unwrap();
        if !names.active.is_empty() {
            return Err(GenerationError::Schema);
        }
        names.verifying = true;
        let checks = core::mem::take(&mut names.checks);
        // type_name is a diagnostic label, not a unique Rust type identity.
        // Verify each encountered projection against its collected definition,
        // with refs resolved but no further expansion. Colliding labels must
        // describe identical metadata or reject; no definition is overwritten.
        for (key, build) in checks {
            self.context.remaining = self
                .context
                .remaining
                .checked_sub(1)
                .ok_or(GenerationError::Complexity)?;
            let actual = build(&mut self.context)?;
            let names = self.context.names.as_ref().unwrap();
            if names.definitions.get(&names.aliases[key]) != Some(&actual) {
                return Err(GenerationError::Schema);
            }
        }
        let types = self.context.names.unwrap().definitions;
        let mut remaining = SCHEMA_WORK;
        for node in types.values() {
            metadata::ty(node, &types, 0, &mut remaining, false)?;
        }
        metadata::validate_aliases(&types)?;
        Ok(types)
    }
}
impl SchemaContext {
    // Collection lives here rather than in generated derives, so a newer derive
    // never requires a new helper from an older compatible library release.
    pub(super) fn named<T: Wire + ?Sized>(
        &mut self,
        build: impl FnOnce(&mut Self) -> Result<Value, GenerationError>,
    ) -> Result<Value, GenerationError> {
        let Some(names) = self.names.as_mut() else {
            return build(self);
        };
        let key = core::any::type_name::<T>();
        if names.verifying {
            return names
                .aliases
                .get(key)
                .map(|name| json!({"k":"ref","name":name}))
                .ok_or(GenerationError::Schema);
        }
        names.checks.push((key, T::native_schema));
        let name = if let Some(name) = names.aliases.get(key) {
            name.clone()
        } else {
            let name = generated_name(key);
            // Never resolve a collision by registration order. Fail closed,
            // including collisions with explicit consumer root names.
            if !names.used.insert(name.clone()) {
                return Err(GenerationError::Schema);
            }
            names.aliases.insert(key, name.clone());
            name
        };
        if !names.definitions.contains_key(&name) && names.active.insert(key) {
            let node = build(self)?;
            let names = self.names.as_mut().unwrap();
            names.active.remove(key);
            names.definitions.insert(name.clone(), node);
        }
        Ok(json!({"k":"ref", "name":name}))
    }
}

// Host metadata only. Always suffix the complete Rust type spelling, including
// namespace and generic arguments: adding a namesake cannot rename an old type.
// This replaces WireType{index}; regenerate clients and migrate source imports.
// Codec bytes, method identities, limits and acceptance are unchanged.
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
