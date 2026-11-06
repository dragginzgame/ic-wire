//! Cost advice includes shapes without a supported bulk replacement.
use ic_wire::typescript::{generate_registry, lint_bulk_roots};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn derived_hotspot_fields_reach_default_advice_and_bulk_fields_stay_quiet() {
    #[derive(serde::Serialize, ic_wire::Wire)]
    struct Hotspots {
        bytes: Vec<u8>,
        signed_bytes: Vec<i8>,
        signed_words: Vec<i16>,
        words: Vec<u16>,
        signed_ids: Vec<i32>,
        ids: Vec<u32>,
        signed_counters: Vec<i64>,
        counters: Vec<u64>,
        hashes: Vec<[u8; 32]>,
        flags: Vec<bool>,
        floats: Vec<ic_wire::FiniteF32>,
        doubles: Vec<ic_wire::FiniteF64>,
        wide_signed: Vec<i128>,
        wide_unsigned: Vec<u128>,
        units: Vec<()>,
        blob: ic_wire::ByteBuf,
        packed_words: ic_wire::Packed<i16>,
        packed_hashes: ic_wire::Packed<[u8; 32]>,
        #[serde(with = "serde_bytes")]
        adapted: Vec<u8>,
        records: Vec<String>,
    }
    let types = BTreeMap::from([(
        "Hotspots".into(),
        ic_wire::typescript::native_schema::<Hotspots>().unwrap(),
    )]);
    let mut report = None;
    let source = generate_registry(&types, &[], "./runtime.js", &BTreeMap::new(), |r| {
        report = Some(r)
    })
    .unwrap();
    let report = report.unwrap();
    let expected = [
        ("bytes", "u8"),
        ("signed_bytes", "i8"),
        ("signed_words", "i16"),
        ("words", "u16"),
        ("signed_ids", "i32"),
        ("ids", "u32"),
        ("signed_counters", "i64"),
        ("counters", "u64"),
        ("hashes", "hash32"),
        ("flags", "bool"),
        ("floats", "f32"),
        ("doubles", "f64"),
        ("wide_signed", "i128"),
        ("wide_unsigned", "u128"),
        ("units", "unit"),
    ]
    .map(|(field, kind)| format!("/types/Hotspots/fields/{field}/list/{kind}"));
    assert_eq!(
        report.warnings.iter().map(|w| &w.path).collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
    let acknowledgements = expected
        .into_iter()
        .map(|path| {
            (
                path,
                "Measured consumer maximum; retain representation".into(),
            )
        })
        .collect();
    assert_eq!(
        source,
        generate_registry(&types, &[], "./runtime.js", &acknowledgements, |r| assert!(
            r.is_clean()
        ))
        .unwrap()
    );
    assert_eq!(report.to_string().lines().count(), 9);
}

#[test]
fn expensive_lists_without_packed_support_have_acknowledgeable_paths() {
    let kinds = [
        ("bool", json!({"k":"bool"})),
        ("f32", json!({"k":"float","bits":32})),
        ("f64", json!({"k":"float","bits":64})),
        ("i128", json!({"k":"i","bits":128})),
        ("u128", json!({"k":"u","bits":128})),
        ("unit", json!({"k":"unit"})),
        ("empty-record", json!({"k":"record","fields":[]})),
        ("empty-tuple", json!({"k":"tuple","items":[]})),
        (
            "empty-array",
            json!({"k":"array","max":0,"item":{"k":"u","bits":8}}),
        ),
    ];
    for (kind, item) in kinds {
        let types = BTreeMap::from([("Element".into(), item)]);
        let roots = BTreeMap::from([(
            "response".into(),
            json!({"k":"list","max":4,"item":{"k":"ref","name":"Element"}}),
        )]);
        let report = lint_bulk_roots(&roots, &types, &BTreeMap::new()).unwrap();
        assert_eq!(report.warnings.len(), 1, "{kind}");
        let warning = &report.warnings[0];
        assert_eq!(warning.path, format!("/response/list/{kind}"));
        assert!(!warning.representation.starts_with("Packed"));
        let acknowledgements = BTreeMap::from([(
            warning.path.clone(),
            "At most four elements, cost measured".into(),
        )]);
        assert!(
            lint_bulk_roots(&roots, &types, &acknowledgements)
                .unwrap()
                .is_clean()
        );
        let scalar = BTreeMap::from([("response".into(), json!({"k":"ref","name":"Element"}))]);
        let stale = lint_bulk_roots(&scalar, &types, &acknowledgements).unwrap();
        assert!(stale.warnings.is_empty());
        assert_eq!(
            stale.stale_acknowledgements,
            std::slice::from_ref(&warning.path)
        );
    }
}

/// Copy unchanged to both isolated sources; includes the newly warned shapes.
#[test]
#[ignore = "explicit matched host-generation measurement"]
fn measure_new_cost_advice() {
    use std::{hint::black_box, time::Instant};
    println!(
        "source-sha256={}",
        option_env!("WIRE_STUDY_SOURCE_HASH").unwrap_or("unrecorded")
    );
    println!("types,sample,repetitions,host_ns_per_generation,warnings,source_bytes");
    for count in [1, 64, 512] {
        let types = (0..count)
            .map(|n| {
                (
                    format!("Shape{n}"),
                    json!({"k":"tuple","items":[
                        {"k":"list","item":{"k":"bool"}},
                        {"k":"list","item":{"k":"float","bits":64}},
                        {"k":"list","item":{"k":"u","bits":128}},
                        {"k":"list","item":{"k":"unit"}}
                    ]}),
                )
            })
            .collect();
        for sample in 0..11 {
            let repetitions = match count {
                1 => 1000,
                64 => 40,
                _ => 5,
            };
            let mut warnings = 0;
            let mut bytes = 0;
            let start = Instant::now();
            for _ in 0..repetitions {
                let source = generate_registry(
                    black_box(&types),
                    &[],
                    "./runtime.js",
                    &BTreeMap::new(),
                    |report| {
                        warnings = black_box(report.warnings.len());
                    },
                )
                .unwrap();
                bytes = black_box(source.len());
            }
            println!(
                "{count},{sample},{repetitions},{},{warnings},{bytes}",
                start.elapsed().as_nanos() / repetitions
            );
        }
    }
}

#[test]
fn default_generation_warns_without_changing_source_or_spamming() {
    let types = BTreeMap::from([("Flags".into(), json!({"k":"list","item":{"k":"bool"}}))]);
    let mut report = None;
    let source = generate_registry(&types, &[], "./runtime.js", &BTreeMap::new(), |r| {
        report = Some(r)
    })
    .unwrap();
    let report = report.unwrap();
    assert_eq!(report.warnings.len(), 1);
    let acknowledged = BTreeMap::from([(
        report.warnings[0].path.clone(),
        "Bounded admin flags".into(),
    )]);
    assert_eq!(
        source,
        generate_registry(&types, &[], "./runtime.js", &acknowledged, |r| assert!(
            r.is_clean()
        ))
        .unwrap()
    );
    let roots = (0..100)
        .map(|n| (format!("m{n}"), types["Flags"].clone()))
        .collect();
    let report = lint_bulk_roots(&roots, &types, &BTreeMap::new()).unwrap();
    assert_eq!(report.warnings.len(), 100);
    assert_eq!(report.to_string().lines().count(), 9);
}
