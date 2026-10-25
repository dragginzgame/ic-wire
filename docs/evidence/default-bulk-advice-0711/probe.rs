use ic_wire::typescript::generate_registry;
use serde_json::{Value, json};
use std::{collections::BTreeMap, hint::black_box, time::Instant};
fn generate(types: &BTreeMap<String, Value>) -> String {
    #[cfg(feature = "candidate")]
    { generate_registry(types, &[], "./wire-runtime.js", &BTreeMap::new(), |report| { black_box(report); }).unwrap() }
    #[cfg(not(feature = "candidate"))]
    { generate_registry(types, &[], "./wire-runtime.js").unwrap() }
}
fn main() {
    let output = std::env::args().nth(1).unwrap();
    for bulk in [false, true] { for count in [1, 100, 1000] {
        let fields: Vec<_> = (0..count).map(|i| json!({"name":format!("field{i}"),"value":if bulk {json!({"k":"list","item":{"k":"u","bits":8}})} else {json!({"k":"u","bits":64})}})).collect();
        let types = BTreeMap::from([("Record".into(),json!({"k":"record","fields":fields}))]);
        let expected=generate(&types);
        std::fs::write(format!("{output}/{bulk}-{count}.ts"), &expected).unwrap();
        for _ in 0..30 {black_box(generate(&types));}
        let mut reps=1;
        loop {let start=Instant::now();for _ in 0..reps {black_box(generate(&types));} if start.elapsed().as_millis()>=10 || reps>=2048 {break;} reps*=2;}
        for sample in 0..11 {let start=Instant::now();for _ in 0..reps {black_box(generate(&types));} println!("{bulk},{count},{sample},{reps},{:.3}",start.elapsed().as_nanos() as f64 / f64::from(reps));}
    }}
}
