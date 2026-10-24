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
