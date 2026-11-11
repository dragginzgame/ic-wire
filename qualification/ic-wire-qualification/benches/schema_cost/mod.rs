//! Host schema costs. Run matched sources with the same compiler/lock/profile.
use ic_wire::typescript::{generate_payload_registry, validate_payload_schema};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, hint::black_box, time::Instant};

#[global_allocator]
static ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;

fn main() {
    let source = Sha256::digest(concat!(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/ic-wire/src/typescript/metadata/mod.rs"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/ic-wire/src/typescript/bulk_lint/mod.rs"
        ))
    ))
    .iter()
    .map(|byte| format!("{byte:02x}"))
    .collect::<String>();
    for (name, width, depth, invalid, shared) in [
        ("small", 1, 0, false, false),
        ("large", 4096, 0, false, false),
        ("nested", 128, 32, false, false),
        ("invalid-unused", 4096, 0, true, false),
        ("shared-ref", 4096, 0, false, true),
        ("escaped", 4096, 0, false, false),
        ("many-types", 1, 0, false, false),
        ("proto-key", 128, 0, false, false),
    ] {
        let mut item = json!({"k":"text"});
        for _ in 0..depth {
            item = json!({"k":"option","item":item});
        }
        if shared {
            item = json!({"k":"ref","name":"Leaf"});
        }
        let mut types = BTreeMap::from([(
            "Record".into(),
            json!({"k":"record","fields":
                (0..width).map(|n| json!({"name":if name == "escaped" { format!("field/{n}~") } else { format!("field{n}") },"value":item})).collect::<Vec<_>>()
            }),
        )]);
        if shared {
            types.insert(
                "Leaf".into(),
                json!({"k":"record","fields":[
                    {"name":"values","value":{"k":"list","item":{"k":"u","bits":64}}},
                    {"name":"next","value":{"k":"option","item":{"k":"ref","name":"Leaf"}}}
                ]}),
            );
        }
        if name == "many-types" {
            for n in 0..256 {
                types.insert(format!("Type{n}"), json!({"k":"text"}));
            }
        }
        if name == "proto-key" {
            types.insert("__proto__".into(), json!({"k":"text"}));
        }
        if invalid {
            types.insert("ZInvalid".into(), json!({"k":"u","bits":7}));
        }
        let root = json!({"k":"ref","name":"Record"});
        let payloads =
            [json!({"name":"codec","schema":root,"bytes":65536,"nodes":65536,"depth":64})];
        let identity = Sha256::digest(serde_json::to_vec(&(&types, &root)).unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let generated = generate_payload_registry(
            &types,
            &payloads,
            "./runtime.js",
            &Default::default(),
            |_| {},
        )
        .map(|source| {
            Sha256::digest(source)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        });
        let validated = validate_payload_schema(&types, &root);
        assert_eq!(validated.is_err(), invalid);
        assert_eq!(generated.is_err(), invalid);
        for operation in ["validate", "generate"] {
            let run = || {
                if operation == "validate" {
                    black_box(validate_payload_schema(black_box(&types), black_box(&root)))
                        .map(|_| ())
                } else {
                    black_box(generate_payload_registry(
                        black_box(&types),
                        black_box(&payloads),
                        "./runtime.js",
                        &Default::default(),
                        |_| {},
                    ))
                    .map(|_| ())
                }
            };
            let region = stats_alloc::Region::new(ALLOCATOR);
            let _ = black_box(run());
            let allocations = region.change();
            let mut repetitions = 1;
            loop {
                let start = Instant::now();
                for _ in 0..repetitions {
                    let _ = black_box(run());
                }
                if start.elapsed().as_millis() >= 10 {
                    break;
                }
                repetitions *= 2;
            }
            let samples: Vec<Value> = (0..11)
                .map(|_| {
                    let start = Instant::now();
                    for _ in 0..repetitions {
                        let _ = black_box(run());
                    }
                    json!(start.elapsed().as_nanos() as f64 / f64::from(repetitions))
                })
                .collect();
            println!(
                "{}",
                json!({"source_sha256":source,"case":name,"operation":operation,"input_sha256":identity,"output":format!("{generated:?}"),"validation":format!("{validated:?}"),"repetitions":repetitions,"samples_ns":samples,"allocations":allocations.allocations,"allocated_bytes":allocations.bytes_allocated,"reallocations":allocations.reallocations})
            );
        }
    }
}
