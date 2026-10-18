//! Explicit compile-pass and compile-fail contracts. Run the owning Make diagnostic target.
#![allow(dead_code, unused_imports, unexpected_cfgs)]
#[cfg(ui_case = "duplicate_field_id")]
mod duplicate_field_id;
#[cfg(ui_case = "duplicate_variant_id")]
mod duplicate_variant_id;
#[cfg(ui_case = "generic_contract")]
mod generic_contract;
#[cfg(ui_case = "missing_id")]
mod missing_id;
#[cfg(ui_case = "recursive_box")]
mod recursive_box;
mod support;
#[cfg(ui_case = "tuple_record")]
mod tuple_record;
#[cfg(ui_case = "unsupported_field")]
mod unsupported_field;
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
