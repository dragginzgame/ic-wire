//! Large-byte workload adapted from the pinned reference conformance example.
mod packed;
use super::run_typescript;
use candidate::typescript::{Method, PayloadLimits, SchemaLimits, ValidatedRegistry};
use candidate::{Bytes, Encoding, Limits, decode, encode};
use serde_json::json;

const MAX: usize = 1_048_576;

#[test]
fn large_bytes_cross_growth_and_length_prefix_boundaries() {
    let graph =
        ValidatedRegistry::from_type::<Bytes<MAX>>("Blob", SchemaLimits::default()).unwrap();
    let limits = PayloadLimits {
        bytes: (MAX + 3) as u64,
        nodes: 1,
        depth: 1,
    };
    let module = graph
        .payload_module("blob", "Blob", "./runtime-adapter.ts", limits)
        .unwrap();
    let frame = graph
        .method_module(
            &[Method {
                export: "blobFrame",
                id: 7,
                request: "Blob",
                response: "Blob",
                request_limits: limits,
                response_limits: limits,
            }],
            "./runtime-adapter.ts",
        )
        .unwrap();
    let sizes = [0, 63, 64, 127, 128, 16_383, 16_384, MAX];
    let output = run_typescript(
        &json!({"module":module,"frameModule":frame,"sizes":sizes}),
        include_str!("exchange.mjs"),
    );
    let payloads: Vec<Vec<u8>> = serde_json::from_value(output).unwrap();
    let policy = Limits {
        bytes: MAX + 3,
        allocation: 16 * MAX,
        nodes: 32,
        depth: 8,
    };
    assert_eq!(payloads.len(), sizes.len());
    for (payload, size) in payloads.iter().zip(sizes) {
        let value = Bytes::<MAX>::new((0..size).map(|i| (i % 251) as u8).collect()).unwrap();
        assert_eq!(*payload, encode(&value, Encoding::Payload, policy).unwrap());
        assert_eq!(
            decode::<Bytes<MAX>>(payload, Encoding::Payload, policy).unwrap(),
            value
        );
    }
}
