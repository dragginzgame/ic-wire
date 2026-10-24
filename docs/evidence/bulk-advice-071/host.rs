//! Reproduction-only host timing probe; not a production example or codec.
use ic_wire::typescript::{generate_registry, lint_bulk_fields};
use serde_json::json;
use std::{collections::BTreeMap, hint::black_box, time::Instant};
fn main() {
    println!("types,phase,sample,repetitions,ns_per_op,warnings,generated_bytes");
    for count in [1, 10, 100] {
        let types = (0..count).map(|n| (format!("Record{n}"), json!({"k":"record","fields":[
            {"name":"words","value":{"k":"list","item":{"k":"i","bits":64}}},
            {"name":"bytes","value":{"k":"list","item":{"k":"u","bits":8}}}
        ]}))).collect::<BTreeMap<_,_>>();
        let acknowledgements = BTreeMap::new();
        let expected = generate_registry(&types, &[], "./wire-runtime.js").unwrap();
        assert_eq!(lint_bulk_fields(&types, &acknowledgements).unwrap().warnings.len(), count * 2);
        let repetitions = 100;
        for sample in 0..11 {
            // Alternate order, keeping generation identical between the two paths.
            for lint in if sample % 2 == 0 { [false, true] } else { [true, false] } {
                let start = Instant::now();
                for _ in 0..repetitions {
                    black_box(generate_registry(black_box(&types), &[], "./wire-runtime.js").unwrap());
                    if lint { black_box(lint_bulk_fields(black_box(&types), &acknowledgements).unwrap()); }
                }
                println!("{count},{},{sample},{repetitions},{},{},{}", if lint {"generation-plus-lint"} else {"generation"}, start.elapsed().as_nanos() as f64 / repetitions as f64, count * 2, expected.len());
            }
        }
    }
}
