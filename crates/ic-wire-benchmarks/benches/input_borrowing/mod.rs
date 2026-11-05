//! Matched physical update probe for owned versus borrowed request decoding.
#![cfg(any(target_family = "wasm", test))]
#[cfg(any(test, not(feature = "endpoint-input"), not(feature = "borrowed-input")))]
use ic_wire::Bytes;
#[cfg(any(test, not(feature = "endpoint-input"), feature = "borrowed-input"))]
use ic_wire::BytesRef;
use ic_wire::Limits;
#[cfg(any(test, not(feature = "endpoint-input")))]
use ic_wire::native;
use serde::{Deserialize, Serialize};
const ID: u64 = 77;
const MAX: usize = 1048576;
const LIMITS: Limits = Limits {
    bytes: MAX + 64,
    allocation: 16 * MAX,
    nodes: 64,
    depth: 16,
};
#[derive(Serialize, Deserialize)]
#[cfg(any(test, not(feature = "endpoint-input"), not(feature = "borrowed-input")))]
struct Owned {
    mode: u8,
    nonce: u64,
    body: Bytes<MAX>,
}
#[derive(Serialize, Deserialize)]
#[cfg(any(test, not(feature = "endpoint-input"), feature = "borrowed-input"))]
struct Borrowed<'a> {
    mode: u8,
    nonce: u64,
    #[serde(borrow)]
    body: BytesRef<'a, MAX>,
}
fn consume(mode: u8, nonce: u64, bytes: &[u8]) -> [u64; 3] {
    let selected = match mode {
        0 => bytes,
        1 => &bytes[..bytes.len().min(256)],
        2 => &[],
        _ => panic!("invalid fixture mode"),
    };
    let sum = selected
        .iter()
        .fold(nonce, |sum, byte| sum.wrapping_add(u64::from(*byte)));
    [nonce, selected.len() as u64, sum]
}
#[cfg(any(test, not(feature = "endpoint-input")))]
fn owned(input: Vec<u8>) -> Result<[u64; 3], ic_wire::Error> {
    let value: Owned = native::decode_request(ID, &input, LIMITS)?;
    drop(input); // Fair owned control releases encoded storage before the handler.
    Ok(consume(value.mode, value.nonce, value.body.as_slice()))
}
#[cfg(any(test, not(feature = "endpoint-input")))]
fn borrowed(input: &[u8]) -> Result<[u64; 3], ic_wire::Error> {
    let value: Borrowed<'_> = native::decode_request(ID, input, LIMITS)?;
    Ok(consume(value.mode, value.nonce, value.body.as_slice()))
}
#[cfg(all(
    target_family = "wasm",
    feature = "endpoint-input",
    not(feature = "borrowed-input")
))]
#[ic_wire_endpoint::update(name = "consume", id = ID, request = LIMITS, response = LIMITS)]
fn owned_endpoint(value: Owned) -> [u64; 3] {
    consume(value.mode, value.nonce, value.body.as_slice())
}
#[cfg(all(
    target_family = "wasm",
    feature = "endpoint-input",
    feature = "borrowed-input"
))]
#[ic_wire_endpoint::update(borrow, name = "consume", id = ID, request = LIMITS, response = LIMITS)]
fn borrowed_endpoint(value: Borrowed<'_>) -> [u64; 3] {
    consume(value.mode, value.nonce, value.body.as_slice())
}
#[cfg(all(target_family = "wasm", not(feature = "endpoint-input")))]
#[expect(
    unsafe_code,
    reason = "This fixture owns the IC export; system calls use safe wrappers."
)]
#[unsafe(export_name = "canister_update consume")]
/// Same endpoint lifecycle and bytes; only request ownership differs.
pub extern "C" fn entry() {
    let length = ic_platform::ic0::msg_arg_data_size();
    assert!(length <= LIMITS.bytes);
    let mut input = vec![0; length];
    ic_platform::ic0::msg_arg_data_copy(&mut input, 0);
    let reply = if cfg!(feature = "borrowed-input") {
        let reply = borrowed(&input).unwrap();
        drop(input);
        reply
    } else {
        owned(input).unwrap()
    };
    let mut output = [0; 64];
    let bytes = native::encode_response_to_slice(ID, &reply, LIMITS, &mut output).unwrap();
    ic_platform::ic0::msg_reply_data_append(bytes);
    ic_platform::ic0::msg_reply();
}
#[cfg(test)]
mod tests;
