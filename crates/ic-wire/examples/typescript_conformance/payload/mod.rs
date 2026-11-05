//! Payload-only parity uses the same registry and scalar traversal as frames.
use ic_wire::{
    Limits, Wire, native,
    typescript::{self, SchemaRegistry},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fmt::Write as _, fs, path::Path};

#[derive(Debug, PartialEq, Serialize, Deserialize, Wire)]
struct Record {
    text: String,
    value: Option<u64>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Wire)]
enum Tree {
    End,
    Next(Vec<Self>),
    Value(Record),
}

fn add<T: typescript::Wire + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(
    registry: &mut SchemaRegistry,
    payloads: &mut Vec<Value>,
    checks: &mut String,
    name: &str,
    value: &T,
    javascript: &str,
) {
    let limits = Limits {
        bytes: 4096,
        allocation: 65536,
        nodes: 4096,
        depth: 64,
    };
    let schema = registry.register::<T>(&format!("Payload{name}")).unwrap();
    let mut metadata = typescript::payload_limits(limits).unwrap();
    metadata["name"] = json!(name);
    metadata["schema"] = schema;
    payloads.push(metadata);
    let bytes = native::encode(value, limits).unwrap();
    assert_eq!(native::decode::<T>(&bytes, limits).unwrap(), *value);
    writeln!(
        checks,
        "checkPayload(payloads.{name}, {javascript}, {bytes:?});"
    )
    .unwrap();
}

pub(super) fn generate(directory: &Path) -> String {
    fs::write(
        directory.join("payload-facade.ts"),
        include_str!("facade.ts"),
    )
    .unwrap();
    let mut registry = SchemaRegistry::new();
    let mut payloads = Vec::new();
    let mut checks = String::from("{ const payloads = await import('./compiled/payloads.js');\n");
    checks.push_str(include_str!("checks.mjs"));
    add(
        &mut registry,
        &mut payloads,
        &mut checks,
        "unit",
        &(),
        "null",
    );
    add(
        &mut registry,
        &mut payloads,
        &mut checks,
        "count",
        &7_u64,
        "7n",
    );
    add(
        &mut registry,
        &mut payloads,
        &mut checks,
        "args",
        &(7_u64,),
        "[7n]",
    );
    add(
        &mut registry,
        &mut payloads,
        &mut checks,
        "record",
        &Record {
            text: "é🦀".into(),
            value: Some(u64::MAX),
        },
        "{text:'é🦀',value:{some:18446744073709551615n}}",
    );
    add(
        &mut registry,
        &mut payloads,
        &mut checks,
        "recursive",
        &Tree::Next(vec![
            Tree::End,
            Tree::Value(Record {
                text: "x".into(),
                value: None,
            }),
        ]),
        "{tag:'Next',value:[{tag:'End',value:null},{tag:'Value',value:{text:'x',value:null}}]}",
    );
    let types = registry.finish().unwrap();
    fs::write(
        directory.join("payloads.ts"),
        typescript::generate_payload_registry(
            &types,
            &payloads,
            "./wire-runtime.js",
            &Default::default(),
            |report| eprint!("{report}"),
        )
        .unwrap(),
    )
    .unwrap();
    checks.push_str("console.log('Payload conformance: unit, scalar, tuple, records, recursive enums, limits and malformed inputs passed'); }\n");
    checks
}
