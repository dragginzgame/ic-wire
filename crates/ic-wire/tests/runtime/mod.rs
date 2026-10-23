//! Runtime metadata and Serde field boundaries.
mod canonical_bytes;
#[cfg(feature = "typescript")]
mod native_schema;

#[cfg(feature = "typescript")]
#[test]
fn typescript_registry_rejects_invalid_names_shapes_and_excessive_metadata() {
    use ic_wire::typescript::{GenerationError, generate_registry};
    use serde_json::json;
    use std::collections::BTreeMap;
    let method = json!({"name":"echo","mode":"query","id":"1",
        "request":{"k":"ref","name":"Atom"},"response":{"k":"ref","name":"Atom"},
        "requestBytes":128,"responseBytes":128,"requestNodes":32,"responseNodes":32,
        "requestDepth":8,"responseDepth":8});
    let types = BTreeMap::from([(
        "Atom".into(),
        json!({"k":"text","check":"decimal","max":32}),
    )]);
    let source =
        generate_registry(&types, std::slice::from_ref(&method), "./native-runtime.js").unwrap();
    assert!(source.contains("\\\"check\\\":\\\"decimal\\\""));
    assert!(source.contains("export type Atom = string;"));
    for check in [json!(null), json!(7), json!(true), json!([]), json!({})] {
        for node in [
            json!({"k":"text","check":check}),
            json!({"k":"ref","name":"Atom","check":check}),
            json!({"k":"list","item":{"k":"unit","check":check}}),
        ] {
            let mut invalid = method.clone();
            invalid["request"] = node;
            assert_eq!(
                generate_registry(&types, &[invalid], "./runtime.js"),
                Err(GenerationError::Schema)
            );
        }
    }
    for name in [
        "bad-name",
        "default",
        "as",
        "string",
        "createCodec",
        "JSON",
        "Uint8Array",
        "Int16Array",
    ] {
        let mut invalid = method.clone();
        invalid["name"] = json!(name);
        assert_eq!(
            generate_registry(&types, &[invalid], "./runtime.js"),
            Err(GenerationError::Schema)
        );
        let invalid = BTreeMap::from([(name.into(), json!({"k":"unit"}))]);
        assert_eq!(
            generate_registry(&invalid, &[], "./runtime.js"),
            Err(GenerationError::Schema)
        );
    }
    for node in [
        json!({"k":"ref","name":"Missing"}),
        json!({"k":"u","bits":7}),
        json!({"k":"float","bits":16}),
        json!({"k":"tuple"}),
        json!({"k":"enum","variants":[]}),
        json!({"k":"enum","variants":[{"name":"A","id":1,"value":{"k":"unit"}},{"name":"B","id":2,"value":{"k":"unit"}},{"name":"A","id":3,"value":{"k":"unit"}}]}),
        json!({"k":"enum","variants":[{"name":"A","id":1,"value":{"k":"unit"}},{"name":"B","id":2,"value":{"k":"unit"}},{"name":"C","id":1,"value":{"k":"unit"}}]}),
        json!({"k":"record","fields":[{"name":"x","value":{"k":"unit"}},{"name":"x","value":{"k":"unit"}}]}),
    ] {
        let invalid = BTreeMap::from([("Atom".into(), node)]);
        assert_eq!(
            generate_registry(&invalid, &[], "./runtime.js"),
            Err(GenerationError::Schema)
        );
    }
    assert_eq!(
        generate_registry(&types, &[method.clone(), method.clone()], "./runtime.js"),
        Err(GenerationError::Schema)
    );
    let mut invalid = method;
    invalid["requestNodes"] = json!(1_u64 << 53);
    assert_eq!(
        generate_registry(&types, &[invalid], "./runtime.js"),
        Err(GenerationError::Complexity)
    );
    let mut deep = json!({"k":"unit"});
    for _ in 0..66 {
        deep = json!({"k":"option","item":deep});
    }
    assert_eq!(
        generate_registry(
            &BTreeMap::from([("Deep".into(), deep)]),
            &[],
            "./runtime.js"
        ),
        Err(GenerationError::Complexity)
    );
}

#[cfg(feature = "typescript")]
#[test]
fn typescript_registry_rejects_transparent_cycles_and_invalid_method_budgets() {
    use ic_wire::typescript::{GenerationError, generate_registry};
    use serde_json::json;
    use std::collections::BTreeMap;
    for types in [
        BTreeMap::from([("Loop".into(), json!({"k":"ref","name":"Loop"}))]),
        BTreeMap::from([
            ("A".into(), json!({"k":"ref","name":"B"})),
            ("B".into(), json!({"k":"ref","name":"A"})),
        ]),
        BTreeMap::from([(
            "Loop".into(),
            json!({"k":"newtype","item":{"k":"ref","name":"Loop"}}),
        )]),
    ] {
        assert_eq!(
            generate_registry(&types, &[], "./runtime.js"),
            Err(GenerationError::Schema)
        );
    }
    let method = json!({"name":"empty","mode":"query","id":"1",
        "request":{"k":"unit"},"response":{"k":"unit"},
        "requestBytes":11,"responseBytes":11,"requestNodes":1,"responseNodes":1,
        "requestDepth":1,"responseDepth":1});
    assert!(
        generate_registry(
            &BTreeMap::new(),
            std::slice::from_ref(&method),
            "./runtime.js"
        )
        .is_ok()
    );
    for (key, minimum) in [
        ("requestBytes", 11),
        ("responseBytes", 11),
        ("requestNodes", 1),
        ("responseNodes", 1),
        ("requestDepth", 1),
        ("responseDepth", 1),
    ] {
        for n in [0, minimum - 1] {
            let mut invalid = method.clone();
            invalid[key] = json!(n);
            assert_eq!(
                generate_registry(&BTreeMap::new(), &[invalid], "./runtime.js"),
                Err(GenerationError::Limits)
            );
        }
    }
}

