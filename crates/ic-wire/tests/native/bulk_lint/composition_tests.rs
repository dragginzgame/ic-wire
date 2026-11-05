//! Small synthetic composition, not an application schema snapshot.
use ic_wire::typescript::{GenerationError, generate_registry, lint_bulk_roots};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn composed(method_count: usize) -> (BTreeMap<String, Value>, Vec<Value>) {
    let types = BTreeMap::from([
        (
            "Words".into(),
            json!({"k":"list","item":{"k":"u","bits":64}}),
        ),
        (
            "Leaf".into(),
            json!({"k":"record","fields":[
                {"name":"ids","value":{"k":"ref","name":"Words"}},
                {"name":"next","value":{"k":"option","item":{"k":"ref","name":"Leaf"}}}
            ]}),
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
    let methods = (0..method_count).map(|n| json!({
        "name":format!("m{n}"),"mode":"query","id":n.to_string(),
        "request":{"k":"ref","name":if method_count == 1 {"Leaf"} else if method_count >= 297 && n == 0 {"Large"} else {"Page"}},
        "response":{"k":"ref","name":if method_count == 1 {"Leaf"} else {"Page"}},
        "requestBytes":4096,"responseBytes":4096,"requestNodes":4096,"responseNodes":4096,
        "requestDepth":64,"responseDepth":64
    })).collect();
    (types, methods)
}

#[test]
fn composed_methods_preserve_every_use_site_and_deterministic_output() {
    // 594 public roots, 112,751 expanded visits; largest root 7,790.
    // Named definitions stay tiny. Recursion stops only on the active path.
    let (types, methods) = composed(297);
    let mut report = None;
    let source = generate_registry(&types, &methods, "./runtime.js", &BTreeMap::new(), |r| {
        report = Some(r)
    })
    .unwrap();
    let report = report.unwrap();
    assert_eq!(report.warnings.len(), 15_925);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.path == "/methods/m0/request/items/0/items/0/fields/ids/list/u64")
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.path == "/methods/m296/response/items/24/fields/ids/list/u64")
    );
    assert_eq!(report.to_string().lines().count(), 9);
    let acknowledgements = report
        .warnings
        .iter()
        .map(|w| (w.path.clone(), "Bounded small-value list".into()))
        .collect();
    assert_eq!(
        source,
        generate_registry(
            &types,
            &methods,
            "./runtime.js",
            &acknowledgements,
            |r| assert!(r.is_clean())
        )
        .unwrap()
    );
    let roots = methods
        .iter()
        .flat_map(|method| {
            ["request", "response"].map(|direction| {
                (
                    format!("{}/{direction}", method["name"].as_str().unwrap()),
                    method[direction].clone(),
                )
            })
        })
        .collect();
    assert_eq!(
        lint_bulk_roots(&roots, &types, &BTreeMap::new())
            .unwrap()
            .warnings
            .len(),
        report.warnings.len()
    );
}

fn dag(level: usize, width: usize) -> BTreeMap<String, Value> {
    let mut types = BTreeMap::from([("N0".into(), json!({"k":"unit"}))]);
    for n in 1..=level {
        types.insert(
            format!("N{n}"),
            json!({"k":"tuple","items":vec![json!({"k":"ref","name":format!("N{}",n-1)});width]}),
        );
    }
    types
}

#[test]
fn expanded_depth_per_root_and_aggregate_work_remain_bounded() {
    // Binary DAG N14 expands to 65,534 visits; N15 exceeds one root's ceiling.
    for (level, count, accepted) in [
        (14, 1, true),
        (15, 1, false),
        (14, 16, false),
        (32, 1, false),
    ] {
        let types = dag(level, if level == 32 { 1 } else { 2 });
        let roots = (0..count)
            .map(|n| {
                (
                    format!("root{n}"),
                    json!({"k":"ref","name":format!("N{level}")}),
                )
            })
            .collect();
        let result = lint_bulk_roots(&roots, &types, &BTreeMap::new());
        if accepted {
            assert!(result.unwrap().is_clean());
        } else {
            assert_eq!(result, Err(GenerationError::Complexity));
        }
    }
}

/// Host-only cost probe; copy this runner unchanged into an isolated baseline.
#[test]
#[ignore = "explicit matched host-generation measurement"]
fn measure_composed_generation() {
    use std::{hint::black_box, time::Instant};
    println!(
        "source-sha256={}",
        option_env!("WIRE_STUDY_SOURCE_HASH").unwrap_or("unrecorded")
    );
    println!("methods,sample,repetitions,host_ns_per_generation,accepted,source_bytes");
    for count in [1, 32, 297] {
        let (types, methods) = composed(count);
        for sample in 0..11 {
            let start = Instant::now();
            let mut accepted = 0;
            let mut bytes = 0;
            let repetitions = match count {
                1 => 1_000,
                32 => 20,
                _ => 3,
            };
            for _ in 0..repetitions {
                let result = generate_registry(
                    black_box(&types),
                    black_box(&methods),
                    "./runtime.js",
                    &BTreeMap::new(),
                    |r| {
                        black_box(r);
                    },
                );
                if let Ok(source) = black_box(result) {
                    accepted += 1;
                    bytes = source.len();
                }
            }
            println!(
                "{count},{sample},{repetitions},{},{accepted},{bytes}",
                start.elapsed().as_nanos() / repetitions
            );
        }
    }
}
