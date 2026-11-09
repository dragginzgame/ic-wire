use ic_wire::typescript::native_schema;
use ic_wire::typescript::{GenerationError, lint_bulk_roots};
use serde_json::json;
use std::collections::BTreeMap;
mod composition_tests;

#[test]
fn resolved_aliases_nested_fields_and_acknowledgements() {
    type Samples = Vec<i64>;
    #[derive(serde::Serialize, ic_wire::Wire)]
    struct Request {
        samples: Option<Samples>,
        bytes: Vec<u8>,
        packed: ic_wire::Packed<i8>,
        bulk_bytes: ic_wire::ByteBuf,
        #[serde(with = "serde_bytes")]
        adapted: Vec<u8>,
        records: Vec<(bool, Vec<u16>)>,
        fixed: [i16; 3],
        unsupported: Vec<u128>,
    }
    let types = BTreeMap::from([("Request".into(), native_schema::<Request>().unwrap())]);
    let report = lint_bulk_roots(&types, &types, &BTreeMap::new()).unwrap();
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|w| (w.path.as_str(), w.representation))
            .collect::<Vec<_>>(),
        [
            ("/Request/fields/samples/item/list/i64", "Packed<i64>"),
            (
                "/Request/fields/bytes/list/u8",
                "ByteBuf / Bytes<N> / serde_bytes"
            ),
            (
                "/Request/fields/records/item/items/1/list/u16",
                "Packed<u16>"
            ),
            (
                "/Request/fields/unsupported/list/u128",
                "bounded u128 lists and workload-specific node limits; Packed<u128> is unsupported"
            ),
        ]
    );
    let mut acknowledged: BTreeMap<_, _> = report
        .warnings
        .iter()
        .map(|w| {
            (
                w.path.clone(),
                "Small, bounded consumer payload; retain varints".into(),
            )
        })
        .collect();
    assert!(
        lint_bulk_roots(&types, &types, &acknowledged)
            .unwrap()
            .is_clean()
    );
    acknowledged.insert("/Old/field".into(), "Former field".into());
    let report = lint_bulk_roots(&types, &types, &acknowledged).unwrap();
    assert!(!report.is_clean());
    assert_eq!(report.stale_acknowledgements, ["/Old/field"]);
    acknowledged.insert("/Old/field".into(), "  ".into());
    assert_eq!(
        lint_bulk_roots(&types, &types, &acknowledged),
        Err(GenerationError::Schema)
    );
}

#[test]
fn references_cycles_and_escaped_names_are_bounded() {
    let types = BTreeMap::from([
        (
            "Word".into(),
            json!({"k":"newtype","item":{"k":"i","bits":8}}),
        ),
        (
            "List".into(),
            json!({"k":"list","item":{"k":"ref","name":"Word"}}),
        ),
        (
            "Tree".into(),
            json!({"k":"record","fields":[
                {"name":"a/b~c","value":{"k":"list","item":{"k":"i","bits":32}}},
                {"name":"next","value":{"k":"option","item":{"k":"ref","name":"Tree"}}},
                {"name":"list","value":{"k":"ref","name":"List"}}
            ]}),
        ),
    ]);
    let report = lint_bulk_roots(&types, &types, &BTreeMap::new()).unwrap();
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|w| w.path.as_str())
            .collect::<Vec<_>>(),
        [
            "/List/list/i8",
            "/Tree/fields/a~1b~0c/list/i32",
            "/Tree/fields/next/item/fields/a~1b~0c/list/i32",
            "/Tree/fields/next/item/fields/list/list/i8",
            "/Tree/fields/list/list/i8"
        ]
    );
    for bad in [
        json!({"k":"ref","name":"Bad"}),
        json!({"k":"list","item":{"k":"absent"}}),
    ] {
        assert_eq!(
            lint_bulk_roots(
                &BTreeMap::from([("Bad".into(), bad)]),
                &BTreeMap::new(),
                &BTreeMap::new()
            ),
            Err(GenerationError::Schema)
        );
    }
    let mut deep = json!({"k":"i","bits":16});
    for _ in 0..66 {
        deep = json!({"k":"option","item":deep});
    }
    assert_eq!(
        lint_bulk_roots(
            &BTreeMap::from([("Deep".into(), deep)]),
            &BTreeMap::new(),
            &BTreeMap::new()
        ),
        Err(GenerationError::Complexity)
    );
}

