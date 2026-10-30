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
#[ic_wire::endpoint::query(contract = ECHO, id = 7, request = LIMITS, response = LIMITS)]
pub fn echo(request: Request) -> Request {
    request
}

/// Unit requests still require a complete identity-checked Wire frame.
#[ic_wire::endpoint::query(contract = COUNT_CONTRACT, id = 8, request = LIMITS, response = LIMITS)]
pub fn count() -> u64 {
    COUNT.get()
}

/// Synchronous mutation followed by bounded reply encoding.
#[ic_wire::endpoint::update(contract = MUTATE, id = 9, request = LIMITS, response = LIMITS)]
pub fn mutate(size: u32) -> ByteBuf {
    COUNT.set(COUNT.get() + 1);
    ByteBuf::from(vec![42; size as usize])
}

/// Explicit unit input/output and non-Rust export name.
#[ic_wire::endpoint::update(name = "unit-call", contract = UNIT, id = 10, request = LIMITS, response = LIMITS)]
pub fn unit(_: ()) {}

/// A normal async query that completes without inter-canister suspension.
#[ic_wire::endpoint::query(contract = ASYNC_QUERY, id = 11, request = LIMITS, response = LIMITS)]
pub async fn async_query(value: u64) -> (u64, bool) {
    (value, true)
}

/// Composite query with an actual inter-canister await through Platform.
#[ic_wire::endpoint::query(composite, contract = COMPOSITE, id = 12, request = LIMITS, response = LIMITS)]
pub async fn composite() -> u64 {
    remote_count().await
}

async fn remote_count() -> u64 {
    let frame = ic_wire::native::encode_request(8, &(), LIMITS).unwrap();
    let reply =
        ic_platform::cdk::call::Call::bounded_wait(ic_platform::cdk::api::canister_self(), "count")
            .with_raw_args(&frame)
            .await
            .unwrap();
    ic_wire::native::decode_response(8, reply.as_ref(), LIMITS).unwrap()
}

/// After an await, only the callback mutation rolls back on encoding failure.
#[ic_wire::endpoint::update(id = 13, request = LIMITS, response = LIMITS)]
pub async fn async_mutate(size: u32) -> ByteBuf {
    COUNT.set(COUNT.get() + 10);
    let _ = remote_count().await;
    COUNT.set(COUNT.get() + 100);
    ByteBuf::from(vec![42; size as usize])
}

/// Async ready execution has the same rollback boundary as a synchronous call.
#[ic_wire::endpoint::update(id = 14, request = LIMITS, response = LIMITS)]
pub async fn async_ready(size: u32) -> ByteBuf {
    mutate(size)
}

/// A deliberately tight traversal budget, independent of frame bytes.
#[ic_wire::endpoint::query(contract = NODES, id = 15, request = Limits { nodes: 3, ..LIMITS }, response = LIMITS)]
pub fn nodes(values: Vec<u8>) -> u64 {
    values.len() as u64
}

/// Fully automatic identity and request/reply defaults.
#[ic_wire::endpoint::query(contract = DEFAULT_BLOB)]
pub fn default_blob(value: ByteBuf) -> ByteBuf {
    value
}

/// Bare attributes also validate unit frames.
#[ic_wire::endpoint::query]
pub fn default_unit() {}

/// A stable export name determines identity, independently of the Rust name.
#[ic_wire::endpoint::query(name = "stable-name", contract = NAMED, request = LIMITS)]
pub fn renamed(value: u64) -> u64 {
    value
}

/// A response-only override leaves request admission at its default.
#[ic_wire::endpoint::query(contract = RESPONSE_ONLY, response = LIMITS)]
pub fn response_only(value: u64) -> u64 {
    value
}

/// Default reply encoding retains synchronous transaction rollback.
#[ic_wire::endpoint::update(contract = DEFAULT_MUTATE)]
pub fn default_mutate(size: u32) -> ByteBuf {
    mutate(size)
}

