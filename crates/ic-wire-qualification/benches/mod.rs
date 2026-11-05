//! Unpublished allocation/borrowing qualification and compiler-diagnostic support.
//! Application transport and policy belong to consumers; the optional typed
//! endpoint integration is owned by the runtime crate.

/// Admission, borrowing, storage reuse and packed-hash experiment.
#[cfg(feature = "allocation-fixture")]
pub mod allocation;
