//! Runtime contract behavior and rejection boundaries.
mod boxed;
mod canonical_bytes;
use ic_wire::{
    Bytes, Error, Limits, List, Method, Text, Wire, decode_request, decode_response,
    encode_request, encode_response,
};

#[derive(Debug, PartialEq, Wire)]
struct Grant {
    #[wire(id = 20)]
    label: Text<32>,
    #[wire(id = 10)]
    subject: Bytes<29>,
}
#[derive(Debug, PartialEq, Wire)]
struct Request {
    #[wire(id = 2)]
    grants: List<Grant, 4>,
    #[wire(id = 1)]
    expiry: Option<u64>,
}
#[derive(Debug, PartialEq, Wire)]
enum Failure {
    #[wire(id = 40)]
    Denied,
    #[wire(id = 9)]
    Detail(Text<32>),
}
struct Read;
const LIMITS: Limits = Limits {
    bytes: 512,
    allocation: 2048,
    nodes: 64,
    depth: 8,
};
impl Method for Read {
    type Request = Request;
    type Response = Result<u64, Failure>;
    const ID: u64 = 7;
    const NAME: &'static str = "read";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}
#[test]
fn nested_roundtrip_and_typed_failures() {
    let request = Request {
        expiry: Some(u64::MAX),
        grants: List::new(vec![Grant {
            label: Text::new("é".into()).unwrap(),
            subject: Bytes::new(vec![1; 29]).unwrap(),
        }])
        .unwrap(),
    };
    let bytes = encode_request::<Read>(&request).unwrap();
    assert_eq!(decode_request::<Read>(bytes.as_bytes()).unwrap(), request);
    for response in [
        Ok(u64::MAX),
        Err(Failure::Denied),
        Err(Failure::Detail(Text::new("reason".into()).unwrap())),
    ] {
        let reply = encode_response::<Read>(&response).unwrap();
        assert_eq!(decode_response::<Read>(reply.as_bytes()).unwrap(), response);
        assert_eq!(
            decode_request::<Read>(reply.as_bytes()),
            Err(Error::Identity)
        );
    }
    for end in 0..bytes.as_bytes().len() {
        assert!(decode_request::<Read>(&bytes.as_bytes()[..end]).is_err());
    }
    let mut trailing = bytes.as_bytes().to_vec();
    trailing.push(0);
    assert_eq!(decode_request::<Read>(&trailing), Err(Error::Trailing));
}
#[test]
fn retry_retains_exact_request_and_separates_transport_failure() {
    let value = Request {
        expiry: Some(42),
        grants: List::new(vec![]).unwrap(),
    };
    let request = encode_request::<Read>(&value).unwrap();
    let retained = request.as_bytes().to_vec();
    // A consumer retries the same owned frame; the codec owns no retry policy.
    let transport = |bytes: &[u8], fail: bool| -> Result<Vec<u8>, &'static str> {
        assert_eq!(bytes, retained);
        if fail {
            return Err("transport unavailable");
        }
        assert_eq!(decode_request::<Read>(bytes).unwrap(), value);
        Ok(encode_response::<Read>(&Err(Failure::Denied))
            .unwrap()
            .into_bytes())
    };
    assert_eq!(
        transport(request.as_bytes(), true),
        Err("transport unavailable")
    );
    let reply = transport(request.as_bytes(), false).unwrap();
    assert_eq!(decode_response::<Read>(&reply), Ok(Err(Failure::Denied)));
    assert_eq!(request.into_bytes(), retained);
}

#[derive(Debug, PartialEq, Wire)]
struct First {
    #[wire(id = 1)]
    a: u64,
    #[wire(id = 2)]
    b: bool,
}
#[derive(Debug, PartialEq, Wire)]
struct Reordered {
    #[wire(id = 2)]
    b: bool,
    #[wire(id = 1)]
    a: u64,
}
#[test]
fn declaration_order_does_not_own_wire_order() {
    assert_eq!(
        postcard::to_allocvec(&First { a: 128, b: true }).unwrap(),
        vec![128, 1, 1]
    );
    assert_eq!(
        postcard::to_allocvec(&First { a: 128, b: true }).unwrap(),
        postcard::to_allocvec(&Reordered { b: true, a: 128 }).unwrap()
    );
    assert_eq!(postcard::to_allocvec(&Failure::Denied).unwrap(), vec![40]);
    assert_eq!(
        postcard::to_allocvec(&Failure::Detail(Text::new("x".into()).unwrap())).unwrap(),
        vec![9, 1, b'x']
    );
}
#[test]
fn bounds_and_malformed_lengths_reject() {
    let bytes = vec![1, 2, 3];
    let original = bytes.as_ptr();
    let recovered = Bytes::<3>::new(bytes).unwrap().into_vec();
    assert_eq!(recovered, [1, 2, 3]);
    assert_eq!(recovered.as_ptr(), original);
    let text = String::from("é");
    let original = text.as_ptr();
    let recovered = Text::<2>::new(text).unwrap().into_string();
    assert_eq!(recovered, "é");
    assert_eq!(recovered.as_ptr(), original);
    assert!(List::<(), 2>::new(vec![(); 3]).is_err());
    assert!(postcard::from_bytes::<List<(), 2>>(&[3]).is_err());
    assert!(postcard::from_bytes::<Bytes<2>>(&[3, 1, 2, 3]).is_err());
    assert!(postcard::from_bytes::<Text<4>>(&[1, 0xff]).is_err());
    assert!(postcard::from_bytes::<Failure>(&[77]).is_err());
}

