//! Explicit compile-pass and compile-fail contracts. Run the owning Make diagnostic target.
// Compile-fail cases stop at different phases; rustc and Cargo also supply
// different check-cfg sets. No expectation is fulfilled in every selected case.
#![allow(
    dead_code,
    unused_imports,
    unexpected_cfgs,
    reason = "Compile-pass/fail diagnostic inputs vary by ui_case and compiler phase."
)]
#[cfg(ui_case = "borrow_escape")]
mod borrow_escape;
#[cfg(ui_case = "borrow_mutation")]
mod borrow_mutation;
#[cfg(ui_case = "encode_only_decode")]
mod encode_only_decode;
#[cfg(ui_case = "native_schema_attributes")]
mod native_schema_attributes;
mod support;
mod valid;
#[cfg(ui_case = "wrong_async_handler")]
mod wrong_async_handler;
#[cfg(ui_case = "wrong_handler")]
mod wrong_handler;
#[cfg(ui_case = "wrong_handler_argument")]
mod wrong_handler_argument;
#[cfg(ui_case = "wrong_reply")]
mod wrong_reply;
#[cfg(ui_case = "wrong_request")]
mod wrong_request;
