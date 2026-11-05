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
        ic_wire_endpoint::encode_response(
            &Measured(ic_wire_endpoint::decode_request::<T>(frame, id, LIMITS)),
            id,
            LIMITS,
        )
    }
}

#[cfg(feature = "ic")]
#[test]
#[ignore = "explicit installed instruction study"]
fn ic_cost() {
    measure_ic_cost(&["inline", "extracted"]);
}

#[cfg(feature = "ic")]
#[test]
#[ignore = "explicit metadata-dependency instruction study"]
fn metadata_dependency_cost() {
    measure_ic_cost(&["before", "after"]);
}

#[cfg(feature = "ic")]
fn measure_ic_cost(lanes: &[&str]) {
    use ic_platform_types::principal::Principal;
    use ic_testkit::pic::PocketIcStartupConfig;
    use ic_testkit::pocket_ic::PocketIcBuilder;
    let owned = PocketIcStartupConfig::from_env(std::time::Duration::from_secs(60))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pic = owned.instance();
    let mut rows = Vec::new();
    let mut sample_caller = 0_u64;
    for lane in lanes {
        let canister = pic.create_canister().unwrap();
        pic.add_cycles(canister, 100_000_000_000_000).unwrap();
        let wasm = std::fs::read(format!(
            "{}/{}.wasm",
            std::env::var("ENDPOINT_COST_DIR").unwrap(),
            lane
        ))
        .unwrap();
        pic.install_canister(canister, wasm, vec![], None).unwrap();
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
            let mut sample_caller_ids = Vec::new();
            for _ in 0..11 {
                // Fresh callers prevent PocketIC's query cache from replacing
                // repeated execution with a cached response/counter.
                sample_caller += 1;
                sample_caller_ids.push(sample_caller);
                let reply = pic
                    .query_call(
                        canister,
                        Principal::from_slice(&sample_caller.to_le_bytes()).into(),
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
                        native::decode_response::<(ByteBuf, ByteBuf)>(id, &reply, LIMITS).unwrap();
                    assert_eq!(actual, bytes);
                    count
                };
                samples.push(u64::from_le_bytes(count.as_slice().try_into().unwrap()));
            }
            rows.push(serde_json::json!({"length":length,"lane":lane,"request_bytes":frame.len(),"ic_instructions_to_counter_field":samples,"sample_caller_ids":sample_caller_ids}));
        }
    }
    std::fs::write(
        format!("{}/ic.json", std::env::var("ENDPOINT_COST_DIR").unwrap()),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