#[cfg(feature = "typescript")]
#[test]
fn typescript_registry_requires_an_exact_array_length() {
    use ic_wire::typescript::{GenerationError, generate_registry};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;
    for length in [
        None,
        Some(Value::Null),
        Some(json!(-1)),
        Some(json!(1.5)),
        Some(json!("2")),
    ] {
        let mut node = json!({"k":"array","item":{"k":"unit"}});
        if let Some(length) = length {
            node["max"] = length;
        }
        assert_eq!(
            generate_registry(
                &BTreeMap::from([("ArrayValue".into(), node)]),
                &[],
                "./runtime.js"
            ),
            Err(GenerationError::Schema)
        );
    }
    for length in [0, 2, (1_u64 << 53) - 1, 1_u64 << 53] {
        let types = BTreeMap::from([(
            "ArrayValue".into(),
            json!({"k":"array","max":length,"item":{"k":"unit"}}),
        )]);
        let result = generate_registry(&types, &[], "./runtime.js");
        if length == 1_u64 << 53 {
            assert_eq!(result, Err(GenerationError::Complexity));
        } else {
            assert!(result.is_ok());
        }
    }
}

#[cfg(feature = "typescript")]
#[test]
fn typescript_registry_rejects_invalid_optional_collection_bounds() {
    use ic_wire::typescript::{GenerationError, generate_registry};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;
    for kind in ["bytes", "text", "list"] {
        for bound in [
            Value::Null,
            json!(-1),
            json!(1.5),
            json!("2"),
            json!(1_u64 << 53),
        ] {
            let node = json!({"k":kind,"max":bound,"item":{"k":"unit"}});
            assert!(
                generate_registry(
                    &BTreeMap::from([("Bounded".into(), node)]),
                    &[],
                    "./runtime.js"
                )
                .is_err()
            );
        }
        for bound in [None, Some(json!(0)), Some(json!(2))] {
            let mut node = json!({"k":kind,"item":{"k":"unit"}});
            if let Some(bound) = bound {
                node["max"] = bound;
            }
            assert!(
                generate_registry(
                    &BTreeMap::from([("Bounded".into(), node)]),
                    &[],
                    "./runtime.js"
                )
                .is_ok()
            );
        }
    }
    for (bound, valid) in [
        (None, false),
        (Some(0), true),
        (Some(2), true),
        (Some((1_u64 << 52) - 1), true),
        (Some(1_u64 << 52), false),
    ] {
        let mut node = json!({"k":"packed-i16"});
        if let Some(bound) = bound {
            node["max"] = json!(bound);
        }
        assert_eq!(
            generate_registry(
                &BTreeMap::from([("Packed".into(), node)]),
                &[],
                "./runtime.js"
            )
            .is_ok(),
            valid
        );
    }
    for minimum in [Value::Null, json!(-1), json!(1.5), json!("2"), json!(3)] {
        let node = json!({"k":"bytes","min":minimum,"max":2});
        assert_eq!(
            generate_registry(
                &BTreeMap::from([("Bounded".into(), node)]),
                &[],
                "./runtime.js"
            ),
            Err(GenerationError::Schema)
        );
    }
}

#[cfg(feature = "typescript")]
#[test]
fn typescript_registry_validates_float_bounds() {
    use ic_wire::typescript::{GenerationError, generate_registry};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;
    for bits in [32, 64] {
        for key in ["min", "max"] {
            for invalid in [Value::Null, json!("1"), json!(true), json!([])] {
                let mut node = json!({"k":"float","bits":bits});
                node[key] = invalid;
                assert_eq!(
                    generate_registry(
                        &BTreeMap::from([("Scalar".into(), node)]),
                        &[],
                        "./runtime.js"
                    ),
                    Err(GenerationError::Schema)
                );
            }
        }
        for (min, max, valid) in [
            (0.0, 1.0, true),
            (0.5, 0.5, true),
            (-1.0, 1.0, true),
            (1.0, 0.0, false),
        ] {
            let node = json!({"k":"float","bits":bits,"min":min,"max":max});
            assert_eq!(
                generate_registry(
                    &BTreeMap::from([("Scalar".into(), node)]),
                    &[],
                    "./runtime.js"
                )
                .is_ok(),
                valid
            );
        }
    }
}

