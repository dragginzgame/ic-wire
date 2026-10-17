//! Small raw CDK endpoint helpers for consumer-owned IC Wire contracts.
//!
//! The codec crate stays independent of the CDK. This adapter owns argument
//! admission and replies only; consumers own exports, authorization, handlers,
//! lifecycle, replay policy and any external-await recovery.
pub mod endpoint;
