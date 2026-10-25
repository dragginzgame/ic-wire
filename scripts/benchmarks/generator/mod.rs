//! Host-only generator timing probe; copy into ic-wire/examples to build.
use ic_wire::typescript::generate_registry;
use serde_json::json;
use std::{collections::BTreeMap, hint::black_box, time::Instant};

fn main() {
    for kind in ["record", "enum", "methods"] {
        for count in [8, 128, 1024] {
            let mut types = BTreeMap::new();
            let mut methods = Vec::new();
            if kind == "methods" {
                for index in 0..count {
                    methods.push(json!({"name":format!("method{index}"),"mode":"query","id":index.to_string(),
                        "request":{"k":"unit"},"response":{"k":"unit"},
                        "requestBytes":11,"responseBytes":11,"requestNodes":1,"responseNodes":1,
                        "requestDepth":1,"responseDepth":1}));
                }
            } else {
                let fields: Vec<_> = (0..count).map(|index| {
                    json!({"name":format!("field{index}"),"id":index,"value":{"k":"unit"}})
                }).collect();
                let node = if kind == "record" {
                    json!({"k":"record","fields":fields})
                } else {
                    json!({"k":"enum","variants":fields})
                };
                types.insert("Payload".into(), node);
            }
            let expected = generate_registry(&types, &methods, "./wire-runtime.js", &Default::default(), |report| { black_box(report); }).unwrap();
            if let Some(directory) = std::env::args().nth(1) {
                std::fs::write(format!("{directory}/{kind}-{count}.ts"), &expected).unwrap();
            }
            for _ in 0..100 {
                black_box(generate_registry(&types, &methods, "./wire-runtime.js", &Default::default(), |report| { black_box(report); }).unwrap());
            }
            let mut repetitions = 1;
            loop {
                let start = Instant::now();
                for _ in 0..repetitions {
                    black_box(generate_registry(&types, &methods, "./wire-runtime.js", &Default::default(), |report| { black_box(report); }).unwrap());
                }
                if start.elapsed().as_millis() >= 20 || repetitions >= 4096 {
                    break;
                }
                repetitions *= 2;
            }
            for batch in 0..11 {
                let start = Instant::now();
                for _ in 0..repetitions {
                    black_box(generate_registry(&types, &methods, "./wire-runtime.js", &Default::default(), |report| { black_box(report); }).unwrap());
                }
                println!(
                    "{kind},{count},{batch},{repetitions},{},{:.3}",
                    expected.len(),
                    start.elapsed().as_nanos() as f64 / f64::from(repetitions)
                );
            }
        }
    }
}