#[cfg(feature = "typescript")]
#[test]
fn typescript_registry_validates_integer_bounds_without_precision_loss() {
    use ic_wire::typescript::{GenerationError, generate_registry};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;
    for kind in ["u", "i"] {
        for bits in [8, 16, 32, 64, 128] {
            for key in ["min", "max"] {
                for (bound, error) in [
                    (Value::Null, GenerationError::Schema),
                    (json!("1"), GenerationError::Schema),
                    (json!(true), GenerationError::Schema),
                    (json!(1.5), GenerationError::Schema),
                    (
                        json!(9_007_199_254_740_993_u64),
                        GenerationError::Complexity,
                    ),
                    (
                        json!(-9_007_199_254_740_993_i64),
                        GenerationError::Complexity,
                    ),
                ] {
                    let mut node = json!({"k":kind,"bits":bits});
                    node[key] = bound;
                    assert_eq!(
                        generate_registry(
                            &BTreeMap::from([("Scalar".into(), node)]),
                            &[],
                            "./runtime.js"
                        ),
                        Err(error)
                    );
                }
            }
            for (min, max, valid) in [
                (0.0, 1.0, true),
                (1.0, 1.0, true),
                (-9_007_199_254_740_991.0, 9_007_199_254_740_991.0, true),
                (2.0, 1.0, false),
            ] {
                let node = json!({"k":kind,"bits":bits,"min":min,"max":max});
                assert_eq!(
                    generate_registry(
                        &BTreeMap::from([("Scalar".into(), node)]),
                        &[],
                        "./runtime.js"
                    )
                    .is_ok(),
                    valid
                );
            }
        }
    }
}

#[test]
fn list_count_rejects_before_element_decoder() {
    struct Never;
    impl<'de> serde::Deserialize<'de> for Never {
        fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            panic!("element decoded before limit check")
        }
    }
    assert!(postcard::from_bytes::<ic_wire::List<Never, 2>>(&[3, 0, 0, 0]).is_err());
}

#[test]
fn finite_float_bits_and_nonfinite_rejection() {
    use ic_wire::{Error, Limits, native};
    use ic_wire::{FiniteF32, FiniteF64};
    const LIMITS: Limits = Limits {
        bytes: 512,
        allocation: 65536,
        nodes: 128,
        depth: 32,
    };
    for value in [
        0.0_f32,
        -0.0,
        f32::from_bits(1),
        f32::MIN_POSITIVE,
        f32::MIN,
        f32::MAX,
        0.1,
        -2.5,
    ] {
        let bytes =
            native::encode_request::<FiniteF32>(0xf10a7, &FiniteF32::new(value).unwrap(), LIMITS)
                .unwrap();
        assert_eq!(&bytes.as_slice()[11..], &value.to_le_bytes());
        assert_eq!(
            native::decode_request::<FiniteF32>(0xf10a7, bytes.as_slice(), LIMITS)
                .unwrap()
                .get()
                .to_bits(),
            value.to_bits()
        );
    }
    for value in [
        0.0_f64,
        -0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::MIN,
        f64::MAX,
        0.1,
        -2.5,
    ] {
        let bytes =
            native::encode_response::<FiniteF64>(0xf10a7, &FiniteF64::new(value).unwrap(), LIMITS)
                .unwrap();
        assert_eq!(&bytes.as_slice()[11..], &value.to_le_bytes());
        assert_eq!(
            native::decode_response::<FiniteF64>(0xf10a7, bytes.as_slice(), LIMITS)
                .unwrap()
                .get()
                .to_bits(),
            value.to_bits()
        );
    }
    for bits in [
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_0000,
        0x7f80_0001,
        0xffc0_0123,
    ] {
        let value = f32::from_bits(bits);
        assert_eq!(FiniteF32::new(value), Err(Error::Malformed));
        let mut bytes =
            native::encode_request::<FiniteF32>(0xf10a7, &FiniteF32::new(0.0).unwrap(), LIMITS)
                .unwrap();
        bytes[11..].copy_from_slice(&bits.to_le_bytes());
        assert_eq!(
            native::decode_request::<FiniteF32>(0xf10a7, &bytes, LIMITS),
            Err(Error::Malformed)
        );
    }
    for bits in [
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_0000,
        0x7ff0_0000_0000_0001,
        0xfff8_0000_0000_0123,
    ] {
        let value = f64::from_bits(bits);
        assert_eq!(FiniteF64::new(value), Err(Error::Malformed));
        let mut bytes =
            native::encode_response::<FiniteF64>(0xf10a7, &FiniteF64::new(0.0).unwrap(), LIMITS)
                .unwrap();
        bytes[11..].copy_from_slice(&bits.to_le_bytes());
        assert_eq!(
            native::decode_response::<FiniteF64>(0xf10a7, &bytes, LIMITS),
            Err(Error::Malformed)
        );
    }
}
