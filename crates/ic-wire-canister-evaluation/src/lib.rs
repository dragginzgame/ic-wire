//! Unpublished IC Wire integration fixtures; not the IC Wire runtime.
//!
//! Raw, zero-argument entrypoints use manual replies and `hidden = true` to avoid
//! generating Candid service metadata. Their callable Wasm exports stay intact.
/// Evaluation-only Canic-shaped macro integration fixture.
#[cfg(feature = "macro-fixture")]
pub mod macro_fixture;

/// Shared runtime contracts used by the installed adoption trial.
#[cfg(feature = "browser-fixture")]
pub mod commands;

#[cfg(feature = "browser-fixture")]
mod browser;
