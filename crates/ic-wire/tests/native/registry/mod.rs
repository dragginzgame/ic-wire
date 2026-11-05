use ic_wire::{
    Wire,
    typescript::{self, GenerationError, SchemaContext, SchemaRegistry},
};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize, Wire)]
enum Tree<T = u8> {
    Empty,
    Children(Vec<Tree<T>>),
    Optional(Option<Box<Tree<T>>>),
    Value(T),
}
#[derive(Serialize, Wire)]
struct Left {
    right: Option<Box<Right>>,
}
#[derive(Serialize, Wire)]
struct Right {
    left: Vec<Left>,
}
#[derive(Serialize, Wire)]
struct Pair<T> {
    item: T,
}
#[derive(Serialize, Wire)]
struct Roots {
    a: Pair<u8>,
    b: Pair<u64>,
    left: Left,
    tree: Tree,
}

#[test]
fn finite_recursive_registries_preserve_inline_behavior_and_generic_shapes() {
    let _: Tree = Tree::Optional(Some(Box::new(Tree::Children(vec![Tree::Empty]))));
    let _ = Tree::Value(7_u8);
    assert_eq!(
        typescript::native_schema::<Tree>(),
        Err(GenerationError::Complexity)
    );
    let mut registry = SchemaRegistry::new();
    assert_eq!(
        registry.register::<Roots>("Roots").unwrap(),
        json!({"k":"ref","name":"Roots"})
    );
    registry.register::<Pair<u8>>("SmallPage").unwrap();
    registry.register::<Tree<u64>>("WideTree").unwrap();
    let types = registry.finish().unwrap();
    typescript::generate_registry(&types, &[], "./runtime.js", &Default::default(), |report| {
        eprint!("{report}")
    })
    .unwrap();
    let fields = types["Roots"]["fields"].as_array().unwrap();
    let a = fields[0]["value"]["name"].as_str().unwrap();
    let b = fields[1]["value"]["name"].as_str().unwrap();
    assert_ne!(a, b);
    let a_scalar = types[a]["fields"][0]["value"]["name"].as_str().unwrap();
    let b_scalar = types[b]["fields"][0]["value"]["name"].as_str().unwrap();
    assert_eq!(types[a_scalar]["bits"], 8);
    assert_eq!(types[b_scalar]["bits"], 64);
}

#[test]
fn duplicate_names_and_rebinding_poison_the_collector() {
    for same_type in [false, true] {
        let mut registry = SchemaRegistry::new();
        registry.register::<Pair<u8>>("Page").unwrap();
        let result = if same_type {
            registry.register::<Pair<u8>>("Page")
        } else {
            registry.register::<Pair<u64>>("Page")
        };
        assert_eq!(result, Err(GenerationError::Schema));
        assert_eq!(registry.finish(), Err(GenerationError::Schema));
    }
    let mut registry = SchemaRegistry::new();
    assert_eq!(
        registry.register::<u8>("bad-name"),
        Err(GenerationError::Schema)
    );
    assert_eq!(registry.finish(), Err(GenerationError::Schema));
}

#[derive(Serialize)]
struct Unresolved;
impl typescript::Wire for Unresolved {
    fn native_schema(_: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"ref","name":"Missing"}))
    }
}
#[derive(Serialize)]
struct InlineLoop;
impl typescript::Wire for InlineLoop {
    fn native_schema(context: &mut SchemaContext) -> Result<Value, GenerationError> {
        context.describe::<Self>()
    }
}
#[test]
fn unresolved_custom_metadata_is_rejected_at_finish() {
    let mut registry = SchemaRegistry::new();
    registry.register::<Unresolved>("Root").unwrap();
    assert_eq!(registry.finish(), Err(GenerationError::Schema));
    let mut registry = SchemaRegistry::new();
    registry.register::<InlineLoop>("Loop").unwrap();
    assert_eq!(registry.finish(), Err(GenerationError::Schema));
}

