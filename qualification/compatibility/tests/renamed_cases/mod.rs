use super::{limits, run_typescript};
use candidate::typescript::{PayloadLimits, SchemaLimits, ValidatedRegistry};
use candidate::{Encoding, decode, encode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, PartialEq, Serialize, Deserialize, candidate::typescript::Wire)]
#[wire(crate = "candidate")]
enum Choice {
    #[serde(rename = "ready-now")]
    Ready,
    #[serde(rename = "has-value")]
    Value(u16),
}
#[derive(Debug, PartialEq, Serialize, Deserialize, candidate::typescript::Wire)]
#[wire(crate = "candidate")]
struct Record {
    #[serde(rename = "__proto__")]
    flag: bool,
    #[serde(rename = "choice-value")]
    choice: Choice,
}

#[test]
fn explicit_serde_names_reach_generated_clients_without_changing_bytes() {
    let graph = ValidatedRegistry::from_type::<Record>("Record", SchemaLimits::default()).unwrap();
    let metadata: Value = serde_json::from_str(&graph.metadata_json().unwrap()).unwrap();
    assert_eq!(
        metadata["Record"],
        json!({"k":"record","fields":[
            {"name":"__proto__","value":{"k":"bool"}},
            {"name":"choice-value","value":{"k":"enum","variants":[
                {"name":"ready-now","id":0,"value":{"k":"unit"}},
                {"name":"has-value","id":1,"value":{"k":"u","bits":16}}
            ]}}
        ]})
    );
    let records = [
        Record {
            flag: false,
            choice: Choice::Ready,
        },
        Record {
            flag: true,
            choice: Choice::Value(128),
        },
    ];
    let literals = [vec![0, 0], vec![1, 1, 128, 1]];
    for (record, literal) in records.iter().zip(&literals) {
        assert_eq!(
            encode(record, Encoding::Payload, limits()).unwrap(),
            *literal
        );
    }
    let module = graph
        .payload_module(
            "codec",
            "Record",
            "./runtime-adapter.ts",
            PayloadLimits {
                bytes: 128,
                nodes: 512,
                depth: 32,
            },
        )
        .unwrap();
    let output = run_typescript(
        &json!({"module":module,"literals":literals}),
        include_str!("exchange.mjs"),
    );
    let output: Vec<Vec<u8>> = serde_json::from_value(output).unwrap();
    assert_eq!(output, literals);
    for (bytes, record) in output.iter().zip(&records) {
        assert_eq!(
            &decode::<Record>(bytes, Encoding::Payload, limits()).unwrap(),
            record
        );
    }
}
