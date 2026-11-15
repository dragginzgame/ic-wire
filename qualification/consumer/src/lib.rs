//! Portable application-owned contract for a bounded sample upload.
//! No transport, persistence, SDK, authentication or endpoint execution.
#![no_std]
extern crate alloc;
use alloc::{string::String, vec::Vec};
#[cfg(all(feature = "candidate", feature = "reference"))]
compile_error!("select exactly one Wire supplier");
#[cfg(not(any(feature = "candidate", feature = "reference")))]
compile_error!("select one Wire supplier");
#[cfg(feature = "candidate")]
use candidate as wire;
#[cfg(all(feature = "reference", not(feature = "candidate")))]
use reference as wire;
use serde::{Deserialize, Serialize};
pub use wire::{Bytes, Error, Limits, List, Text};

/// Application sample field, instantiated independently for each supplier.
#[cfg(feature = "candidate")]
pub type Samples = wire::Packed<i16, 64>;
/// Reference supplier for the same application sample field.
#[cfg(all(feature = "reference", not(feature = "candidate")))]
pub type Samples = wire::PackedI16<64>;

/// Frozen upload identity, independent of crate version.
pub const METHOD: u64 = 7;
/// Complete request frame and native traversal allowances.
pub const REQUEST_LIMITS: Limits = Limits {
    bytes: 1400,
    allocation: 65536,
    nodes: 1024,
    depth: 16,
};
/// Complete response frame and native traversal allowances.
pub const RESPONSE_LIMITS: Limits = Limits {
    bytes: 32,
    allocation: 1024,
    nodes: 32,
    depth: 8,
};

/// Application's explicit field representations; ordinary tags stay a list.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "schema", not(target_family = "wasm")),
    derive(wire::typescript::Wire)
)]
#[cfg_attr(
    all(feature = "schema", not(target_family = "wasm")),
    wire(crate = "wire")
)]
pub struct Upload {
    /// Expected application revision.
    pub revision: u64,
    /// UTF-8 application key.
    pub key: Text<32>,
    /// Opaque content, capped at 1 KiB.
    pub content: Bytes<1024>,
    /// Explicit packed little-endian signed samples.
    pub samples: Samples,
    /// Ordinary Postcard integer tags.
    pub tags: List<u16, 8>,
}
/// Application reply, distinct from the upload shape.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "schema", not(target_family = "wasm")),
    derive(wire::typescript::Wire)
)]
#[cfg_attr(
    all(feature = "schema", not(target_family = "wasm")),
    wire(crate = "wire")
)]
pub struct Receipt {
    /// Committed revision reported by the application.
    pub revision: u64,
    /// Application result; Wire supplies no commit semantics.
    pub accepted: bool,
}

/// Representative request used identically by both supplier builds.
/// # Errors
/// Returns a field-bound error if the fixed fixture ceases to fit its contract.
pub fn sample_upload() -> Result<Upload, Error> {
    Ok(Upload {
        revision: 128,
        key: Text::new(String::from("sensor"))?,
        content: Bytes::new(alloc::vec![0, 255])?,
        samples: Samples::new(alloc::vec![-32768, 0, 32767])?,
        tags: List::new(alloc::vec![1, 128])?,
    })
}

/// Encode this contract's request, including identity and request direction.
/// # Errors
/// Returns the selected supplier's typed admission or serialization refusal.
pub fn encode_upload(value: &Upload) -> Result<Vec<u8>, Error> {
    #[cfg(feature = "candidate")]
    {
        wire::encode(
            value,
            wire::Encoding::Frame {
                method: METHOD,
                direction: wire::Direction::Request,
            },
            REQUEST_LIMITS,
        )
    }
    #[cfg(all(feature = "reference", not(feature = "candidate")))]
    {
        wire::native::encode_request(METHOD, value, REQUEST_LIMITS)
    }
}
/// Decode a complete request using the fixed application policy.
/// # Errors
/// Returns identity, budget or malformed-input refusal.
pub fn decode_upload(bytes: &[u8]) -> Result<Upload, Error> {
    #[cfg(feature = "candidate")]
    {
        wire::decode(
            bytes,
            wire::Encoding::Frame {
                method: METHOD,
                direction: wire::Direction::Request,
            },
            REQUEST_LIMITS,
        )
    }
    #[cfg(all(feature = "reference", not(feature = "candidate")))]
    {
        wire::native::decode_request(METHOD, bytes, REQUEST_LIMITS)
    }
}
/// Encode this contract's response with its independent budget.
/// # Errors
/// Returns the selected supplier's typed admission or serialization refusal.
pub fn encode_receipt(value: &Receipt) -> Result<Vec<u8>, Error> {
    #[cfg(feature = "candidate")]
    {
        wire::encode(
            value,
            wire::Encoding::Frame {
                method: METHOD,
                direction: wire::Direction::Response,
            },
            RESPONSE_LIMITS,
        )
    }
    #[cfg(all(feature = "reference", not(feature = "candidate")))]
    {
        wire::native::encode_response(METHOD, value, RESPONSE_LIMITS)
    }
}
/// Decode a complete response; no retry or state interpretation is supplied.
/// # Errors
/// Returns identity, budget or malformed-input refusal.
pub fn decode_receipt(bytes: &[u8]) -> Result<Receipt, Error> {
    #[cfg(feature = "candidate")]
    {
        wire::decode(
            bytes,
            wire::Encoding::Frame {
                method: METHOD,
                direction: wire::Direction::Response,
            },
            RESPONSE_LIMITS,
        )
    }
    #[cfg(all(feature = "reference", not(feature = "candidate")))]
    {
        wire::native::decode_response(METHOD, bytes, RESPONSE_LIMITS)
    }
}

/// Generate typed payload and method clients from the application types.
/// # Errors
/// Returns schema or bounded-output admission errors.
#[cfg(all(feature = "schema", not(target_family = "wasm")))]
pub fn clients(
    runtime: &str,
    mut diagnostics: impl FnMut(wire::typescript::BulkLintReport),
) -> Result<(String, String), wire::typescript::GenerationError> {
    use wire::typescript::{Method, PayloadLimits, SchemaLimits, SchemaRegistry};
    let mut registry = SchemaRegistry::new(SchemaLimits::default())?;
    registry.register::<Upload>("Upload")?;
    registry.register::<Receipt>("Receipt")?;
    let graph = registry.finish()?;
    let request = PayloadLimits {
        bytes: 1389,
        nodes: 32,
        depth: 8,
    };
    let response = PayloadLimits {
        bytes: 21,
        nodes: 3,
        depth: 2,
    };
    Ok((
        graph.payload_module(
            "uploadPayload",
            "Upload",
            runtime,
            request,
            &Default::default(),
            &mut diagnostics,
        )?,
        graph.method_module(
            &[Method {
                export: "upload",
                id: METHOD,
                request: "Upload",
                response: "Receipt",
                request_limits: request,
                response_limits: response,
            }],
            runtime,
            &Default::default(),
            &mut diagnostics,
        )?,
    ))
}
