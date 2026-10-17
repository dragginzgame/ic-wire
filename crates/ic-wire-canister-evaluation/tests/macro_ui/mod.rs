//! Explicit compile-pass and compile-fail contracts. Run the owning Make diagnostic target.
#![allow(dead_code, unused_imports, unexpected_cfgs)]
#[cfg(ui_case = "async_query")]
mod async_query;
#[cfg(ui_case = "async_response")]
mod async_response;
#[cfg(ui_case = "candid_selector")]
mod candid_selector;
#[cfg(ui_case = "export_name")]
mod export_name;
#[cfg(ui_case = "missing_guard")]
mod missing_guard;
#[cfg(ui_case = "request")]
mod request;
#[cfg(ui_case = "response")]
mod response;
mod support;
mod valid;
