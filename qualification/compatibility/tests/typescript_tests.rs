//! Candidate Rust exchange with the published reference TypeScript runtime.

mod scalar_cases;

use candidate::typescript::{
    Method, PayloadLimits, RUNTIME_SCHEMA_FORMAT, SchemaLimits, ValidatedRegistry,
};
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Chain {
    value: u16,
    next: Option<Box<Chain>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Shared {
    left: Chain,
    right: Chain,
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
    let mut types = BTreeMap::from([(
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
    types.insert(
        "Chain".into(),
        json!({"k":"record","fields":[
            {"name":"value","value":{"k":"u","bits":16}},
            {"name":"next","value":{"k":"option","item":{"k":"ref","name":"Chain"}}}
        ]}),
    );
    types.insert(
        "Shared".into(),
        json!({"k":"record","fields":[
            {"name":"left","value":{"k":"ref","name":"Chain"}},
            {"name":"right","value":{"k":"ref","name":"Chain"}}
        ]}),
    );
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
    let chain = Chain {
        value: 1,
        next: Some(Box::new(Chain {
            value: 128,
            next: None,
        })),
    };
    let shared = Shared {
        left: Chain {
            value: 7,
            next: None,
        },
        right: Chain {
            value: 7,
            next: None,
        },
    };
    let chain_bytes = encode(&chain, Encoding::Payload, policy).unwrap();
    let shared_bytes = encode(&shared, Encoding::Payload, policy).unwrap();
    assert_eq!(chain_bytes, [1, 1, 128, 1, 0]);
    assert_eq!(shared_bytes, [7, 0, 7, 0]);
    let fixture = json!({"types":exported,"cases":cases,"chain":chain_bytes,"shared":shared_bytes,"policy":{"bytes":policy.bytes,"nodes":policy.nodes,"depth":policy.depth}});
    let browser_limits = PayloadLimits {
        bytes: policy.bytes as u64,
        nodes: policy.nodes as u64,
        depth: policy.depth as u64,
    };
    let mut fixture = fixture;
    fixture["module"] = graph
        .payload_module(
            "recordCodec",
            "Record",
            "./runtime-adapter.ts",
            browser_limits,
        )
        .unwrap()
        .into();
    fixture["badModule"] = graph
        .payload_module("recordCodec", "Record", "./bad-runtime.ts", browser_limits)
        .unwrap()
        .into();
    let framed_limits = PayloadLimits {
        bytes: browser_limits.bytes - 11,
        ..browser_limits
    };
    let methods = [
        Method {
            export: "roundtrip",
            id: 7,
            request: "Record",
            response: "Record",
            request_limits: framed_limits,
            response_limits: framed_limits,
        },
        Method {
            export: "highId",
            id: u64::MAX,
            request: "Chain",
            response: "Shared",
            request_limits: PayloadLimits {
                bytes: 5,
                nodes: 6,
                depth: 4,
            },
            response_limits: PayloadLimits {
                bytes: 4,
                nodes: 7,
                depth: 3,
            },
        },
    ];
    fixture["frameModule"] = graph
        .method_module(&methods, "./runtime-adapter.ts")
        .unwrap()
        .into();
    fixture["badFrameModule"] = graph
        .method_module(&methods, "./bad-runtime.ts")
        .unwrap()
        .into();
    let result = run_typescript(&fixture, include_str!("typescript_exchange.mjs"));
    let high_request: Vec<u8> = serde_json::from_value(result["highRequest"].clone()).unwrap();
    let high_response: Vec<u8> = serde_json::from_value(result["highResponse"].clone()).unwrap();
    assert_eq!(
        decode::<Chain>(
            &high_request,
            Encoding::Frame {
                method: u64::MAX,
                direction: Direction::Request
            },
            policy
        )
        .unwrap(),
        chain
    );
    assert_eq!(
        decode::<Shared>(
            &high_response,
            Encoding::Frame {
                method: u64::MAX,
                direction: Direction::Response
            },
            policy
        )
        .unwrap(),
        shared
    );
    let returned_chain: Vec<u8> = serde_json::from_value(result["chain"].clone()).unwrap();
    let returned_shared: Vec<u8> = serde_json::from_value(result["shared"].clone()).unwrap();
    assert_eq!(returned_chain, chain_bytes);
    assert_eq!(returned_shared, shared_bytes);
    assert_eq!(
        decode::<Chain>(&returned_chain, Encoding::Payload, policy).unwrap(),
        chain
    );
    assert_eq!(
        decode::<Shared>(&returned_shared, Encoding::Payload, policy).unwrap(),
        shared
    );
    let outputs: Vec<Vec<Vec<u8>>> = serde_json::from_value(result["records"].clone()).unwrap();
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

fn run_typescript(fixture: &Value, script: &str) -> Value {
    // Published runtime source, without sibling paths or product copies.
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("ic-wire-interop-{}-{nonce}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let runtime = directory.join("reference-runtime.ts");
    fs::write(&runtime, reference::typescript::RUNTIME).unwrap();
    if let Some(module) = fixture["module"].as_str() {
        fs::write(directory.join("client.ts"), module).unwrap();
        fs::write(
            directory.join("frame-client.ts"),
            fixture["frameModule"].as_str().unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("bad-frame-client.ts"),
            fixture["badFrameModule"].as_str().unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("bad-client.ts"),
            fixture["badModule"].as_str().unwrap(),
        )
        .unwrap();
        fs::write(directory.join("runtime-adapter.ts"), format!(
            "import {{createPayloadCodec as create, createCodec as frame}} from './reference-runtime.ts';\nexport const schemaFormat = {RUNTIME_SCHEMA_FORMAT:?};\nexport let observed: unknown;\nexport const observedMethods: unknown[] = [];\nexport function createPayloadCodec(meta: unknown, types: unknown) {{ observed = structuredClone(meta); return create(meta, types); }}\nexport function createCodec(meta: unknown, types: unknown) {{ observedMethods.push(structuredClone(meta)); return frame(meta, types); }}\n"
        )).unwrap();
        fs::write(directory.join("bad-runtime.ts"), "export const schemaFormat = 'incompatible'; export function createPayloadCodec() { throw new Error('factory must not run'); } export const createCodec = createPayloadCodec;").unwrap();
    }
    let mut child = Command::new("node")
        .args([
            "--experimental-transform-types",
            "--input-type=module",
            "--eval",
            script,
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
        .write_all(serde_json::to_string(fixture).unwrap().as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    fs::remove_dir_all(&directory).unwrap();
    assert!(
        result.status.success(),
        "TypeScript exchange failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}
