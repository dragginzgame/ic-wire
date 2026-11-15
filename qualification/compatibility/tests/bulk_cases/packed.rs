//! Paired packed/ordinary controls from the pinned reference packed-i16 workload.
use super::run_typescript;
use candidate::typescript::{Method, PayloadLimits, SchemaLimits, SchemaRegistry};
use candidate::{Encoding, Limits, List, PackedI16, decode, encode};
use serde_json::json;

const MAX: usize = 16_384;

#[test]
fn packed_and_ordinary_signed_buffers_preserve_distinct_representations() {
    let mut registry = SchemaRegistry::new(SchemaLimits::default()).unwrap();
    registry.register::<PackedI16<MAX>>("Packed").unwrap();
    registry.register::<List<i16, MAX>>("Ordinary").unwrap();
    let graph = registry.finish().unwrap();
    let packed = PayloadLimits {
        bytes: (2 * MAX + 3) as u64,
        nodes: 1,
        depth: 1,
    };
    let ordinary = PayloadLimits {
        bytes: (3 * MAX + 3) as u64,
        nodes: (MAX + 1) as u64,
        depth: 2,
    };
    let module = graph
        .payload_module("packedPayload", "Packed", "./runtime-adapter.ts", packed)
        .unwrap();
    let frame = graph
        .method_module(
            &[
                Method {
                    export: "packed",
                    id: 7,
                    request: "Packed",
                    response: "Packed",
                    request_limits: packed,
                    response_limits: packed,
                },
                Method {
                    export: "ordinary",
                    id: 8,
                    request: "Ordinary",
                    response: "Ordinary",
                    request_limits: ordinary,
                    response_limits: ordinary,
                },
            ],
            "./runtime-adapter.ts",
        )
        .unwrap();
    let sizes = [0, 1, 64, 128, 1024, MAX];
    let output = run_typescript(
        &json!({"module":module,"frameModule":frame,"sizes":sizes}),
        include_str!("packed.mjs"),
    );
    let payloads: Vec<[Vec<u8>; 2]> = serde_json::from_value(output).unwrap();
    let policy = Limits {
        bytes: 3 * MAX + 3,
        allocation: 4_194_304,
        // Rust charges weighted Serde visits, independently of JS value nodes.
        nodes: 131_072,
        depth: 8,
    };
    let mixed = [i16::MIN, -16385, -65, -64, -1, 0, 63, 64, 16384, i16::MAX];
    assert_eq!(payloads.len(), sizes.len() * 4);
    for (index, [packed_bytes, ordinary_bytes]) in payloads.iter().enumerate() {
        let size = sizes[index / 4];
        let values: Vec<i16> = (0..size)
            .map(|i| match index % 4 {
                0 => (i % 64) as i16,
                1 => i16::MAX,
                2 => -1 - (i % 64) as i16,
                _ => mixed[i % mixed.len()],
            })
            .collect();
        let packed_value = PackedI16::<MAX>::new(values.clone()).unwrap();
        let ordinary_value = List::<i16, MAX>::new(values).unwrap();
        assert_eq!(
            *packed_bytes,
            encode(&packed_value, Encoding::Payload, policy).unwrap()
        );
        assert_eq!(
            *ordinary_bytes,
            encode(&ordinary_value, Encoding::Payload, policy).unwrap()
        );
        assert_eq!(
            decode::<PackedI16<MAX>>(packed_bytes, Encoding::Payload, policy).unwrap(),
            packed_value
        );
        assert_eq!(
            decode::<List<i16, MAX>>(ordinary_bytes, Encoding::Payload, policy).unwrap(),
            ordinary_value
        );
    }
}