struct Scalar;
impl Method for Scalar {
    type Request = u64;
    type Response = ();
    const ID: u64 = 8;
    const NAME: &'static str = "scalar";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}
#[test]
fn frozen_envelope_empty_reply_and_noncanonical_rejection() {
    let request = encode_request::<Scalar>(&1).unwrap();
    assert_eq!(
        request.as_bytes(),
        &[b'I', b'W', 8, 0, 0, 0, 0, 0, 0, 0, 0, 1]
    );
    let response = encode_response::<Scalar>(&()).unwrap();
    assert_eq!(ic_wire::encoded_request_len::<Scalar>(&1), Ok(12));
    assert_eq!(ic_wire::encoded_response_len::<Scalar>(&()), Ok(11));
    assert_eq!(
        response.as_bytes(),
        &[b'I', b'W', 8, 0, 0, 0, 0, 0, 0, 0, 1]
    );
    assert_eq!(decode_response::<Scalar>(response.as_bytes()), Ok(()));
    let mut noncanonical = request.as_bytes().to_vec();
    *noncanonical.last_mut().unwrap() = 0x81;
    noncanonical.push(0);
    assert_eq!(
        decode_request::<Scalar>(&noncanonical),
        Err(Error::Noncanonical)
    );
    let mut wrong = request.as_bytes().to_vec();
    wrong[2] = 9;
    assert_eq!(decode_request::<Scalar>(&wrong), Err(Error::Identity));
    wrong = request.as_bytes().to_vec();
    wrong[0] = 0;
    assert_eq!(decode_request::<Scalar>(&wrong), Err(Error::Identity));
}
struct Budget<const BYTES: usize, const ALLOC: usize, const NODES: usize, const DEPTH: usize>;
impl<const B: usize, const A: usize, const N: usize, const D: usize> Method for Budget<B, A, N, D> {
    type Request = List<(), 4>;
    type Response = ();
    const ID: u64 = 99;
    const NAME: &'static str = "budget";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: B,
        allocation: A,
        nodes: N,
        depth: D,
    };
    const RESPONSE_LIMITS: Limits = LIMITS;
}
#[test]
fn all_schema_budgets_precede_decode() {
    // Even empty input refuses the schema before touching a deserializer.
    assert_eq!(
        decode_request::<Budget<20, 100, 8, 4>>(&[]),
        Err(Error::SchemaLimit)
    );
    assert_eq!(
        decode_request::<Budget<100, 20, 8, 4>>(&[]),
        Err(Error::SchemaLimit)
    );
    assert_eq!(
        decode_request::<Budget<100, 100, 4, 4>>(&[]),
        Err(Error::SchemaLimit)
    );
    assert_eq!(
        decode_request::<Budget<100, 100, 8, 1>>(&[]),
        Err(Error::SchemaLimit)
    );
    let request = List::new(vec![(); 4]).unwrap();
    let mut scratch = vec![42; 1024];
    let capacity = scratch.capacity();
    assert_eq!(
        ic_wire::encode_request_into::<Budget<20, 100, 8, 4>>(&request, &mut scratch),
        Err(Error::SchemaLimit)
    );
    assert!(scratch.is_empty());
    assert_eq!(scratch.capacity(), capacity);
    assert_eq!(
        ic_wire::encoded_request_len::<Budget<20, 100, 8, 4>>(&request),
        Err(Error::SchemaLimit)
    );
    let bytes = encode_request::<Budget<100, 100, 8, 4>>(&request).unwrap();
    assert_eq!(
        decode_request::<Budget<100, 100, 8, 4>>(bytes.as_bytes()),
        Ok(request)
    );
    let mut too_many = bytes.into_bytes();
    *too_many.last_mut().unwrap() = 5;
    assert_eq!(
        decode_request::<Budget<100, 100, 8, 4>>(&too_many),
        Err(Error::Malformed)
    );
}
#[test]
fn list_count_rejects_before_element_decoder() {
    struct Never;
    impl<'de> serde::Deserialize<'de> for Never {
        fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            panic!("element decoded before limit check")
        }
    }
    assert!(postcard::from_bytes::<List<Never, 2>>(&[3, 0, 0, 0]).is_err());
}