#[test]
fn all_supported_list_words_and_enum_result_paths() {
    let types = BTreeMap::from([(
        "Root".into(),
        json!({"k":"enum","variants":[{"name":"Words","id":0,"value":{"k":"result","ok":{"k":"tuple","items": [
        {"k":"list","item":{"k":"i","bits":8}},
        {"k":"list","item":{"k":"i","bits":16}},
        {"k":"list","item":{"k":"u","bits":16}},
        {"k":"list","item":{"k":"i","bits":32}},
        {"k":"list","item":{"k":"u","bits":32}},
        {"k":"list","item":{"k":"i","bits":64}},
        {"k":"list","item":{"k":"u","bits":64}}
    ]},"err":{"k":"bytes"}}}]}),
    )]);
    let report = lint_bulk_roots(&types, &types, &BTreeMap::new()).unwrap();
    assert_eq!(report.warnings.len(), 7);
    assert_eq!(
        report.warnings[6].path,
        "/Root/variants/Words/ok/items/6/list/u64"
    );
    assert_eq!(report.warnings[6].representation, "Packed<u64>");
}

#[test]
fn rooted_review_distinguishes_shared_fields_and_ignores_intern_names() {
    use ic_wire::typescript::{SchemaRegistry, lint_bulk_roots};
    #[derive(serde::Serialize, ic_wire::Wire)]
    struct Reply {
        field_ids: Vec<u32>,
        other_ids: Vec<u32>,
    }
    let collect = |extra| {
        let mut registry = SchemaRegistry::new();
        if extra {
            registry.register::<Option<String>>("Unrelated").unwrap();
        }
        let root = registry.register::<Reply>("Reply").unwrap();
        (
            BTreeMap::from([("Reply".into(), root)]),
            registry.finish().unwrap(),
        )
    };
    let (roots, types) = collect(false);
    let acknowledged = BTreeMap::from([(
        "/Reply/fields/field_ids/list/u32".into(),
        "Small IDs".into(),
    )]);
    let report = lint_bulk_roots(&roots, &types, &acknowledged).unwrap();
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].path, "/Reply/fields/other_ids/list/u32");
    let (other_roots, other_types) = collect(true);
    assert_eq!(
        lint_bulk_roots(&other_roots, &other_types, &acknowledged).unwrap(),
        report
    );
    let mut removed = types.clone();
    removed.get_mut("Reply").unwrap()["fields"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    assert_eq!(
        lint_bulk_roots(&roots, &removed, &acknowledged)
            .unwrap()
            .stale_acknowledgements,
        ["/Reply/fields/field_ids/list/u32"]
    );
    for replacement in [
        json!({"k":"bytes"}),
        json!({"k":"list","item":{"k":"u","bits":64}}),
    ] {
        let mut changed = types.clone();
        changed.get_mut("Reply").unwrap()["fields"][0]["value"] = replacement;
        let report = lint_bulk_roots(&roots, &changed, &acknowledged).unwrap();
        assert_eq!(
            report.stale_acknowledgements,
            ["/Reply/fields/field_ids/list/u32"]
        );
        assert!(
            report
                .warnings
                .iter()
                .any(|w| w.path == "/Reply/fields/other_ids/list/u32")
        );
    }
}

#[test]
fn rooted_aliases_recursion_inline_errors_and_work_limits() {
    use ic_wire::typescript::lint_bulk_roots;
    for invalid in [json!({"k":"missing"}), json!({"k":"ref","name":"Missing"})] {
        assert_eq!(
            lint_bulk_roots(
                &BTreeMap::from([("Root".into(), invalid)]),
                &BTreeMap::new(),
                &BTreeMap::new()
            ),
            Err(GenerationError::Schema)
        );
    }
    let bulk = BTreeMap::from([
        ("Bytes".into(), json!({"k":"bytes"})),
        ("Packed".into(), json!({"k":"packed","word":"u32"})),
    ]);
    assert!(
        lint_bulk_roots(&bulk, &BTreeMap::new(), &BTreeMap::new())
            .unwrap()
            .is_clean()
    );
    let mut types = BTreeMap::from([
        (
            "Words".into(),
            json!({"k":"list","item":{"k":"u","bits":32,"max":17}}),
        ),
        ("Alias".into(), json!({"k":"ref","name":"Words"})),
        (
            "Tree".into(),
            json!({"k":"enum","variants":[
                {"id":0,"name":"Leaf","value":{"k":"ref","name":"Alias"}},
                {"id":1,"name":"Next","value":{"k":"record","fields":[
                    {"name":"values","value":{"k":"ref","name":"Words"}},
                    {"name":"next","value":{"k":"option","item":{"k":"ref","name":"Tree"}}}
                ]}}
            ]}),
        ),
    ]);
    let roots = BTreeMap::from([(
        "query/reply".into(),
        json!({"k":"result","ok":{"k":"ref","name":"Tree"},"err":{"k":"ref","name":"Alias"}}),
    )]);
    let report = lint_bulk_roots(&roots, &types, &BTreeMap::new()).unwrap();
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|w| w.path.as_str())
            .collect::<Vec<_>>(),
        [
            "/query~1reply/ok/variants/Leaf/list/u32",
            "/query~1reply/ok/variants/Next/fields/values/list/u32",
            "/query~1reply/err/list/u32",
        ]
    );
    types.insert("Alias".into(), json!({"k":"ref","name":"Missing"}));
    assert_eq!(
        lint_bulk_roots(&roots, &types, &BTreeMap::new()),
        Err(GenerationError::Schema)
    );
    // A tiny acyclic DAG can expand exponentially; refuse it before generating
    // an unbounded report, even though the definitions individually validate.
    let mut dag = BTreeMap::from([("N0".into(), json!({"k":"list","item":{"k":"u","bits":32}}))]);
    for n in 1..18 {
        dag.insert(format!("N{n}"),json!({"k":"tuple","items":[{"k":"ref","name":format!("N{}",n-1)},{"k":"ref","name":format!("N{}",n-1)}]}));
    }
    let roots = BTreeMap::from([("Root".into(), json!({"k":"ref","name":"N17"}))]);
    assert_eq!(
        lint_bulk_roots(&roots, &dag, &BTreeMap::new()),
        Err(GenerationError::Complexity)
    );
    assert_eq!(
        ic_wire::typescript::generate_registry(
            &dag,
            &[],
            "./runtime.js",
            &BTreeMap::new(),
            |_| panic!("excessive expansion must not emit a partial report"),
        ),
        Err(GenerationError::Complexity)
    );
}

