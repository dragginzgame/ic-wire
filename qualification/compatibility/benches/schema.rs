//! Bounded paired host-schema timing probe; not a release performance gate.
use candidate::typescript::{SchemaLimits, SchemaRegistry, Wire};
use serde_json::{Value, json};
use std::{hint::black_box, time::Instant};

#[derive(Wire, serde::Serialize)]
#[wire(crate = "candidate")]
struct Chain {
    value: u16,
    next: Option<Box<Chain>>,
}
impl reference::typescript::Wire for Chain {
    fn native_schema(
        context: &mut reference::typescript::SchemaContext,
    ) -> Result<Value, reference::typescript::GenerationError> {
        Ok(json!({"k":"record","fields":[
            {"name":"value","value":context.describe::<u16>()?},
            {"name":"next","value":context.describe::<Option<Box<Chain>>>()?}
        ]}))
    }
}
struct Sample {
    nanos: [u128; 3],
    json: String,
}
fn candidate<T: Wire>(names: &[String]) -> Sample {
    let start = Instant::now();
    let mut registry = SchemaRegistry::new(SchemaLimits::default()).unwrap();
    for name in names {
        registry.register::<T>(black_box(name)).unwrap();
    }
    let collected = start.elapsed().as_nanos();
    let start = Instant::now();
    let graph = black_box(registry).finish().unwrap();
    let finished = start.elapsed().as_nanos();
    let start = Instant::now();
    let json = black_box(&graph).metadata_json().unwrap();
    let exported = start.elapsed().as_nanos();
    Sample {
        nanos: [collected, finished, exported],
        json,
    }
}
fn reference<T: reference::typescript::Wire>(names: &[String]) -> Sample {
    let start = Instant::now();
    let mut registry = reference::typescript::SchemaRegistry::new();
    for name in names {
        black_box(registry.register::<T>(black_box(name)).unwrap());
    }
    let collected = start.elapsed().as_nanos();
    let start = Instant::now();
    let graph = black_box(registry).finish().unwrap();
    let finished = start.elapsed().as_nanos();
    let start = Instant::now();
    let json = serde_json::to_string(black_box(&graph)).unwrap();
    let exported = start.elapsed().as_nanos();
    Sample {
        nanos: [collected, finished, exported],
        json,
    }
}
fn summary(mut values: Vec<u128>) -> Value {
    values.sort_unstable();
    json!({"min_ns":values[0], "median_ns":values[values.len()/2], "max_ns":values[values.len()-1]})
}
fn compare<T: Wire + reference::typescript::Wire>(shape: &str, count: usize) {
    let names: Vec<_> = (0..count).map(|i| format!("Root{i:04}")).collect();
    let mut candidate_samples = Vec::new();
    let mut reference_samples = Vec::new();
    let mut sizes = [0; 2];
    for round in 0..24 {
        let (a, b) = if round % 2 == 0 {
            (candidate::<T>(&names), reference::<T>(&names))
        } else {
            let b = reference::<T>(&names);
            (candidate::<T>(&names), b)
        };
        // Qualification and parsing are outside the timed phases.
        assert_eq!(
            serde_json::from_str::<Value>(&a.json).unwrap(),
            serde_json::from_str::<Value>(&b.json).unwrap()
        );
        sizes = [a.json.len(), b.json.len()];
        if round >= 3 {
            candidate_samples.push(a.nanos);
            reference_samples.push(b.nanos);
        }
    }
    for (index, phase) in ["collect", "finish", "export", "total"].iter().enumerate() {
        let timing = |sample: &[u128; 3]| {
            if index == 3 {
                sample.iter().sum()
            } else {
                sample[index]
            }
        };
        println!(
            "{}",
            json!({"shape":shape,"roots":count,"phase":phase,"samples":21,
            "candidate":summary(candidate_samples.iter().map(timing).collect()),
            "reference":summary(reference_samples.iter().map(timing).collect()),
            "candidate_json_bytes":sizes[0],"reference_json_bytes":sizes[1]})
        );
    }
}
fn main() {
    for count in [1, 128, 1024] {
        compare::<bool>("scalar", count);
        compare::<Chain>("recursive", count);
    }
}
