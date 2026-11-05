//! Standalone typed Wire canister: no Canic or Candid runtime dependency.
use ic_wire::{ByteBuf, Limits};
use std::cell::Cell;

/// Illustrative full-frame allowance, not a product default.
pub const LIMITS: Limits = Limits {
    bytes: 128,
    allocation: 8192,
    nodes: 32,
    depth: 8,
};
thread_local! { static COUNT: Cell<u64> = const { Cell::new(0) }; }

/// A typed record with an explicit binary field bound.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(ic_wire::Wire))]
pub struct Request {
    /// Application value.
    pub value: u64,
    /// Bounded binary data.
    pub bytes: ic_wire::Bytes<32>,
}

/// Record query, preserving a single typed value.
#[ic_wire_endpoint::query(contract = ECHO, id = 7, request = LIMITS, response = LIMITS)]
pub fn echo(request: Request) -> Request {
    request
}

/// Unit requests still require a complete identity-checked Wire frame.
#[ic_wire_endpoint::query(contract = COUNT_CONTRACT, id = 8, request = LIMITS, response = LIMITS)]
pub fn count() -> u64 {
    COUNT.get()
}

/// Synchronous mutation followed by bounded reply encoding.
#[ic_wire_endpoint::update(contract = MUTATE, id = 9, request = LIMITS, response = LIMITS)]
pub fn mutate(size: u32) -> ByteBuf {
    COUNT.set(COUNT.get() + 1);
    ByteBuf::from(vec![42; size as usize])
}

/// Explicit unit input/output and non-Rust export name.
#[ic_wire_endpoint::update(name = "unit-call", contract = UNIT, id = 10, request = LIMITS, response = LIMITS)]
pub fn unit(_: ()) {}

/// A normal async query that completes without inter-canister suspension.
#[ic_wire_endpoint::query(contract = ASYNC_QUERY, id = 11, request = LIMITS, response = LIMITS)]
pub async fn async_query(value: u64) -> (u64, bool) {
    (value, true)
}

/// Composite query with an actual inter-canister await through Platform.
#[ic_wire_endpoint::query(composite, contract = COMPOSITE, id = 12, request = LIMITS, response = LIMITS)]
pub async fn composite() -> u64 {
    remote_count().await
}

async fn remote_count() -> u64 {
    let frame = ic_wire::native::encode_request(8, &(), LIMITS).unwrap();
    let reply = ic_platform_runtime::cdk::call::Call::bounded_wait(
        ic_platform_runtime::cdk::api::canister_self(),
        "count",
    )
    .with_raw_args(&frame)
    .await
    .unwrap();
    ic_wire::native::decode_response(8, reply.as_ref(), LIMITS).unwrap()
}

/// After an await, only the callback mutation rolls back on encoding failure.
#[ic_wire_endpoint::update(id = 13, request = LIMITS, response = LIMITS)]
pub async fn async_mutate(size: u32) -> ByteBuf {
    COUNT.set(COUNT.get() + 10);
    let _ = remote_count().await;
    COUNT.set(COUNT.get() + 100);
    ByteBuf::from(vec![42; size as usize])
}

/// Async ready execution has the same rollback boundary as a synchronous call.
#[ic_wire_endpoint::update(id = 14, request = LIMITS, response = LIMITS)]
pub async fn async_ready(size: u32) -> ByteBuf {
    mutate(size)
}

/// A deliberately tight traversal budget, independent of frame bytes.
#[ic_wire_endpoint::query(contract = NODES, id = 15, request = Limits { nodes: 3, ..LIMITS }, response = LIMITS)]
pub fn nodes(values: Vec<u8>) -> u64 {
    values.len() as u64
}

/// Fully automatic identity and request/reply defaults.
#[ic_wire_endpoint::query(contract = DEFAULT_BLOB)]
pub fn default_blob(value: ByteBuf) -> ByteBuf {
    value
}

/// Bare attributes also validate unit frames.
#[ic_wire_endpoint::query]
pub fn default_unit() {}

/// A stable export name determines identity, independently of the Rust name.
#[ic_wire_endpoint::query(name = "stable-name", contract = NAMED, request = LIMITS)]
pub fn renamed(value: u64) -> u64 {
    value
}

/// A response-only override leaves request admission at its default.
#[ic_wire_endpoint::query(contract = RESPONSE_ONLY, response = LIMITS)]
pub fn response_only(value: u64) -> u64 {
    value
}

/// Default reply encoding retains synchronous transaction rollback.
#[ic_wire_endpoint::update(contract = DEFAULT_MUTATE)]
pub fn default_mutate(size: u32) -> ByteBuf {
    mutate(size)
}

// Framework composition: one Platform export, one Wire decoder, one existing
// guarded dispatch. Access refusal stays outside the declared response type.
fn composed_admit() -> Result<(), String> {
    ic_wire_endpoint::admit_request(LIMITS);
    Ok(())
}
fn composed_decode(bytes: Vec<u8>) -> bool {
    ic_wire_endpoint::decode_request(&bytes, 16, LIMITS)
}
fn guarded_handler(allowed: bool) -> Result<u64, &'static str> {
    if !allowed {
        return Err("denied");
    }
    COUNT.set(COUNT.get() + 1);
    Ok(COUNT.get())
}
#[ic_platform_runtime::cdk::update(
    guard = "composed_admit",
    decode_with = "composed_decode",
    manual_reply
)]
fn composed(allowed: bool) {
    match guarded_handler(allowed) {
        Ok(value) => ic_platform_runtime::cdk::api::msg_reply(ic_wire_endpoint::encode_response(
            &value, 16, LIMITS,
        )),
        Err(error) => ic_platform_runtime::cdk::api::msg_reject(error),
    }
}

#[cfg(test)]
mod tests;

/// A bounded view of the same representation as `Request`.
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(ic_wire::Wire))]
pub struct RequestRef<'a> {
    /// Application value.
    pub value: u64,
    /// Bytes borrowed from the retained argument buffer.
    #[serde(borrow)]
    pub bytes: ic_wire::BytesRef<'a, 32>,
}

/// A borrowed reply is encoded while the request buffer is still alive.
#[ic_wire_endpoint::query(borrow, contract = BORROWED, id = 17, request = LIMITS, response = LIMITS)]
pub fn borrowed_echo(value: RequestRef<'_>) -> RequestRef<'_> {
    value
}

/// Mutate only after borrowed request validation; an oversized reply traps.
#[ic_wire_endpoint::update(borrow, id = 18, request = LIMITS, response = LIMITS)]
pub fn borrowed_mutate(value: RequestRef<'_>) -> ByteBuf {
    COUNT.set(COUNT.get() + 1);
    ByteBuf::from(vec![42; value.value as usize])
}