#[test]
fn generation_always_reports_use_sites_and_preserves_source_when_acknowledged() {
    use ic_wire::typescript::generate_registry;
    let types = BTreeMap::from([
        (
            "Words".into(),
            json!({"k":"list","item":{"k":"u","bits":8}}),
        ),
        (
            "Message".into(),
            json!({"k":"record","fields":[{"name":"data","value":{"k":"ref","name":"Words"}}]}),
        ),
    ]);
    let method = json!({"name":"upload","mode":"update","id":"1",
        "request":{"k":"ref","name":"Message"},"response":{"k":"ref","name":"Message"},
        "requestBytes":128,"responseBytes":128,"requestNodes":128,"responseNodes":128,
        "requestDepth":8,"responseDepth":8});
    let methods = [method];
    let mut seen = None;
    let source = generate_registry(
        &types,
        &methods,
        "./runtime.js",
        &BTreeMap::new(),
        |report| seen = Some(report),
    )
    .unwrap();
    let report = seen.unwrap();
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|w| w.path.as_str())
            .collect::<Vec<_>>(),
        [
            "/methods/upload/request/fields/data/list/u8",
            "/methods/upload/response/fields/data/list/u8",
        ]
    );
    assert!(report.to_string().contains("truncated-input rejection"));
    let mut acknowledged =
        BTreeMap::from([(report.warnings[0].path.clone(), "At most four bytes".into())]);
    assert_eq!(
        source,
        generate_registry(&types, &methods, "./runtime.js", &acknowledged, |report| {
            assert_eq!(report.warnings.len(), 1);
            assert_eq!(
                report.warnings[0].path,
                "/methods/upload/response/fields/data/list/u8"
            );
        })
        .unwrap()
    );
    acknowledged.insert(report.warnings[1].path.clone(), "Small response".into());
    assert_eq!(
        source,
        generate_registry(
            &types,
            &methods,
            "./runtime.js",
            &acknowledged,
            |report| assert!(report.is_clean())
        )
        .unwrap()
    );
    acknowledged.insert("/old".into(), "Removed field".into());
    generate_registry(&types, &methods, "./runtime.js", &acknowledged, |report| {
        assert!(!report.is_clean());
        assert_eq!(report.stale_acknowledgements, ["/old"]);
        assert!(report.to_string().contains("stale acknowledgement"));
    })
    .unwrap();
}

#[test]
fn generation_reports_exported_types_and_never_emits_partial_diagnostics() {
    use ic_wire::typescript::generate_registry;
    let types = BTreeMap::from([("Samples".into(), native_schema::<Vec<i16>>().unwrap())]);
    generate_registry(&types, &[], "./runtime.js", &BTreeMap::new(), |report| {
        assert_eq!(report.warnings[0].path, "/types/Samples/list/i16");
    })
    .unwrap();
    let empty_reason = BTreeMap::from([("/types/Samples/list/i16".into(), " ".into())]);
    assert_eq!(
        generate_registry(&types, &[], "./runtime.js", &empty_reason, |_| panic!(
            "invalid input must not emit"
        )),
        Err(GenerationError::Schema)
    );
    assert_eq!(
        generate_registry(
            &types,
            &[json!({})],
            "./runtime.js",
            &BTreeMap::new(),
            |_| panic!("invalid method must not emit")
        ),
        Err(GenerationError::Schema)
    );
    generate_registry(
        &BTreeMap::new(),
        &[],
        "./runtime.js",
        &BTreeMap::new(),
        |report| assert!(report.is_clean()),
    )
    .unwrap();
}

