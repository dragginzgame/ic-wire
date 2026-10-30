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
        let count = ic_platform::ic0::performance_counter(0);
        #[cfg(not(target_family = "wasm"))]
        let count = 0u64;
        tuple.serialize_element(&ic_wire::BytesRef::<8>::new(&count.to_le_bytes()).unwrap())?;
        tuple.end()
    }
}

#[cfg_attr(not(feature = "inline-baseline"), ic_wire::endpoint::query(id = 21, request = LIMITS, response = LIMITS))]
#[cfg_attr(
    feature = "inline-baseline",
    ic_platform::cdk::query(
        guard = "admit",
        decode_with = "decode_blob",
        encode_with = "encode_blob"
    )
)]
fn echo(value: ByteBuf) -> Measured<ByteBuf> {
    Measured(value)
}

#[cfg_attr(not(feature = "inline-baseline"), ic_wire::endpoint::query(id = 22, request = LIMITS, response = LIMITS))]
#[cfg_attr(
    feature = "inline-baseline",
    ic_platform::cdk::query(
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
    if ic_platform::ic0::msg_arg_data_size() > LIMITS.bytes {
        ic_platform::cdk::api::trap("Wire request exceeds byte allowance");
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
        .unwrap_or_else(|error| ic_platform::cdk::api::trap(error.to_string()))
}
#[cfg(any(feature = "inline-baseline", test))]
fn before_encode<T: Serialize>(value: &T, id: u64) -> Vec<u8> {
    native::encode_response(id, value, LIMITS)
        .unwrap_or_else(|error| ic_platform::cdk::api::trap(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{hint::black_box, time::Instant};

    #[test]
    #[ignore = "explicit native timing study"]
    fn native_cost() {
        let mut rows = Vec::new();
        for length in [None, Some(0), Some(32), Some(65536)] {
            let bytes = ByteBuf::from(vec![0xa5; length.unwrap_or(0)]);
            let (id, frame) = if length.is_none() {
                (22, native::encode_request(22, &(), LIMITS).unwrap())
            } else {
                (21, native::encode_request(21, &bytes, LIMITS).unwrap())
            };
            for lane in ["inline", "extracted"] {
                let mut samples = Vec::new();
                let repetitions = if length == Some(65536) { 100 } else { 10000 };
                for _ in 0..11 {
                    let start = Instant::now();
                    for _ in 0..repetitions {
                        if id == 22 {
                            black_box(operation::<()>(lane, black_box(&frame), id));
                        } else {
                            black_box(operation::<ByteBuf>(lane, black_box(&frame), id));
                        }
                    }
                    samples.push(start.elapsed().as_nanos() as f64 / f64::from(repetitions));
                }
                rows.push(serde_json::json!({"length":length,"lane":lane,"repetitions":repetitions,"native_ns":samples}));
            }
        }
        std::fs::write(
            std::env::var_os("ENDPOINT_NATIVE_REPORT").unwrap(),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
    fn operation<T: serde::de::DeserializeOwned + Serialize>(
        lane: &str,
        frame: &[u8],
        id: u64,
    ) -> Vec<u8> {
        if lane == "inline" {
            before_encode(&Measured(before_decode::<T>(frame, id)), id)
        } else {
            ic_wire::endpoint::encode_response(
                &Measured(ic_wire::endpoint::decode_request::<T>(frame, id, LIMITS)),
                id,
                LIMITS,
            )
        }
    }

    #[cfg(feature = "ic")]
    #[test]
    #[ignore = "explicit installed instruction study"]
    fn ic_cost() {
        use ic_platform::{pocket_ic::PocketIcBuilder, principal::Principal};
        use ic_testkit::pic::PocketIcStartupConfig;
        let owned = PocketIcStartupConfig::from_env(std::time::Duration::from_secs(60))
            .unwrap()
            .build_instance(PocketIcBuilder::new().with_application_subnet())
            .unwrap();
        let pic = owned.instance();
        let mut rows = Vec::new();
        for lane in ["inline", "extracted"] {
            let canister = pic.create_canister();
            pic.add_cycles(canister, 100_000_000_000_000);
            let wasm = std::fs::read(format!(
                "{}/{}.wasm",
                std::env::var("ENDPOINT_COST_DIR").unwrap(),
                lane
            ))
            .unwrap();
            pic.install_canister(canister, wasm, vec![], None);
            for length in [None, Some(0), Some(32), Some(65536)] {
                let bytes = ByteBuf::from(vec![0xa5; length.unwrap_or(0)]);
                let (method, id, frame) = if length.is_none() {
                    ("unit", 22, native::encode_request(22, &(), LIMITS).unwrap())
                } else {
                    (
                        "echo",
                        21,
                        native::encode_request(21, &bytes, LIMITS).unwrap(),
                    )
                };
                let mut samples = Vec::new();
                for _ in 0..11 {
                    let reply = pic
                        .query_call(
                            canister,
                            Principal::anonymous().into(),
                            method,
                            frame.clone(),
                        )
                        .unwrap();
                    let count = if length.is_none() {
                        native::decode_response::<((), ByteBuf)>(id, &reply, LIMITS)
                            .unwrap()
                            .1
                    } else {
                        let (actual, count) =
                            native::decode_response::<(ByteBuf, ByteBuf)>(id, &reply, LIMITS)
                                .unwrap();
                        assert_eq!(actual, bytes);
                        count
                    };
                    samples.push(u64::from_le_bytes(count.as_slice().try_into().unwrap()));
                }
                rows.push(serde_json::json!({"length":length,"lane":lane,"request_bytes":frame.len(),"ic_instructions_to_counter_field":samples}));
            }
        }
        std::fs::write(
            format!("{}/ic.json", std::env::var("ENDPOINT_COST_DIR").unwrap()),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}