#[test]
fn supported_scalar_extremes_and_fixed_arrays_fit_declared_shapes() {
    fn check<T: Wire + serde::de::DeserializeOwned + core::fmt::Debug + PartialEq>(value: T) {
        let bytes = postcard::to_allocvec(&value).unwrap();
        assert!(bytes.len() <= T::SHAPE.bytes);
        assert_eq!(postcard::from_bytes::<T>(&bytes).unwrap(), value);
    }
    check(());
    check(true);
    check(u8::MAX);
    check(i8::MIN);
    check(u16::MAX);
    check(i16::MIN);
    check(u32::MAX);
    check(i32::MIN);
    check(u64::MAX);
    check(i64::MIN);
    check(u128::MAX);
    check(i128::MIN);
    check([255_u8; 32]);
    check(List::<(), 4>::new(vec![]).unwrap());
    check(Text::<0>::new(String::new()).unwrap());
    check(Bytes::<0>::new(vec![]).unwrap());
}

#[test]
fn generated_names_and_explicit_variant_order_are_stable() {
    #[derive(Debug, PartialEq, Wire)]
    enum Visitor {
        #[wire(id = 40)]
        Denied,
        #[wire(id = 9)]
        Detail(Text<32>),
    }
    #[derive(Debug, PartialEq, Wire)]
    enum Reversed {
        #[wire(id = 9)]
        Detail(Text<32>),
        #[wire(id = 40)]
        Denied,
    }
    #[derive(Debug, PartialEq, Wire)]
    struct Result {}
    assert_eq!(
        postcard::to_allocvec(&Visitor::Denied).unwrap(),
        postcard::to_allocvec(&Reversed::Denied).unwrap()
    );
    assert_eq!(
        postcard::from_bytes::<Visitor>(&[40]).unwrap(),
        Visitor::Denied
    );
    assert_eq!(postcard::from_bytes::<Result>(&[]).unwrap(), Result {});
}

#[test]
fn saturated_node_count_is_never_admitted() {
    struct Saturated;
    impl Method for Saturated {
        type Request = List<(), { usize::MAX }>;
        type Response = ();
        const ID: u64 = 1;
        const NAME: &'static str = "saturated";
        const REQUEST_LIMITS: Limits = Limits {
            bytes: usize::MAX,
            allocation: usize::MAX,
            nodes: usize::MAX,
            depth: usize::MAX,
        };
        const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
    }
    let value = List::new(vec![]).unwrap();
    assert!(matches!(
        encode_request::<Saturated>(&value),
        Err(Error::SchemaLimit)
    ));
    assert!(matches!(
        decode_request::<Saturated>(&[]),
        Err(Error::SchemaLimit)
    ));
    #[cfg(feature = "typescript")]
    assert_eq!(
        ic_wire::typescript::generate::<Saturated>(),
        Err(ic_wire::typescript::GenerationError::Limits)
    );
}

