//! Differential rooted advice against the published reference, on admitted shapes.
use candidate::typescript::{
    GenerationError, Method, PayloadLimits, SchemaLimits, ValidatedRegistry,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn compare(
    types: &BTreeMap<String, Value>,
    roots: &BTreeMap<String, String>,
    acks: &BTreeMap<String, String>,
) -> candidate::typescript::BulkLintReport {
    let graph = ValidatedRegistry::from_metadata(types, SchemaLimits::default()).unwrap();
    let actual = graph.lint_bulk_roots(roots, acks).unwrap();
    let old_roots = roots
        .iter()
        .map(|(label, name)| (label.clone(), json!({"k":"ref","name":name})))
        .collect();
    let old = reference::typescript::lint_bulk_roots(&old_roots, types, acks).unwrap();
    assert_eq!(
        actual
            .warnings
            .iter()
            .map(|warning| &warning.path)
            .collect::<Vec<_>>(),
        old.warnings
            .iter()
            .map(|warning| &warning.path)
            .collect::<Vec<_>>()
    );
    assert_eq!(actual.stale_acknowledgements, old.stale_acknowledgements);
    actual
}
#[test]
fn rooted_advice_preserves_shared_recursive_paths_and_acknowledgements() {
    let types = BTreeMap::from([
        (
            "Words".into(),
            json!({"k":"list","item":{"k":"u","bits":32}}),
        ),
        (
            "Tree".into(),
            json!({"k":"record","fields":[
                {"name":"a/b~c","value":{"k":"ref","name":"Words"}},
                {"name":"other","value":{"k":"ref","name":"Words"}},
                {"name":"next","value":{"k":"option","item":{"k":"ref","name":"Tree"}}}
            ]}),
        ),
    ]);
    let roots = BTreeMap::from([("Reply/~".into(), "Tree".into())]);
    let report = compare(&types, &roots, &BTreeMap::new());
    assert_eq!(
        report
            .warnings
            .iter()
            .map(|w| w.path.as_str())
            .collect::<Vec<_>>(),
        [
            "/Reply~1~0/fields/a~1b~0c/list/u32",
            "/Reply~1~0/fields/other/list/u32"
        ]
    );
    let mut acks = BTreeMap::from([(
        report.warnings[0].path.clone(),
        "Small bounded values; retain varints".into(),
    )]);
    assert_eq!(compare(&types, &roots, &acks).warnings.len(), 1);
    acks.insert("/removed/list/u32".into(), "Former field".into());
    assert!(!compare(&types, &roots, &acks).is_clean());
    let changed = types
        .iter()
        .map(|(name, value)| {
            (
                name.clone(),
                if name == "Words" {
                    json!({"k":"list","item":{"k":"u","bits":64}})
                } else {
                    value.clone()
                },
            )
        })
        .collect();
    assert_eq!(
        compare(&changed, &roots, &acks)
            .stale_acknowledgements
            .len(),
        2
    );
    acks.insert("/blank".into(), "  ".into());
    let graph = ValidatedRegistry::from_metadata(&types, SchemaLimits::default()).unwrap();
    assert_eq!(
        graph.lint_bulk_roots(&roots, &acks),
        Err(GenerationError::Schema)
    );
    assert_eq!(
        graph.lint_bulk_roots(
            &BTreeMap::from([("Bad".into(), "Missing".into())]),
            &BTreeMap::new()
        ),
        Err(GenerationError::Schema)
    );
}
#[test]
fn advice_covers_primitive_and_zero_width_lists_without_flagging_bulk_fields() {
    let mut types: BTreeMap<String, Value> = BTreeMap::new();
    for (name, item) in [
        ("Bytes", json!({"k":"u","bits":8})),
        ("Wide", json!({"k":"i","bits":128})),
        ("Bool", json!({"k":"bool"})),
        ("F32", json!({"k":"float","bits":32})),
        ("Unit", json!({"k":"unit"})),
        ("EmptyRecord", json!({"k":"record","fields":[]})),
        ("EmptyTuple", json!({"k":"tuple","items":[]})),
        (
            "EmptyArray",
            json!({"k":"array","max":0,"item":{"k":"unit"}}),
        ),
        (
            "Hash",
            json!({"k":"newtype","item":{"k":"array","max":16,"item":{"k":"u","bits":8}}}),
        ),
    ] {
        types.insert(name.into(), json!({"k":"list","item":item}));
    }
    types.insert("Bulk".into(),json!({"k":"tuple","items":[{"k":"bytes"},{"k":"packed","word":"hash32"},{"k":"array","max":16,"item":{"k":"u","bits":8}}]}));
    let roots = types
        .keys()
        .map(|name| (name.clone(), name.clone()))
        .collect();
    let report = compare(&types, &roots, &BTreeMap::new());
    assert_eq!(report.warnings.len(), 9);
    assert_eq!(report.to_string().lines().count(), 9);
    assert!(report.to_string().contains("1 more bulk diagnostics"));
    let acks = report
        .warnings
        .into_iter()
        .map(|w| {
            (
                w.path,
                "Retain explicit bound and measured small-value representation".into(),
            )
        })
        .collect();
    assert!(compare(&types, &roots, &acks).is_clean());
}
#[test]
fn generation_requires_one_report_and_acknowledgements_do_not_change_source() {
    let graph = ValidatedRegistry::from_metadata(
        &BTreeMap::from([("Data".into(), json!({"k":"list","item":{"k":"u","bits":8}}))]),
        SchemaLimits::default(),
    )
    .unwrap();
    let limits = PayloadLimits {
        bytes: 64,
        nodes: 64,
        depth: 8,
    };
    let mut calls = 0;
    let source = graph
        .payload_module(
            "data",
            "Data",
            "./runtime.ts",
            limits,
            &BTreeMap::new(),
            |report| {
                calls += 1;
                assert_eq!(report.warnings[0].path, "/types/Data/list/u8");
            },
        )
        .unwrap();
    assert_eq!(calls, 1);
    let acks = BTreeMap::from([(
        "/types/Data/list/u8".into(),
        "Bounded ordinary fixture".into(),
    )]);
    assert_eq!(
        source,
        graph
            .payload_module(
                "data",
                "Data",
                "./runtime.ts",
                limits,
                &acks,
                |report| assert!(report.is_clean())
            )
            .unwrap()
    );
    let methods = [Method {
        export: "upload",
        id: 7,
        request: "Data",
        response: "Data",
        request_limits: limits,
        response_limits: limits,
    }];
    let mut paths = Vec::new();
    graph
        .method_module(&methods, "./runtime.ts", &BTreeMap::new(), |report| {
            paths = report.warnings.into_iter().map(|w| w.path).collect()
        })
        .unwrap();
    assert_eq!(
        paths,
        [
            "/methods/upload/request/list/u8",
            "/methods/upload/response/list/u8"
        ]
    );
    let bad = BTreeMap::from([("/x".into(), " ".into())]);
    assert_eq!(
        graph.payload_module("data", "Data", "./runtime.ts", limits, &bad, |_| panic!(
            "failed generation called handler"
        )),
        Err(GenerationError::Schema)
    );
}
#[test]
fn composed_services_and_exponential_graphs_preserve_reference_work_limits() {
    let mut types = BTreeMap::from([("N0".into(), json!({"k":"unit"}))]);
    for level in 1..=15 {
        types.insert(format!("N{level}"),json!({"k":"tuple","items":[{"k":"ref","name":format!("N{}",level-1)},{"k":"ref","name":format!("N{}",level-1)}]}));
    }
    let graph = ValidatedRegistry::from_metadata(&types, SchemaLimits::default()).unwrap();
    for (level, count, accepted) in [(14, 1, true), (15, 1, false), (14, 16, false)] {
        let roots = (0..count)
            .map(|n| (format!("root{n}"), format!("N{level}")))
            .collect();
        if accepted {
            assert!(compare(&types, &roots, &BTreeMap::new()).is_clean());
        } else {
            assert_eq!(
                graph.lint_bulk_roots(&roots, &BTreeMap::new()),
                Err(GenerationError::Complexity)
            );
        }
    }
    let types = BTreeMap::from([
        (
            "Leaf".into(),
            json!({"k":"record","fields":[{"name":"ids","value":{"k":"list","item":{"k":"u","bits":64}}},{"name":"next","value":{"k":"option","item":{"k":"ref","name":"Leaf"}}}]}),
        ),
        (
            "Page".into(),
            json!({"k":"tuple","items":vec![json!({"k":"ref","name":"Leaf"});25]}),
        ),
        (
            "Large".into(),
            json!({"k":"tuple","items":vec![json!({"k":"ref","name":"Page"});44]}),
        ),
    ]);
    let roots = (0..297)
        .flat_map(|n| {
            ["request", "response"].map(move |direction| {
                (
                    format!("m{n}/{direction}"),
                    if n == 0 && direction == "request" {
                        "Large"
                    } else {
                        "Page"
                    }
                    .into(),
                )
            })
        })
        .collect();
    assert_eq!(
        compare(&types, &roots, &BTreeMap::new()).warnings.len(),
        15_925
    );
}

#[test]
fn diagnostic_paths_are_bounded_and_clean_handlers_are_not_skipped() {
    let graph = ValidatedRegistry::from_metadata(
        &BTreeMap::from([("Empty".into(), json!({"k":"unit"}))]),
        SchemaLimits::default(),
    )
    .unwrap();
    for label in ["x".repeat(4096), "/".repeat(2048)] {
        assert_eq!(
            graph.lint_bulk_roots(&BTreeMap::from([(label, "Empty".into())]), &BTreeMap::new()),
            Err(GenerationError::Complexity)
        );
    }
    assert_eq!(
        graph.lint_bulk_roots(
            &BTreeMap::new(),
            &BTreeMap::from([("x".repeat(4097), "old path".into())])
        ),
        Err(GenerationError::Complexity)
    );
    let mut called = false;
    graph
        .payload_module(
            "empty",
            "Empty",
            "./runtime.ts",
            PayloadLimits {
                bytes: 0,
                nodes: 1,
                depth: 1,
            },
            &BTreeMap::new(),
            |report| {
                called = true;
                assert!(report.is_clean());
            },
        )
        .unwrap();
    assert!(called);
}
