use super::{limits, run_typescript};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wire::typescript::{PayloadLimits, SchemaLimits, ValidatedRegistry, Wire};
use wire::{ByteArray, ByteBuf, Encoding, decode, encode};
#[derive(Debug, PartialEq, Serialize, Deserialize, Wire)]
#[wire(crate = "wire", fixed)]
struct Envelope {
    plain: [u8; 2],
    exact: ByteArray<2>,
    buffer: ByteBuf,
    #[serde(with = "serde_bytes")]
    optional: Option<Vec<u8>>,
}
#[test]
fn published_byte_types_and_optional_adapter_keep_bulk_layout() {
    let graph =
        ValidatedRegistry::from_type::<Envelope>("Envelope", SchemaLimits::default()).unwrap();
    let module = graph
        .payload_module(
            "codec",
            "Envelope",
            "./runtime-adapter.ts",
            PayloadLimits {
                bytes: 64,
                nodes: 32,
                depth: 8,
            },
            &Default::default(),
            |report| eprint!("{report}"),
        )
        .unwrap();
    let literals = vec![
        vec![0, 255, 2, 0, 255, 2, 7, 8, 0],
        vec![0, 255, 2, 0, 255, 2, 7, 8, 1, 0],
        vec![0, 255, 2, 0, 255, 2, 7, 8, 1, 2, 3, 4],
    ];
    let output: Vec<Vec<u8>> = serde_json::from_value(run_typescript(
        &json!({"module":module,"literals":literals}),
        include_str!("bytes.mjs"),
    ))
    .unwrap();
    assert_eq!(output, literals);
    for (bytes, optional) in output.iter().zip([None, Some(vec![]), Some(vec![3, 4])]) {
        let value = Envelope {
            plain: [0, 255],
            exact: ByteArray::new([0, 255]),
            buffer: ByteBuf::from(vec![7, 8]),
            optional,
        };
        assert_eq!(encode(&value, Encoding::Payload, limits()).unwrap(), *bytes);
        assert_eq!(
            decode::<Envelope>(bytes, Encoding::Payload, limits()).unwrap(),
            value
        );
    }
}