#[test]
fn hash_lists_resolve_aliases_preserve_paths_and_acknowledgements() {
    use ic_wire::typescript::lint_bulk_roots;
    type Hash = [u8; 32];
    #[derive(serde::Serialize, ic_wire::Wire)]
    struct Reply {
        hashes: Option<ic_wire::List<Hash, 128>>,
        ids: Vec<[u8; 16]>,
        packed_ids: ic_wire::Packed<[u8; 16]>,
        fixed_id: [u8; 16],
        fixed_hash: Hash,
        packed: ic_wire::Packed<Hash>,
        byte_arrays: Vec<ic_wire::ByteArray<32>>,
        short: Vec<[u8; 31]>,
        signed: Vec<[i8; 32]>,
        nested: Vec<([u8; 32], Vec<Hash>)>,
    }
    let schema = native_schema::<Reply>().unwrap();
    let roots = BTreeMap::from([("Reply".into(), schema)]);
    let report = lint_bulk_roots(&roots, &BTreeMap::new(), &BTreeMap::new()).unwrap();
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|w| (w.path.as_str(), w.representation))
            .collect::<Vec<_>>(),
        [
            ("/Reply/fields/hashes/item/list/hash32", "Packed<[u8; 32]>"),
            ("/Reply/fields/ids/list/hash16", "Packed<[u8; 16]>"),
            (
                "/Reply/fields/nested/item/items/1/list/hash32",
                "Packed<[u8; 32]>"
            ),
        ]
    );
    let acknowledgements = report
        .warnings
        .iter()
        .map(|w| {
            (
                w.path.clone(),
                "Small bounded hash list; retain this contract".into(),
            )
        })
        .collect();
    assert!(
        lint_bulk_roots(&roots, &BTreeMap::new(), &acknowledgements)
            .unwrap()
            .is_clean()
    );
    let types = BTreeMap::from([
        (
            "Byte".into(),
            json!({"k":"newtype","item":{"k":"u","bits":8}}),
        ),
        (
            "Hash".into(),
            json!({"k":"newtype","item":{"k":"array","max":32,"item":{"k":"ref","name":"Byte"}}}),
        ),
        (
            "Hashes".into(),
            json!({"k":"list","item":{"k":"ref","name":"Hash"}}),
        ),
        (
            "Recursive".into(),
            json!({"k":"array","max":32,"item":{"k":"ref","name":"Recursive"}}),
        ),
        (
            "RecursiveList".into(),
            json!({"k":"list","item":{"k":"ref","name":"Recursive"}}),
        ),
    ]);
    let report = lint_bulk_roots(&types, &types, &BTreeMap::new()).unwrap();
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].path, "/Hashes/list/hash32");
    assert_eq!(report.warnings[0].representation, "Packed<[u8; 32]>");
}

#[test]
fn display_bounds_warning_and_stale_output_without_losing_paths() {
    use ic_wire::typescript::{BulkLintReport, BulkWarning};
    let report = BulkLintReport {
        warnings: (0..20)
            .map(|i| BulkWarning {
                path: format!("/hashes-{i}"),
                representation: "Packed<[u8; 32]>",
            })
            .collect(),
        stale_acknowledgements: (0..10).map(|i| format!("/old-{i}")).collect(),
    };
    let output = report.to_string();
    assert_eq!(output.lines().count(), 9);
    assert!(output.contains("/hashes-7:"));
    assert!(!output.contains("/hashes-8:"));
    assert!(output.contains("22 more bulk diagnostics"));
    assert_eq!(report.warnings.len(), 20);
    assert_eq!(report.stale_acknowledgements.len(), 10);
    assert!(!report.is_clean());
    let stale = BulkLintReport {
        warnings: vec![],
        stale_acknowledgements: report.stale_acknowledgements,
    };
    assert_eq!(stale.to_string().lines().count(), 9);
    assert!(stale.to_string().contains("2 more bulk diagnostics"));
}
mod cost_tests;

#[test]
fn float_advice_distinguishes_ordinary_finite_lists_from_explicit_adapters() {
    #[derive(serde::Serialize, ic_wire::Wire)]
    struct Floats {
        ordinary: Vec<ic_wire::FiniteF32>,
        #[serde(with = "ic_wire::serde_packed::finite_f32")]
        adapted: Vec<f32>,
    }
    let roots = BTreeMap::from([("Floats".into(), native_schema::<Floats>().unwrap())]);
    let report = lint_bulk_roots(&roots, &roots, &BTreeMap::new()).unwrap();
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].path, "/Floats/fields/ordinary/list/f32");
    assert!(
        report.warnings[0]
            .representation
            .contains("primitive Vec<f32>")
    );
    assert!(
        report.warnings[0]
            .representation
            .contains("preserve bounds")
    );
}
