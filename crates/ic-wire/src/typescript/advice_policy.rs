//! Application-scoped decisions over diagnostics from independently rendered modules.
use super::{BulkLintReport, GenerationError, SchemaRegistry};
use alloc::{collections::BTreeMap, string::String};

/// Stable machine-readable advisory identifier. Diagnostic wording may change.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BulkAdvisory {
    /// Review byte-buffer encoding for ordinary unsigned byte lists.
    OpaqueBytes,
    /// Review packed encoding for fixed-width items.
    PackedList,
    /// Review a bitmap or bulk boolean representation.
    BooleanList,
    /// Review limits for wide integers without a packed representation.
    WideIntegerList,
    /// Review traversal limits for zero-byte elements.
    ZeroByteList,
}

impl BulkAdvisory {
    /// Stable identifier for exporter configuration and diagnostic output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpaqueBytes => "opaque-bytes",
            Self::PackedList => "packed-list",
            Self::BooleanList => "boolean-list",
            Self::WideIntegerList => "wide-integer-list",
            Self::ZeroByteList => "zero-byte-list",
        }
    }
}

impl core::str::FromStr for BulkAdvisory {
    type Err = GenerationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        [
            Self::OpaqueBytes,
            Self::PackedList,
            Self::BooleanList,
            Self::WideIntegerList,
            Self::ZeroByteList,
        ]
        .into_iter()
        .find(|id| id.as_str() == value)
        .ok_or(GenerationError::Schema)
    }
}

/// Exact named declaration in the admitted graph, independent of the endpoint path.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BulkSite {
    /// Declared list element, before following its representation.
    ListElement(String),
    /// Named record and serialized field of a list.
    Field(String, String),
}

impl BulkSite {
    /// Match lists of this concrete Rust type in a named schema collection.
    #[must_use]
    pub fn list_element<T: ?Sized>() -> Self {
        Self::ListElement(SchemaRegistry::type_label::<T>())
    }

    /// Match the serialized field of this named Rust record.
    #[must_use]
    pub fn field<T: ?Sized>(name: &str) -> Self {
        Self::Field(SchemaRegistry::type_label::<T>(), name.into())
    }
}

/// Reasoned decisions for one complete application export. Create a fresh policy per run.
#[derive(Default)]
pub struct BulkPolicy {
    rules: BTreeMap<BulkSite, BTreeMap<BulkAdvisory, Decision>>,
}

struct Decision {
    reason: String,
    matched: bool,
}

impl BulkPolicy {
    /// Register a declaration and advisory pair.
    ///
    /// # Errors
    /// Returns Schema for an empty reason, empty declaration or duplicate rule.
    pub fn acknowledge(
        &mut self,
        site: BulkSite,
        advisory: BulkAdvisory,
        reason: &str,
    ) -> Result<(), GenerationError> {
        if reason.trim().is_empty()
            || match &site {
                BulkSite::ListElement(name) => name.is_empty(),
                BulkSite::Field(record, field) => record.is_empty() || field.is_empty(),
            }
        {
            return Err(GenerationError::Schema);
        }
        let rules = self.rules.entry(site).or_default();
        let alloc::collections::btree_map::Entry::Vacant(entry) = rules.entry(advisory) else {
            return Err(GenerationError::Schema);
        };
        entry.insert(Decision {
            reason: reason.into(),
            matched: false,
        });
        Ok(())
    }

    /// Apply in the generation diagnostics callback, before displaying its report.
    /// Existing per-report stale path acknowledgements are preserved.
    pub fn apply(&mut self, report: &mut BulkLintReport) {
        report.warnings.retain(|warning| {
            let mut keep = true;
            for site in &warning.sites {
                if let Some(rule) = self
                    .rules
                    .get_mut(site)
                    .and_then(|rules| rules.get_mut(&warning.advisory))
                {
                    rule.matched = true;
                    keep = false;
                }
            }
            keep
        });
    }

    /// Unused declarations, advisory IDs and reasons after all modules have been visited.
    pub fn stale(&self) -> impl Iterator<Item = (&BulkSite, BulkAdvisory, &str)> {
        self.rules.iter().flat_map(|(site, rules)| {
            rules
                .iter()
                .filter(|(_, rule)| !rule.matched)
                .map(move |(id, rule)| (site, *id, rule.reason.as_str()))
        })
    }
}
