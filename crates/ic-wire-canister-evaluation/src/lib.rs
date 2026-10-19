//! Unpublished native codec controls and compiler-diagnostic support.
//! Transport and endpoint integration belong to consumers.

/// Private measurement cases; alternate codecs are controls, never runtime modes.
#[cfg(feature = "performance-fixture")]
pub mod performance;

/// Maintainer-requested raw MessagePack/Postcard experiment and bounded Wire control.
#[cfg(feature = "comparison")]
pub mod messagepack;
