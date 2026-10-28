use ic_wire::{
    Limits,
    typescript::{self, GenerationError},
};
use serde_json::json;
use std::{cell::Cell, collections::BTreeMap};

#[test]
fn payload_generation_validates_policy_references_names_and_diagnostics() {
    let policy = Limits {
        bytes: 0,
        allocation: 0,
        nodes: 1,
        depth: 1,
    };
    assert_eq!(
        typescript::payload_limits(policy).unwrap(),
        json!({"bytes":0,"nodes":1,"depth":1})
    );
    assert_eq!(
        typescript::method_limits(policy, policy),
        Err(GenerationError::Limits)
    );
    let types = BTreeMap::from([("Units".into(), json!({"k":"unit"}))]);
    let payload =
        json!({"name":"codec","schema":{"k":"ref","name":"Units"},"bytes":0,"nodes":1,"depth":1});
    let generate = |entries: &[serde_json::Value]| {
        typescript::generate_payload_registry(
            &types,
            entries,
            "./runtime.js",
            &Default::default(),
            |report| assert!(report.is_clean()),
        )
    };
    let source = generate(std::slice::from_ref(&payload)).unwrap();
    assert!(source.contains("createPayloadCodec as $createCodec"));
    assert!(source.contains("encode: (value: Units)"));
    assert_eq!(
        generate(&[payload.clone(), payload.clone()]),
        Err(GenerationError::Schema)
    );
    for (key, value, expected) in [
        ("name", json!("Units"), GenerationError::Schema),
        ("name", json!("bad-name"), GenerationError::Schema),
        (
            "schema",
            json!({"k":"ref","name":"Missing"}),
            GenerationError::Schema,
        ),
        ("nodes", json!(0), GenerationError::Limits),
        ("depth", json!(0), GenerationError::Limits),
        ("bytes", json!(-1), GenerationError::Complexity),
        (
            "bytes",
            json!(9007199254740992_u64),
            GenerationError::Complexity,
        ),
    ] {
        let mut bad = payload.clone();
        bad[key] = value;
        assert_eq!(generate(&[bad]), Err(expected));
    }
    let list = json!({"name":"items","schema":{"k":"list","item":{"k":"u","bits":64}},"bytes":64,"nodes":64,"depth":16});
    let called = Cell::new(false);
    typescript::generate_payload_registry(
        &types,
        std::slice::from_ref(&list),
        "./runtime.js",
        &Default::default(),
        |report| {
            called.set(true);
            assert_eq!(report.warnings[0].path, "/payloads/items/list/u64");
        },
    )
    .unwrap();
    assert!(called.get());
    let acknowledgements =
        BTreeMap::from([("/payloads/items/list/u64".into(), "small values".into())]);
    typescript::generate_payload_registry(
        &types,
        &[list],
        "./runtime.js",
        &acknowledgements,
        |report| assert!(report.is_clean()),
    )
    .unwrap();
}
