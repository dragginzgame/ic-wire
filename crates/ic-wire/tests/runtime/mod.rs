//! Runtime contract behavior and rejection boundaries.
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
    fn check<T: Wire + core::fmt::Debug + PartialEq>(value: T) {
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
