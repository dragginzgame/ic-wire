//! Candidate Rust exchange with the published reference TypeScript runtime.

use candidate::typescript::{SchemaLimits, ValidatedRegistry};
use candidate::{Bytes, Direction, Encoding, Limits, List, PackedI16, decode, encode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Choice {
    Ready,
    Value(u16),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Record {
    number: u8,
    choice: Choice,
    optional: Option<u8>,
    bytes: Bytes<3>,
    packed: PackedI16<2>,
    ordinary: List<i16, 2>,
}

fn limits() -> Limits {
    Limits {
        bytes: 128,
        allocation: 32_768,
        nodes: 512,
        depth: 32,
    }
}

#[test]
fn reference_typescript_exchanges_candidate_payloads_and_frames() {
    let types = BTreeMap::from([(
        "Record".into(),
        json!({"k":"record","fields":[
            {"name":"number","value":{"k":"u","bits":8}},
            {"name":"choice","value":{"k":"enum","variants":[
                {"name":"Ready","id":0,"value":{"k":"unit"}},
                {"name":"Value","id":1,"value":{"k":"u","bits":16}}
            ]}},
            {"name":"optional","value":{"k":"option","item":{"k":"u","bits":8}}},
            {"name":"bytes","value":{"k":"bytes","max":3}},
            {"name":"packed","value":{"k":"packed-i16","max":2}},
            {"name":"ordinary","value":{"k":"list","max":2,"item":{"k":"i","bits":16}}}
        ]}),
    )]);
    let graph = ValidatedRegistry::from_metadata(&types, SchemaLimits::default()).unwrap();
    let exported: Value = serde_json::from_str(&graph.metadata_json().unwrap()).unwrap();
    assert_eq!(exported, serde_json::to_value(&types).unwrap());
    reference::typescript::validate_payload_schema(&types, &json!({"k":"ref","name":"Record"}))
        .unwrap();

    let mut records = Vec::new();
    let mut literals = Vec::new();
    for (words, literal) in [
        (vec![], vec![0, 0, 0, 0, 0, 0]),
        (vec![0], vec![0, 0, 0, 0, 2, 0, 0, 1, 0]),
        (vec![32767], vec![0, 0, 0, 0, 2, 255, 127, 1, 254, 255, 3]),
    ] {
        records.push(Record {
            number: 0,
            choice: Choice::Ready,
            optional: None,
            bytes: Bytes::new(vec![]).unwrap(),
            packed: PackedI16::new(words.clone()).unwrap(),
            ordinary: List::new(words).unwrap(),
        });
        literals.push(literal);
    }
    records.push(Record {
        number: 7,
        choice: Choice::Value(128),
        optional: Some(9),
        bytes: Bytes::new(vec![0xaa, 0xbb]).unwrap(),
        packed: PackedI16::new(vec![1, -1]).unwrap(),
        ordinary: List::new(vec![1, -1]).unwrap(),
    });
    literals.push(vec![
        7, 1, 128, 1, 1, 9, 2, 170, 187, 4, 1, 0, 255, 255, 2, 2, 1,
    ]);

    let mut cases = Vec::new();
    for (record, literal) in records.iter().zip(&literals) {
        let payload = encode(record, Encoding::Payload, limits()).unwrap();
        assert_eq!(&payload, literal);
        let request = encode(
            record,
            Encoding::Frame {
                method: 7,
                direction: Direction::Request,
            },
            limits(),
        )
        .unwrap();
        let response = encode(
            record,
            Encoding::Frame {
                method: 7,
                direction: Direction::Response,
            },
            limits(),
        )
        .unwrap();
        assert_eq!(&request[..11], &[73, 87, 7, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&response[..11], &[73, 87, 7, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(&request[11..], literal);
        assert_eq!(&response[11..], literal);
        cases.push(json!({"value":{
            "number":record.number,
            "choice":match record.choice { Choice::Ready => json!({"tag":"Ready","value":null}), Choice::Value(value) => json!({"tag":"Value","value":value}) },
            "optional":record.optional.map(|value| json!({"some":value})),
            "bytes":record.bytes.as_slice(), "packed":record.packed.as_slice(), "ordinary":record.ordinary.as_slice()
        }, "payload":payload, "request":request, "response":response}));
    }
    let policy = limits();
    let fixture = json!({"types":exported,"cases":cases,"policy":{"bytes":policy.bytes,"nodes":policy.nodes,"depth":policy.depth}});
    // The published constant owns the runtime source; no sibling path or copied
    // product runtime is involved. Node runs it directly with type transformation.
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("ic-wire-interop-{}-{nonce}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let runtime = directory.join("reference-runtime.ts");
    fs::write(&runtime, reference::typescript::RUNTIME).unwrap();
    let mut child = Command::new("node")
        .args([
            "--experimental-transform-types",
            "--input-type=module",
            "--eval",
            include_str!("typescript_exchange.mjs"),
        ])
        .arg(&runtime)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node with --experimental-transform-types must be installed explicitly");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(&fixture).unwrap().as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    fs::remove_dir_all(&directory).unwrap();
    assert!(
        result.status.success(),
        "TypeScript exchange failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let outputs: Vec<Vec<Vec<u8>>> = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(outputs.len(), records.len());
    for ((output, record), literal) in outputs.iter().zip(&records).zip(&literals) {
        assert_eq!(output.len(), 3);
        for (bytes, encoding) in output.iter().zip([
            Encoding::Payload,
            Encoding::Frame {
                method: 7,
                direction: Direction::Request,
            },
            Encoding::Frame {
                method: 7,
                direction: Direction::Response,
            },
        ]) {
            assert_eq!(
                &decode::<Record>(bytes, encoding, limits()).unwrap(),
                record
            );
        }
        assert_eq!(&output[0], literal);
    }
}
