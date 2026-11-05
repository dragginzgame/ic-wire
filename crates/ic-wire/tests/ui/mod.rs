//! Explicit compile-pass and compile-fail contracts. Run the owning Make diagnostic target.
#![expect(
    dead_code,
    reason = "Compiler inputs define handlers without executing them."
)]
// Cargo supplies check-cfg for its test target; direct diagnostic rustc does not.
#![cfg_attr(
    test,
    expect(
        unexpected_cfgs,
        reason = "ui_case is selected by the direct compiler diagnostic driver."
    )
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
