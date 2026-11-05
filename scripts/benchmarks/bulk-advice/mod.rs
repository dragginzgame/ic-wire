//! Copy-only host probe for bulk advice; never a published API or codec mode.
use ic_wire::typescript::{generate_registry, lint_bulk_roots};
use serde_json::json;
use std::{collections::BTreeMap, hint::black_box, time::Instant};
fn main() {
    let output = std::path::PathBuf::from(std::env::args_os().nth(1).unwrap());
    std::fs::create_dir(&output).unwrap();
    println!("kind,count,operation,batch,repetitions,output_bytes,warnings,ns_per_op");
    for kind in ["hash", "hash-alias", "integer-control"] {
        for count in [1, 8, 128, 1024] {
            let item = match kind {
                "hash" => json!({"k":"array","max":32,"item":{"k":"u","bits":8}}),
                "hash-alias" => json!({"k":"ref","name":"Hash"}),
                _ => json!({"k":"u","bits":64}),
            };
            let mut types = BTreeMap::new();
            if kind == "hash-alias" {
                types.insert(
                    "Byte".into(),
                    json!({"k":"newtype","item":{"k":"u","bits":8}}),
                );
                types.insert("Hash".into(),json!({"k":"newtype","item":{"k":"array","max":32,"item":{"k":"ref","name":"Byte"}}}));
            }
            types.insert("Root".into(), json!({"k":"record","fields":(0..count).map(|i|json!({"name":format!("f{i}"),"value":{"k":"list","max":4096,"item":item}})).collect::<Vec<_>>()}));
            let roots = BTreeMap::from([("Root".into(), json!({"k":"ref","name":"Root"}))]);
            let report = lint_bulk_roots(&roots, &types, &BTreeMap::new()).unwrap();
            let generated =
                generate_registry(&types, &[], "./wire-runtime.js", &BTreeMap::new(), |r| {
                    black_box(r);
                })
                .unwrap();
            std::fs::write(output.join(format!("{kind}-{count}.ts")), &generated).unwrap();
            for operation in ["lint", "generation", "display"] {
                let output_bytes = if operation == "display" {
                    report.to_string().len()
                } else {
                    generated.len()
                };
                let run = || {
                    if operation == "lint" {
                        black_box(lint_bulk_roots(&roots, &types, &BTreeMap::new()).unwrap());
                    } else if operation == "display" {
                        black_box(report.to_string());
                    } else {
                        black_box(
                            generate_registry(
                                &types,
                                &[],
                                "./wire-runtime.js",
                                &BTreeMap::new(),
                                |r| {
                                    black_box(r);
                                },
                            )
                            .unwrap(),
                        );
                    }
                };
                for _ in 0..8 {
                    run();
                }
                let mut repetitions = 1;
                loop {
                    let start = Instant::now();
                    for _ in 0..repetitions {
                        run();
                    }
                    if start.elapsed().as_millis() >= 20 || repetitions >= 4096 {
                        break;
                    }
                    repetitions *= 2;
                }
                for batch in 0..11 {
                    let start = Instant::now();
                    for _ in 0..repetitions {
                        run();
                    }
                    println!(
                        "{kind},{count},{operation},{batch},{repetitions},{},{},{:.3}",
                        output_bytes,
                        report.warnings.len(),
                        start.elapsed().as_nanos() as f64 / f64::from(repetitions)
                    );
                }
            }
        }
    }
}
