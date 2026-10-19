//! Unpublished native codec controls and compiler-diagnostic support.
//! Transport and endpoint integration belong to consumers.

/// Private measurement cases; alternate codecs are controls, never runtime modes.
#[cfg(feature = "performance-fixture")]
pub mod performance;