#[derive(Serialize)]
struct Changing;
impl typescript::Wire for Changing {
    fn native_schema(_: &mut SchemaContext) -> Result<Value, GenerationError> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        Ok(json!({"k":"u","bits":if CALLS.fetch_add(1, Ordering::Relaxed) == 0 {8} else {16}}))
    }
}
#[derive(Serialize)]
struct TooDeep;
impl typescript::Wire for TooDeep {
    fn native_schema(_: &mut SchemaContext) -> Result<Value, GenerationError> {
        let mut node = json!({"k":"unit"});
        for _ in 0..65 {
            node = json!({"k":"option","item":node});
        }
        Ok(node)
    }
}
#[test]
fn conflicting_projections_and_excessive_shapes_fail_closed() {
    let mut registry = SchemaRegistry::new();
    registry.register::<Changing>("Changing").unwrap();
    assert_eq!(registry.finish(), Err(GenerationError::Schema));
    let mut registry = SchemaRegistry::new();
    registry.register::<TooDeep>("Deep").unwrap();
    assert_eq!(registry.finish(), Err(GenerationError::Complexity));
}

#[test]
fn directional_limit_projection_checks_exact_numbers_without_heap_claims() {
    let request = super::LIMITS;
    let response = ic_wire::Limits {
        bytes: 17,
        nodes: 23,
        depth: 5,
        allocation: 0,
    };
    let projected = typescript::method_limits(request, response).unwrap();
    assert_eq!(
        projected,
        json!({"requestBytes":4096,"requestNodes":4096,"requestDepth":64,
        "responseBytes":17,"responseNodes":23,"responseDepth":5})
    );
    for invalid in [
        ic_wire::Limits {
            bytes: 10,
            ..request
        },
        ic_wire::Limits {
            nodes: 0,
            ..request
        },
        ic_wire::Limits {
            depth: 0,
            ..request
        },
    ] {
        assert_eq!(
            typescript::method_limits(invalid, response),
            Err(GenerationError::Limits)
        );
        assert_eq!(
            typescript::method_limits(request, invalid),
            Err(GenerationError::Limits)
        );
    }
    #[cfg(target_pointer_width = "64")]
    {
        let exact = ic_wire::Limits {
            bytes: (1 << 53) - 1,
            nodes: (1 << 53) - 1,
            depth: (1 << 53) - 1,
            allocation: 0,
        };
        let projected = typescript::method_limits(exact, exact).unwrap();
        assert!(
            projected
                .as_object()
                .unwrap()
                .values()
                .all(|n| n.as_u64() == Some((1 << 53) - 1))
        );
    }
    #[cfg(target_pointer_width = "64")]
    for invalid in [
        ic_wire::Limits {
            bytes: 1 << 53,
            ..request
        },
        ic_wire::Limits {
            nodes: 1 << 53,
            ..request
        },
        ic_wire::Limits {
            depth: 1 << 53,
            ..request
        },
    ] {
        assert_eq!(
            typescript::method_limits(invalid, response),
            Err(GenerationError::Complexity)
        );
        assert_eq!(
            typescript::method_limits(request, invalid),
            Err(GenerationError::Complexity)
        );
    }
}

#[derive(Serialize, Wire)]
struct ByteAdapter<T: serde_bytes::Serialize> {
    #[serde(with = "serde_bytes")]
    value: T,
}

