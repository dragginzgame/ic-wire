//! Finite named projection; names are host metadata, never wire identities.
use super::{GenerationError, SchemaContext, Wire};
use crate::typescript::metadata;
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
/// remain distinct. Names depend on traversal/compiler type names and are not
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
                remaining: 65536,
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
        let key = core::any::type_name::<T>();
        if !names.used.insert(name.into()) {
            return Err(GenerationError::Schema);
        }
        if let Some(existing) = names.aliases.get(key) {
            names.checks.push((key, T::native_schema));
            names
                .definitions
                .insert(name.into(), json!({"k":"ref","name":existing}));
            self.context.remaining -= 1;
            self.failed = false;
            return Ok(json!({"k":"ref","name":name}));
        }
        names.aliases.insert(key, name.into());
        self.context.describe::<T>()?;
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
        let mut remaining = 65536;
        for node in types.values() {
            metadata::ty(node, &types, 0, &mut remaining)?;
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
            let mut index = names.aliases.len();
            let name = loop {
                let name = format!("WireType{index}");
                if names.used.insert(name.clone()) {
                    break name;
                }
                index += 1;
            };
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
