//! Derived metadata goes through the same graph admission as explicit schemas.
#![cfg(all(feature = "derive", not(target_family = "wasm")))]
use ic_wire::typescript::{GenerationError, SchemaContext, SchemaLimits, ValidatedRegistry, Wire};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize, Wire)]
struct Unit;
#[derive(Serialize, Deserialize, Wire)]
struct Newtype(u16);
#[derive(Serialize, Deserialize, Wire)]
enum Choice {
    Empty,
    Value(u16),
    Record { flag: bool },
}
#[derive(Serialize, Deserialize, Wire)]
struct Record {
    r#type: u8,
    choice: Choice,
    ordinary: Vec<u8>,
    bytes: ic_wire::Bytes<3>,
}
#[derive(Serialize, Deserialize, Wire)]
struct Recursive {
    next: Option<Box<Recursive>>,
}

#[test]
fn derives_exact_ordinary_layouts_without_changing_serde() {
    let graph = ValidatedRegistry::from_type::<Record>("Record", SchemaLimits::default()).unwrap();
    let metadata: Value = serde_json::from_str(&graph.metadata_json().unwrap()).unwrap();
    assert_eq!(
        metadata["Record"],
        json!({"k":"record","fields":[
            {"name":"type","value":{"k":"u","bits":8}},
            {"name":"choice","value":{"k":"enum","variants":[
                {"name":"Empty","id":0,"value":{"k":"unit"}},
                {"name":"Value","id":1,"value":{"k":"u","bits":16}},
                {"name":"Record","id":2,"value":{"k":"record","fields":[{"name":"flag","value":{"k":"bool"}}]}}
            ]}},
            {"name":"ordinary","value":{"k":"list","item":{"k":"u","bits":8}}},
            {"name":"bytes","value":{"k":"bytes","max":3}}
        ]})
    );
    for (name, actual, expected) in [
        (
            "Unit",
            ValidatedRegistry::from_type::<Unit>("Unit", SchemaLimits::default()).unwrap(),
            json!({"k":"unit"}),
        ),
        (
            "Newtype",
            ValidatedRegistry::from_type::<Newtype>("Newtype", SchemaLimits::default()).unwrap(),
            json!({"k":"newtype","item":{"k":"u","bits":16}}),
        ),
    ] {
        assert_eq!(
            serde_json::from_str::<Value>(&actual.metadata_json().unwrap()).unwrap()[name],
            expected
        );
    }
}

#[test]
fn construction_limits_and_recursion_refuse_without_partial_registry() {
    assert_eq!(
        ValidatedRegistry::from_type::<Recursive>("Recursive", SchemaLimits::default())
            .unwrap_err(),
        GenerationError::Schema
    );
    assert_eq!(
        ValidatedRegistry::from_type::<Record>(
            "Record",
            SchemaLimits {
                nodes: 1,
                ..SchemaLimits::default()
            }
        )
        .unwrap_err(),
        GenerationError::Complexity
    );
    assert_eq!(
        ValidatedRegistry::from_type::<Record>(
            "Record",
            SchemaLimits {
                depth: 0,
                ..SchemaLimits::default()
            }
        )
        .unwrap_err(),
        GenerationError::Complexity
    );
    assert!(
        ValidatedRegistry::from_type::<bool>(
            "Flag",
            SchemaLimits {
                nodes: 2,
                depth: 0,
                ..SchemaLimits::default()
            }
        )
        .is_ok()
    );
}

struct SwallowsFailure;
impl Wire for SwallowsFailure {
    fn schema(context: &mut SchemaContext) -> Result<Value, GenerationError> {
        let _error = context.describe::<Recursive>();
        Ok(json!({"k":"bool"}))
    }
}
#[test]
fn manual_projection_cannot_hide_a_nested_admission_failure() {
    assert_eq!(
        ValidatedRegistry::from_type::<SwallowsFailure>("Root", SchemaLimits::default())
            .unwrap_err(),
        GenerationError::Schema
    );
}

#[derive(Serialize, Deserialize, Wire)]
struct Generic<Result: Clone = u16, const N: usize = 3>
where
    Result: PartialEq,
{
    value: Result,
    bytes: ic_wire::Bytes<N>,
}

#[derive(Serialize, Deserialize, Wire)]
enum GenericChoice<T> {
    Empty,
    Value(T),
    Named { value: T },
}

#[derive(Serialize, Deserialize, Wire)]
struct Borrowed<'a, T, const N: usize> {
    text: &'a str,
    #[serde(borrow = "'a")]
    bytes: ic_wire::BytesRef<'a, N>,
    value: T,
}

#[test]
fn concrete_generics_preserve_bounds_defaults_and_variant_layout() {
    fn metadata<T: Wire>() -> Value {
        serde_json::from_str(
            &ValidatedRegistry::from_type::<T>("Root", SchemaLimits::default())
                .unwrap()
                .metadata_json()
                .unwrap(),
        )
        .unwrap()
    }
    let small = metadata::<Generic>();
    let large = metadata::<Generic<u64, 16>>();
    assert_eq!(
        small["Root"]["fields"][0]["value"],
        json!({"k":"u","bits":16})
    );
    assert_eq!(
        large["Root"]["fields"][0]["value"],
        json!({"k":"u","bits":64})
    );
    assert_eq!(small["Root"]["fields"][1]["value"]["max"], 3);
    assert_eq!(large["Root"]["fields"][1]["value"]["max"], 16);
    let choice = metadata::<GenericChoice<u16>>();
    assert_eq!(
        choice["Root"]["variants"][1]["value"],
        json!({"k":"u","bits":16})
    );
    assert_eq!(
        choice["Root"]["variants"][2]["value"]["fields"][0]["value"],
        json!({"k":"u","bits":16})
    );
}

#[test]
fn borrowed_generic_projection_accepts_nonstatic_lifetimes_and_preserves_custody() {
    use ic_wire::{Encoding, Limits, decode};
    fn project<'a>(_: &'a str) -> ValidatedRegistry {
        ValidatedRegistry::from_type::<Borrowed<'a, u16, 3>>("Borrowed", SchemaLimits::default())
            .unwrap()
    }
    let text = String::from("hi");
    let graph = project(&text);
    let value: Value = serde_json::from_str(&graph.metadata_json().unwrap()).unwrap();
    assert_eq!(value["Borrowed"]["fields"][0]["value"], json!({"k":"text"}));
    assert_eq!(
        value["Borrowed"]["fields"][1]["value"],
        json!({"k":"bytes","max":3})
    );
    let literal = [2, b'h', b'i', 2, 170, 187, 128, 1];
    let decoded: Borrowed<'_, u16, 3> = decode(
        &literal,
        Encoding::Payload,
        Limits {
            bytes: 128,
            allocation: 32768,
            nodes: 512,
            depth: 32,
        },
    )
    .unwrap();
    assert_eq!(decoded.text, "hi");
    assert_eq!(decoded.value, 128);
    assert_eq!(decoded.text.as_ptr(), literal[1..].as_ptr());
    assert_eq!(decoded.bytes.as_slice().as_ptr(), literal[4..].as_ptr());
}