#[test]
fn optional_byte_adapters_follow_types_and_preserve_option_tags() {
    type Cursor = Option<Vec<u8>>;
    let schema = typescript::native_schema::<ByteAdapter<Cursor>>().unwrap();
    assert_eq!(
        schema["fields"][0]["value"],
        json!({"k":"option","item":{"k":"bytes"}})
    );
    assert_eq!(
        typescript::native_schema::<ByteAdapter<&Cursor>>().unwrap(),
        schema
    );
    assert_eq!(
        typescript::native_schema::<ByteAdapter<Option<Cursor>>>().unwrap()["fields"][0]["value"],
        json!({"k":"option","item":{"k":"option","item":{"k":"bytes"}}})
    );
    assert_eq!(
        typescript::native_schema::<ByteAdapter<Vec<u8>>>().unwrap()["fields"][0]["value"],
        json!({"k":"bytes"})
    );
    for (value, expected) in [
        (None, vec![0]),
        (Some(vec![]), vec![1, 0]),
        (Some(vec![0, 128, 255]), vec![1, 3, 0, 128, 255]),
    ] {
        assert_eq!(
            ic_wire::native::encode(&ByteAdapter { value }, super::LIMITS).unwrap(),
            expected
        );
    }
    let mut registry = SchemaRegistry::new();
    registry.register::<ByteAdapter<Cursor>>("Request").unwrap();
    typescript::generate_registry(
        &registry.finish().unwrap(),
        &[],
        "./runtime.js",
        &Default::default(),
        |report| eprint!("{report}"),
    )
    .unwrap();
}

#[derive(Serialize, Wire)]
enum RenamedValue {
    #[serde(rename = "Int")]
    Int64(i64),
    List(Vec<Self>),
}

#[test]
fn renamed_recursive_variants_keep_declaration_ordinals() {
    let value = RenamedValue::List(vec![RenamedValue::Int64(-7)]);
    assert_eq!(
        ic_wire::native::encode(&value, super::LIMITS).unwrap(),
        [1, 1, 0, 13]
    );
    let mut registry = SchemaRegistry::new();
    registry.register::<RenamedValue>("Value").unwrap();
    let types = registry.finish().unwrap();
    assert_eq!(types["Value"]["variants"][0]["name"], "Int");
    assert_eq!(types["Value"]["variants"][0]["id"], 0);
    typescript::generate_registry(&types, &[], "./runtime.js", &Default::default(), |report| {
        eprint!("{report}")
    })
    .unwrap();
}

#[test]
fn registry_rejects_numeric_method_id_collisions_before_emitting_diagnostics() {
    let method = |name: &str, id: &str, mode: &str, kind: &str| {
        json!({
            "name":name,"id":id,"mode":mode,
            "request":{"k":kind,"bits":8},"response":{"k":kind,"bits":8},
            "requestBytes":128,"responseBytes":128,"requestNodes":32,"responseNodes":32,
            "requestDepth":8,"responseDepth":8
        })
    };
    // Identity does not include the payload schema: this demonstrates why the
    // host must reject collisions rather than expecting the runtime to infer types.
    let limits = ic_wire::Limits {
        bytes: 128,
        allocation: 4096,
        nodes: 32,
        depth: 8,
    };
    let frame = ic_wire::native::encode_request(7, &255_u8, limits).unwrap();
    assert_eq!(
        ic_wire::native::decode_request::<i8>(7, &frame, limits),
        Ok(-1)
    );
    assert_eq!(
        ic_wire::native::decode_request::<i8>(8, &frame, limits),
        Err(ic_wire::Error::Identity)
    );
    for (first, second) in [
        ("7", "7"),
        ("7", "07"),
        ("7", "+7"),
        ("0", "00"),
        ("18446744073709551615", "018446744073709551615"),
    ] {
        for mode in ["query", "update"] {
            assert_eq!(
                typescript::generate_registry(
                    &Default::default(),
                    &[
                        method("unsigned", first, "query", "u"),
                        method("signed", second, mode, "i")
                    ],
                    "./runtime.js",
                    &Default::default(),
                    |_| panic!("invalid registry emitted diagnostics"),
                ),
                Err(GenerationError::Schema)
            );
        }
    }
    let methods = [
        method("zero", "0", "query", "u"),
        method("seven", "7", "query", "u"),
        method("eight", "8", "update", "i"),
        method("maximum", "18446744073709551615", "query", "u"),
    ];
    let mut reports = 0;
    typescript::generate_registry(
        &Default::default(),
        &methods,
        "./runtime.js",
        &Default::default(),
        |report| {
            assert!(report.is_clean());
            reports += 1;
        },
    )
    .unwrap();
    assert_eq!(reports, 1);
}
