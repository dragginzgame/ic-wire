//! Unpublished allocation/borrowing qualification and compiler-diagnostic support.
//! Transport and endpoint integration belong to consumers.

/// Admission, borrowing, storage reuse and packed-hash experiment.
#[cfg(feature = "allocation-fixture")]
pub mod allocation;