#[cfg(feature = "typescript")]
#[test]
fn typescript_projects_explicit_id_order_and_rejects_opaque_metadata() {
    use ic_wire::{
        schema::Schema,
        typescript::{GenerationError, generate},
    };
    let Schema::Record(fields) = Grant::SCHEMA else {
        panic!("record metadata");
    };
    assert_eq!(
        fields.iter().map(|f| (f.id, f.name)).collect::<Vec<_>>(),
        [(10, "subject"), (20, "label")]
    );
    let generated = generate::<Read>().unwrap();
    assert!(generated.contains("readonly some: bigint"));
    assert!(generated.contains("readonly tag: \"Detail\""));
    assert!(generated.contains("export const methodName = \"read\""));
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Manual;
    impl Wire for Manual {
        const SHAPE: ic_wire::Shape = ic_wire::Shape::scalar(0);
    }
    struct Opaque;
    impl Method for Opaque {
        type Request = Manual;
        type Response = ();
        const ID: u64 = 1;
        const NAME: &'static str = "opaque";
        const REQUEST_LIMITS: Limits = LIMITS;
        const RESPONSE_LIMITS: Limits = LIMITS;
    }
    assert_eq!(generate::<Opaque>(), Err(GenerationError::Schema));
}

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
    for name in ["bad-name", "default", "string", "createCodec", "JSON"] {
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

#[test]
fn finite_float_bits_and_nonfinite_rejection() {
    use ic_wire::{FiniteF32, FiniteF64};
    struct FloatMethod;
    impl Method for FloatMethod {
        type Request = FiniteF32;
        type Response = FiniteF64;
        const ID: u64 = 0xf10a7;
        const NAME: &'static str = "float";
        const REQUEST_LIMITS: Limits = LIMITS;
        const RESPONSE_LIMITS: Limits = LIMITS;
    }
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
        let bytes = encode_request::<FloatMethod>(&FiniteF32::new(value).unwrap()).unwrap();
        assert_eq!(&bytes.as_bytes()[11..], &value.to_le_bytes());
        assert_eq!(
            decode_request::<FloatMethod>(bytes.as_bytes())
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
        let bytes = encode_response::<FloatMethod>(&FiniteF64::new(value).unwrap()).unwrap();
        assert_eq!(&bytes.as_bytes()[11..], &value.to_le_bytes());
        assert_eq!(
            decode_response::<FloatMethod>(bytes.as_bytes())
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
        let mut bytes = encode_request::<FloatMethod>(&FiniteF32::new(0.0).unwrap())
            .unwrap()
            .into_bytes();
        bytes[11..].copy_from_slice(&bits.to_le_bytes());
        assert_eq!(decode_request::<FloatMethod>(&bytes), Err(Error::Malformed));
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
        let mut bytes = encode_response::<FloatMethod>(&FiniteF64::new(0.0).unwrap())
            .unwrap()
            .into_bytes();
        bytes[11..].copy_from_slice(&bits.to_le_bytes());
        assert_eq!(
            decode_response::<FloatMethod>(&bytes),
            Err(Error::Malformed)
        );
    }
    assert_eq!(FiniteF32::SHAPE.bytes, 4);
    assert_eq!(FiniteF64::SHAPE.bytes, 8);
    assert_eq!(FiniteF64::SHAPE.heap, 0);
}

// Manual Wire implementations must not bypass the declared byte bound, and
// serializer failures must remain distinct from noncanonical decoded values.
#[derive(Debug, PartialEq, serde::Deserialize)]
struct Adversarial(u8);
impl serde::Serialize for Adversarial {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            1 => serializer.serialize_u64(u64::MAX),
            2 => Err(serde::ser::Error::custom("intentional serializer failure")),
            3 => serializer.serialize_u8(4),
            4 => serializer.serialize_unit(),
            5 => serializer.serialize_bytes(&[0; 8]),
            6 => serializer.serialize_u16(128),
            _ => serializer.serialize_u8(self.0),
        }
    }
}
impl Wire for Adversarial {
    const SHAPE: ic_wire::Shape = <u16 as Wire>::SHAPE;
}
struct AdversarialMethod;
impl Method for AdversarialMethod {
    type Request = Adversarial;
    type Response = ();
    const ID: u64 = 99;
    const NAME: &'static str = "adversarial";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}
#[test]
fn custom_serializer_bounds_and_error_identity() {
    let canonical = encode_request::<AdversarialMethod>(&Adversarial(0)).unwrap();
    for tag in [1, 2, 5] {
        let mut scratch = vec![42; 1024];
        assert_eq!(
            ic_wire::encode_request_into::<AdversarialMethod>(&Adversarial(tag), &mut scratch),
            Err(Error::Malformed)
        );
        assert!(scratch.is_empty());
        assert_eq!(
            ic_wire::encoded_request_len::<AdversarialMethod>(&Adversarial(tag)),
            Err(Error::Malformed)
        );
        assert_eq!(
            encode_request::<AdversarialMethod>(&Adversarial(tag))
                .err()
                .unwrap(),
            Error::Malformed
        );
        let mut bytes = canonical.as_bytes().to_vec();
        *bytes.last_mut().unwrap() = tag;
        assert_eq!(
            decode_request::<AdversarialMethod>(&bytes),
            Err(Error::Malformed)
        );
    }
    for tag in [3, 4, 6] {
        let mut bytes = canonical.as_bytes().to_vec();
        *bytes.last_mut().unwrap() = tag;
        assert_eq!(
            decode_request::<AdversarialMethod>(&bytes),
            Err(Error::Noncanonical)
        );
    }
}
mod large_bytes;

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
