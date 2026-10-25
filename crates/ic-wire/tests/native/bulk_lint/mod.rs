use ic_wire::typescript::native_schema;
use ic_wire::typescript::{GenerationError, lint_bulk_fields};
use serde_json::json;
use std::collections::BTreeMap;

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
    let report = lint_bulk_fields(&types, &BTreeMap::new()).unwrap();
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|w| (w.path.as_str(), w.representation))
            .collect::<Vec<_>>(),
        [
            ("/Request/fields/samples/item", "Packed<i64>"),
            ("/Request/fields/bytes", "ByteBuf / Bytes<N> / serde_bytes"),
            ("/Request/fields/records/item/items/1", "Packed<u16>"),
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
    assert!(lint_bulk_fields(&types, &acknowledged).unwrap().is_clean());
    acknowledged.insert("/Old/field".into(), "Former field".into());
    let report = lint_bulk_fields(&types, &acknowledged).unwrap();
    assert!(!report.is_clean());
    assert_eq!(report.stale_acknowledgements, ["/Old/field"]);
    acknowledged.insert("/Old/field".into(), "  ".into());
    assert_eq!(
        lint_bulk_fields(&types, &acknowledged),
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
    let report = lint_bulk_fields(&types, &BTreeMap::new()).unwrap();
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|w| w.path.as_str())
            .collect::<Vec<_>>(),
        ["/List", "/Tree/fields/a~1b~0c"]
    );
    for bad in [
        json!({"k":"ref","name":"Bad"}),
        json!({"k":"list","item":{"k":"absent"}}),
    ] {
        assert_eq!(
            lint_bulk_fields(&BTreeMap::from([("Bad".into(), bad)]), &BTreeMap::new()),
            Err(GenerationError::Schema)
        );
    }
    let mut deep = json!({"k":"i","bits":16});
    for _ in 0..66 {
        deep = json!({"k":"option","item":deep});
    }
    assert_eq!(
        lint_bulk_fields(&BTreeMap::from([("Deep".into(), deep)]), &BTreeMap::new()),
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
    let report = lint_bulk_fields(&types, &BTreeMap::new()).unwrap();
    assert_eq!(report.warnings.len(), 7);
    assert_eq!(report.warnings[6].path, "/Root/variants/Words/ok/items/6");
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
}