// Framework composition: one Platform export, one Wire decoder, one existing
// guarded dispatch. Access refusal stays outside the declared response type.
fn composed_admit() -> Result<(), String> {
    ic_wire::endpoint::admit_request(LIMITS);
    Ok(())
}
fn composed_decode(bytes: Vec<u8>) -> bool {
    ic_wire::endpoint::decode_request(&bytes, 16, LIMITS)
}
fn guarded_handler(allowed: bool) -> Result<u64, &'static str> {
    if !allowed {
        return Err("denied");
    }
    COUNT.set(COUNT.get() + 1);
    Ok(COUNT.get())
}
#[ic_platform::cdk::update(
    guard = "composed_admit",
    decode_with = "composed_decode",
    manual_reply
)]
fn composed(allowed: bool) {
    match guarded_handler(allowed) {
        Ok(value) => {
            ic_platform::cdk::api::msg_reply(ic_wire::endpoint::encode_response(&value, 16, LIMITS))
        }
        Err(error) => ic_platform::cdk::api::msg_reject(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_fields_and_reply_retain_input_storage() {
        let value = Request {
            value: 5,
            bytes: ic_wire::Bytes::new(vec![1, 2]).unwrap(),
        };
        let frame = ic_wire::native::encode_request(17, &value, LIMITS).unwrap();
        let view: RequestRef<'_> = ic_wire::endpoint::decode_request(&frame, 17, LIMITS);
        assert_eq!(
            view.bytes.as_slice().as_ptr(),
            frame[frame.len() - 2..].as_ptr()
        );
        let reply = borrowed_echo(view);
        assert_eq!(
            ic_wire::endpoint::encode_response(&reply, 17, LIMITS),
            ic_wire::native::encode_response(17, &value, LIMITS).unwrap()
        );
        #[cfg(feature = "schema")]
        {
            assert_eq!(
                BORROWED.request_schema().unwrap(),
                ECHO.request_schema().unwrap()
            );
            assert_eq!(
                BORROWED.response_schema().unwrap(),
                ECHO.response_schema().unwrap()
            );
        }
    }

    #[test]
    fn defaults_and_overrides_bind_the_actual_contract() {
        use ic_wire::endpoint::{DEFAULT_REQUEST_LIMITS, DEFAULT_RESPONSE_LIMITS};
        let fields = |value: Limits| (value.bytes, value.allocation, value.nodes, value.depth);
        assert_eq!(
            fields(DEFAULT_REQUEST_LIMITS),
            (65_536, 8_388_608, 65_536, 32)
        );
        assert_eq!(
            fields(DEFAULT_RESPONSE_LIMITS),
            fields(DEFAULT_REQUEST_LIMITS)
        );
        assert_eq!(fields(DEFAULT_BLOB.request), fields(DEFAULT_REQUEST_LIMITS));
        assert_eq!(
            fields(DEFAULT_BLOB.response),
            fields(DEFAULT_RESPONSE_LIMITS)
        );
        assert_eq!(DEFAULT_BLOB.id, 3_695_642_065_081_963_175);
        assert_eq!(NAMED.id, 11_911_248_233_000_959_399);
        assert_eq!(NAMED.name, "stable-name");
        assert_eq!(fields(NAMED.request), fields(LIMITS));
        assert_eq!(fields(NAMED.response), fields(DEFAULT_RESPONSE_LIMITS));
        assert_eq!(
            fields(RESPONSE_ONLY.request),
            fields(DEFAULT_REQUEST_LIMITS)
        );
        assert_eq!(fields(RESPONSE_ONLY.response), fields(LIMITS));
        assert_eq!(ECHO.id, 7, "explicit identity remains authoritative");
        assert_eq!(
            DEFAULT_BLOB.request_schema().unwrap(),
            ic_wire::typescript::native_schema::<ByteBuf>().unwrap()
        );
        assert_eq!(
            DEFAULT_BLOB.response_schema().unwrap(),
            DEFAULT_BLOB.request_schema().unwrap()
        );
    }

    #[test]
    fn declaration_matches_types_constants_and_existing_projection() {
        // Actual aliases are accepted by the actual functions, including unit/tuple.
        let request: Request = Request {
            value: 5,
            bytes: ic_wire::Bytes::new(vec![1, 2]).unwrap(),
        };
        let result: Request = echo(request.clone());
        assert_eq!(request, result);
        let _: () = unit(());
        assert_eq!(ECHO.id, 7);
        assert_eq!(ECHO.name, "echo");
        assert_eq!(ECHO.mode, "query");
        assert_eq!(COMPOSITE.mode, "composite_query");
        assert_eq!(MUTATE.mode, "update");
        assert_eq!(UNIT.name, "unit-call");
        let fields = |value: Limits| (value.bytes, value.allocation, value.nodes, value.depth);
        assert_eq!(fields(ECHO.request), fields(LIMITS));
        assert_eq!(fields(ECHO.response), fields(LIMITS));
        assert_eq!(NODES.request.nodes, 3);
        #[cfg(feature = "schema")]
        {
            assert_eq!(
                ECHO.request_schema().unwrap(),
                ic_wire::typescript::native_schema::<Request>().unwrap()
            );
            assert_eq!(
                ECHO.response_schema().unwrap(),
                ECHO.request_schema().unwrap()
            );
        }
        assert_eq!(
            COUNT_CONTRACT.request_schema().unwrap(),
            ic_wire::typescript::native_schema::<()>().unwrap()
        );
        assert_eq!(
            ASYNC_QUERY.response_schema().unwrap(),
            ic_wire::typescript::native_schema::<(u64, bool)>().unwrap()
        );
        // Concrete pre-existing frame oracle: IW, little-endian ID, direction, value.
        assert_eq!(
            ic_wire::native::encode_request(7, &42u8, LIMITS).unwrap(),
            [73, 87, 7, 0, 0, 0, 0, 0, 0, 0, 0, 42]
        );
    }
}

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
#[ic_wire::endpoint::query(borrow, contract = BORROWED, id = 17, request = LIMITS, response = LIMITS)]
pub fn borrowed_echo(value: RequestRef<'_>) -> RequestRef<'_> {
    value
}

/// Mutate only after borrowed request validation; an oversized reply traps.
#[ic_wire::endpoint::update(borrow, id = 18, request = LIMITS, response = LIMITS)]
pub fn borrowed_mutate(value: RequestRef<'_>) -> ByteBuf {
    COUNT.set(COUNT.get() + 1);
    ByteBuf::from(vec![42; value.value as usize])
}
