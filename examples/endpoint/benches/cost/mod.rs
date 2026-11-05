//! Matched inline Platform hooks versus extracted Wire hooks; no Canic policies.
#[cfg(any(feature = "inline-baseline", test))]
use ic_wire::native;
use ic_wire::{ByteBuf, Limits};
use serde::{Serialize, ser::SerializeTuple};

const LIMITS: Limits = Limits {
    bytes: 131072,
    allocation: 8 * 1024 * 1024,
    nodes: 32,
    depth: 8,
};

struct Measured<T>(T);
impl<T: Serialize> Serialize for Measured<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.0)?;
        // Fixed-width bytes preserve the two-pass encoder's output length. The
        // second pass observes entry/copy/decode/handler and encoding up to here;
        // final counter serialization, msg_reply and cleanup are excluded.
        #[cfg(target_family = "wasm")]
        let count = ic_platform_runtime::ic0::performance_counter(0);
        #[cfg(not(target_family = "wasm"))]
        let count = 0u64;
        tuple.serialize_element(&ic_wire::BytesRef::<8>::new(&count.to_le_bytes()).unwrap())?;
        tuple.end()
    }
}

#[cfg_attr(not(feature = "inline-baseline"), ic_wire_endpoint::query(id = 21, request = LIMITS, response = LIMITS))]
#[cfg_attr(
    feature = "inline-baseline",
    ic_platform_runtime::cdk::query(
        guard = "admit",
        decode_with = "decode_blob",
        encode_with = "encode_blob"
    )
)]
fn echo(value: ByteBuf) -> Measured<ByteBuf> {
    Measured(value)
}

#[cfg_attr(not(feature = "inline-baseline"), ic_wire_endpoint::query(id = 22, request = LIMITS, response = LIMITS))]
#[cfg_attr(
    feature = "inline-baseline",
    ic_platform_runtime::cdk::query(
        guard = "admit",
        decode_with = "decode_unit",
        encode_with = "encode_unit"
    )
)]
fn unit(value: ()) -> Measured<()> {
    Measured(value)
}

// Qualification-only baseline: Canic's identity/limit/error handling expressed
// as inline hooks at Platform's maintained generator. No export/executor copy.
// This isolates extraction overhead, not a complete Canic migration speedup.
#[cfg(feature = "inline-baseline")]
fn admit() -> Result<(), String> {
    if ic_platform_runtime::ic0::msg_arg_data_size() > LIMITS.bytes {
        ic_platform_runtime::cdk::api::trap("Wire request exceeds byte allowance");
    }
    Ok(())
}
#[cfg(feature = "inline-baseline")]
fn decode_blob(bytes: Vec<u8>) -> ByteBuf {
    before_decode(&bytes, 21)
}
#[cfg(feature = "inline-baseline")]
fn encode_blob(value: Measured<ByteBuf>) -> Vec<u8> {
    before_encode(&value, 21)
}
#[cfg(feature = "inline-baseline")]
fn decode_unit(bytes: Vec<u8>) {
    before_decode(&bytes, 22)
}
#[cfg(feature = "inline-baseline")]
fn encode_unit(value: Measured<()>) -> Vec<u8> {
    before_encode(&value, 22)
}

#[cfg(any(feature = "inline-baseline", test))]
fn before_decode<T: serde::de::DeserializeOwned>(bytes: &[u8], id: u64) -> T {
    native::decode_request(id, bytes, LIMITS)
        .unwrap_or_else(|error| ic_platform_runtime::cdk::api::trap(error.to_string()))
}
#[cfg(any(feature = "inline-baseline", test))]
fn before_encode<T: Serialize>(value: &T, id: u64) -> Vec<u8> {
    native::encode_response(id, value, LIMITS)
        .unwrap_or_else(|error| ic_platform_runtime::cdk::api::trap(error.to_string()))
}

#[cfg(test)]
mod tests;
