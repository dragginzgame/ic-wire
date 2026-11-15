use super::{limits, run_typescript};
use serde_json::json;
use wire::typescript::{PayloadLimits, SchemaLimits, ValidatedRegistry};
use wire::{Encoding, FiniteF32, FiniteF64, decode, encode};

#[test]
fn finite_floats_preserve_bits_and_refuse_nonfinite_values() {
    let graph =
        ValidatedRegistry::from_type::<(FiniteF32, FiniteF64)>("Floats", SchemaLimits::default())
            .unwrap();
    let module = graph
        .payload_module(
            "codec",
            "Floats",
            "./runtime-adapter.ts",
            PayloadLimits {
                bytes: 12,
                nodes: 3,
                depth: 2,
            },
            &Default::default(),
            |report| eprint!("{report}"),
        )
        .unwrap();
    let literals = vec![
        vec![0; 12],
        vec![0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 128],
        vec![1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 192, 63, 0, 0, 0, 0, 0, 0, 248, 63],
    ];
    let output: Vec<Vec<u8>> = serde_json::from_value(run_typescript(
        &json!({"module":module,"literals":literals}),
        include_str!("floats.mjs"),
    ))
    .unwrap();
    assert_eq!(output, literals);
    for (bytes, (a, b)) in output.iter().zip([
        (0.0, 0.0),
        (-0.0, -0.0),
        (f32::from_bits(1), f64::from_bits(1)),
        (1.5, 1.5),
    ]) {
        let value = (FiniteF32::new(a).unwrap(), FiniteF64::new(b).unwrap());
        assert_eq!(encode(&value, Encoding::Payload, limits()).unwrap(), *bytes);
        let decoded = decode::<(FiniteF32, FiniteF64)>(bytes, Encoding::Payload, limits()).unwrap();
        assert_eq!(decoded.0.get().to_bits(), a.to_bits());
        assert_eq!(decoded.1.get().to_bits(), b.to_bits());
    }
    for value in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
        assert_eq!(FiniteF32::new(value), Err(wire::Error::Malformed));
        assert_eq!(
            decode::<FiniteF32>(&value.to_le_bytes(), Encoding::Payload, limits()),
            Err(wire::Error::Malformed)
        );
    }
    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        assert_eq!(FiniteF64::new(value), Err(wire::Error::Malformed));
        assert_eq!(
            decode::<FiniteF64>(&value.to_le_bytes(), Encoding::Payload, limits()),
            Err(wire::Error::Malformed)
        );
    }
}
