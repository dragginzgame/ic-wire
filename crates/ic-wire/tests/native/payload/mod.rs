use ic_wire::{
    Limits,
    typescript::{self, GenerationError},
};
use serde_json::json;
use std::{cell::Cell, collections::BTreeMap};

#[test]
fn standalone_schema_accepts_constructor_tuples_and_guarded_recursion() {
    let types = BTreeMap::from([
        ("Count".into(), json!({"k":"u","bits":64})),
        (
            "Tree".into(),
            json!({"k":"record","fields":[
                {"name":"value","value":{"k":"ref","name":"Count"}},
                {"name":"children","value":{"k":"list","item":{"k":"ref","name":"Tree"}}}
            ]}),
        ),
    ]);
    for schema in [
        json!({"k":"unit"}),
        json!({"k":"ref","name":"Tree"}),
        json!({"k":"tuple","items":[{"k":"ref","name":"Count"},{"k":"text","check":"consumerRule"}]}),
    ] {
        assert_eq!(typescript::validate_payload_schema(&types, &schema), Ok(()));
    }
}

#[test]
fn standalone_schema_reuses_generator_rejections_including_unused_definitions() {
    for bad in [
        json!({"k":"ref","name":"Missing"}),
        json!({"k":"u","bits":7}),
        json!({"k":"bytes","min":2,"max":1}),
        json!({"k":"array","item":{"k":"unit"}}),
        json!({"k":"unit","check":false}),
        json!({"k":"unknown"}),
    ] {
        let types = BTreeMap::new();
        assert_eq!(
            typescript::validate_payload_schema(&types, &bad),
            Err(GenerationError::Schema)
        );
        let payload = json!({"name":"codec","schema":bad,"bytes":64,"nodes":64,"depth":16});
        assert_eq!(
            typescript::generate_payload_registry(
                &types,
                &[payload],
                "./runtime.js",
                &Default::default(),
                |_| panic!("invalid metadata")
            ),
            Err(GenerationError::Schema)
        );
        let types = BTreeMap::from([("Unused".into(), bad)]);
        assert_eq!(
            typescript::validate_payload_schema(&types, &json!({"k":"unit"})),
            Err(GenerationError::Schema)
        );
    }
    let types = BTreeMap::from([("bad-name".into(), json!({"k":"unit"}))]);
    assert_eq!(
        typescript::validate_payload_schema(&types, &json!({"k":"unit"})),
        Err(GenerationError::Schema)
    );
    let aliases = BTreeMap::from([
        ("Left".into(), json!({"k":"ref","name":"Right"})),
        (
            "Right".into(),
            json!({"k":"newtype","item":{"k":"ref","name":"Left"}}),
        ),
    ]);
    assert_eq!(
        typescript::validate_payload_schema(&aliases, &json!({"k":"unit"})),
        Err(GenerationError::Schema)
    );
}

#[test]
fn standalone_schema_shares_depth_and_total_work_allowances_with_registry() {
    let types = BTreeMap::from([("Unused".into(), json!({"k":"unit"}))]);
    let mut root = json!({"k":"unit"});
    for _ in 0..64 {
        root = json!({"k":"option","item":root});
    }
    assert_eq!(typescript::validate_payload_schema(&types, &root), Ok(()));
    let deep = json!({"k":"option","item":root});
    assert_eq!(
        typescript::validate_payload_schema(&types, &deep),
        Err(GenerationError::Complexity)
    );
    let mut root = json!({"k":"tuple","items":vec![json!({"k":"unit"});65534]});
    assert_eq!(typescript::validate_payload_schema(&types, &root), Ok(()));
    root["items"]
        .as_array_mut()
        .unwrap()
        .push(json!({"k":"unit"}));
    // Registry + tuple + children = 65,537 nodes; roots do not get a fresh budget.
    assert_eq!(
        typescript::validate_payload_schema(&types, &root),
        Err(GenerationError::Complexity)
    );
    assert_eq!(
        typescript::validate_payload_schema(&BTreeMap::new(), &root),
        Ok(())
    );
}

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
