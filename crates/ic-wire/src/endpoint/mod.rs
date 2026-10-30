//! Typed IC entrypoints, available only with the `endpoint` feature.
//!
//! Bare `query` and `update` derive an identity from the exported method name and
//! use [`DEFAULT_REQUEST_LIMITS`] / [`DEFAULT_RESPONSE_LIMITS`]. Optional `id`,
//! `request` and `response` constant expressions override those choices.
//! A handler takes zero arguments (a framed unit request) or one owned Serde value.
//! Synchronous handlers can select `borrow` with one request: borrowed fields and
//! replies refer to Platform's argument vector, retained through reply encoding.
//! This avoids a second field copy; the initial ingress copy is still required.
//! Async handlers remain owned. Framing, budgets and validation are unchanged.
//! `query(composite, ...)` selects a composite query. Async handlers use Platform's
//! protected executor. `name = "..."` overrides the export name; `crate = "path"`
//! selects a renamed/re-exported Wire crate.
//!
//! Optional `contract = NAME` exposes a typed [`Contract`] constant taken from the
//! exact signature and settings. Its native schema methods delegate to existing
//! Wire projection; no automatic registry, custom Wasm metadata section or
//! artifact admission is installed.
//!
//! Encoding failures trap, rolling back a synchronous update's mutations.
//! Async updates follow ordinary IC transactions: state committed before an
//! inter-canister await cannot be rolled back by a later reply-encoding trap.
//! Only the current callback's changes roll back. Query state is not persisted.
//!
//! Frameworks may instead pass the helpers below to Platform's codec hooks around
//! their existing guarded handlers. Admission must run as a Platform `guard`
//! before its argument copy. Framework access refusals, dispatch, registration and
//! final artifact checks remain framework-owned; do not stack two export macros.

use crate::{Limits, native};
use alloc::{string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};

pub use ic_wire_derive::{query, update};

/// Default full-frame request limits for the optional endpoint integration.
///
/// 64 KiB of frame bytes, 8 MiB conservative traversal allowance, 65,536 charged
/// value visits and depth 32. The allocation allowance excludes the ingress copy;
/// it is not a measured heap cap. Field bounds and application validation still
/// apply. These are endpoint policy, not implicit limits in the core codec.
pub const DEFAULT_REQUEST_LIMITS: Limits = Limits {
    bytes: 64 * 1024,
    allocation: 8 * 1024 * 1024,
    nodes: 65_536,
    depth: 32,
};

/// Default full-frame response limits, with the same values as the request defaults.
/// Request and response overrides remain independent; reply failure still traps.
pub const DEFAULT_RESPONSE_LIMITS: Limits = DEFAULT_REQUEST_LIMITS;

/// A handler's types and explicit frame settings, without a registry or transport.
///
/// Prefer the attribute's `contract = NAME` output to a separately maintained
/// declaration. Native schemas do not prove target-specific type-layout parity;
/// the consumer still owns final artifact admission.
pub struct Contract<Request, Response> {
    /// IC method name.
    pub name: &'static str,
    /// IC mode: `query`, `composite_query`, or `update`.
    pub mode: &'static str,
    /// Native frame method identity.
    pub id: u64,
    /// Request limits, including the 11-byte frame header.
    pub request: Limits,
    /// Response limits, including the 11-byte frame header.
    pub response: Limits,
    types: core::marker::PhantomData<fn(Request) -> Response>,
}

impl<Request, Response> Contract<Request, Response> {
    /// Check frame size, identity and direction before copying the full ingress.
    /// Run as a Platform guard; payload validation still occurs during decoding.
    pub fn admit_request(&self) {
        admit_frame(self.id, self.request);
    }

    /// Construct the declaration used by the endpoint attribute.
    #[doc(hidden)]
    pub const fn new(
        name: &'static str,
        mode: &'static str,
        id: u64,
        request: Limits,
        response: Limits,
    ) -> Self {
        Self {
            name,
            mode,
            id,
            request,
            response,
            types: core::marker::PhantomData,
        }
    }
}

#[cfg(all(feature = "typescript", not(target_family = "wasm")))]
impl<Request: crate::typescript::Wire, Response: crate::typescript::Wire>
    Contract<Request, Response>
{
    /// Project the actual request type using the existing native schema owner.
    pub fn request_schema(&self) -> Result<serde_json::Value, crate::typescript::GenerationError> {
        crate::typescript::native_schema::<Request>()
    }

    /// Project the actual response type using the existing native schema owner.
    pub fn response_schema(&self) -> Result<serde_json::Value, crate::typescript::GenerationError> {
        crate::typescript::native_schema::<Response>()
    }
}

/// Platform paths used by generated entrypoints, including renamed Wire imports.
#[doc(hidden)]
pub mod __private {
    pub use ic_platform::cdk;
}

/// Trap on an oversized ingress before Platform allocates/copies argument bytes.
pub fn admit_request(limits: Limits) {
    admit_request_len(limits);
}

fn admit_request_len(limits: Limits) -> usize {
    // This moves Canic's pre-copy check to Wire without changing its acceptance
    // rule: the full native frame counts toward bytes, including its 11-byte header.
    // Consumers retain their existing method IDs and limits; no wire migration.
    let length = ic_platform::ic0::msg_arg_data_size();
    if length > limits.bytes {
        ic_platform::cdk::api::trap("Wire request exceeds byte allowance");
    }
    length
}

#[cfg_attr(target_arch = "wasm32", inline(never))]
fn admit_frame(id: u64, limits: Limits) {
    let length = admit_request_len(limits);
    if length < crate::contract::HEADER {
        trap_error(crate::Error::Malformed);
    }
    let mut header = [0; crate::contract::HEADER];
    ic_platform::ic0::msg_arg_data_copy(&mut header, 0);
    // Previously identity/direction were checked after copying the complete
    // ingress. Check the same eleven bytes first, using the same validator.
    // Bytes and accepted frames are unchanged; no consumer migration. Payload
    // validation remains mandatory, including the existing decoder header check.
    crate::contract::payload(id, 0, &header).unwrap_or_else(|error| trap_error(error));
}

/// Decode a complete native request frame, trapping on typed codec failure.
///
/// Call [`admit_request`] before obtaining the argument buffer. This function
/// preserves native identity, traversal, primitive and complete-input validation.
/// Borrowed fields refer to `bytes`; the caller must retain it through their use.
pub fn decode_request<'de, T: Deserialize<'de>>(bytes: &'de [u8], id: u64, limits: Limits) -> T {
    native::decode_request(id, bytes, limits).unwrap_or_else(|error| trap_error(error))
}

/// Encode the native response frame within its allowances, trapping on failure.
///
/// A trap (rather than a rejection or error reply) preserves synchronous mutation
/// rollback. Earlier async transactions remain committed; see the module docs.
pub fn encode_response<T: Serialize + ?Sized>(value: &T, id: u64, limits: Limits) -> Vec<u8> {
    native::encode_response(id, value, limits).unwrap_or_else(|error| trap_error(error))
}

// Keep failure formatting shared across request/reply types and admission.
#[cold]
#[inline(never)]
fn trap_error(error: crate::Error) -> ! {
    ic_platform::cdk::api::trap(error.to_string())
}
