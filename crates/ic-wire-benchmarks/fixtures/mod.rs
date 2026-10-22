//! Matched values; setup and semantic roundtrip assertions are outside measurement.
#![allow(dead_code)]
#[cfg(feature = "candid")]
use ic_platform::candid::contract::CandidType;
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt::Debug, hint::black_box, rc::Rc};

pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 2 * 1024 * 1024,
    allocation: 256 * 1024 * 1024,
    nodes: 8 * 1024 * 1024,
    depth: 64,
};
pub const METHOD: u64 = 0x6265_6e63_6800_0001;
#[cfg(feature = "candid")]
pub const CANDID_LIMITS: ic_platform::candid::DecodeLimits = ic_platform::candid::DecodeLimits {
    reply_bytes: 2 * 1024 * 1024,
    decoding_quota: 128 * 1024 * 1024,
    skipping_quota: 128 * 1024 * 1024,
    type_entries: 4096,
    header_bytes: 64 * 1024,
};

type Decode = Box<dyn Fn(&[u8]) -> bool>;
type Reuse = Box<dyn Fn(&mut Vec<u8>)>;
pub struct Lane {
    pub name: &'static str,
    pub bytes: Vec<u8>,
    pub encode: Box<dyn Fn() -> Vec<u8>>,
    pub decode: Decode,
    pub reuse: Option<Reuse>,
}
pub struct Case {
    pub name: String,
    pub input_json: Vec<u8>,
    pub lanes: Vec<Lane>,
}

fn lane<T: PartialEq + Debug + 'static>(
    name: &'static str,
    value: Rc<T>,
    encode: impl Fn(&T) -> Vec<u8> + 'static,
    decode: impl Fn(&[u8]) -> Option<T> + 'static,
) -> Lane {
    let bytes = encode(&value);
    assert_eq!(decode(&bytes).as_ref(), Some(value.as_ref()), "{name}");
    Lane {
        name,
        bytes,
        encode: Box::new(move || encode(black_box(&value))),
        decode: Box::new(move |bytes| {
            let result = decode(black_box(bytes));
            let ok = result.is_some();
            black_box(result);
            ok
        }),
        reuse: None,
    }
}

// The optional bound keeps Candid out of the wire-only linked artifact.
#[cfg(feature = "candid")]
pub trait Compared: CandidType {}
#[cfg(feature = "candid")]
impl<T: CandidType> Compared for T {}
#[cfg(not(feature = "candid"))]
pub trait Compared {}
#[cfg(not(feature = "candid"))]
impl<T> Compared for T {}

fn case<T: Serialize + DeserializeOwned + Compared + PartialEq + Debug + 'static>(
    name: &str,
    value: T,
) -> Case {
    let input_json = serde_json::to_vec(&value).unwrap();
    let value = Rc::new(value);
    let mut lanes = Vec::new();
    #[cfg(feature = "wire")]
    {
        let mut wire = lane(
            "wire-native",
            value.clone(),
            |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
            |bytes| ic_wire::native::decode_request(METHOD, bytes, LIMITS).ok(),
        );
        let v = value.clone();
        wire.reuse = Some(Box::new(move |out| {
            ic_wire::native::encode_request_into(METHOD, black_box(&*v), LIMITS, out).unwrap();
        }));
        lanes.push(wire);
    }
    #[cfg(feature = "candid")]
    lanes.push(lane(
        "candid",
        value,
        |v| ic_platform::candid::contract::encode_one(v).unwrap(),
        |bytes| ic_platform::candid::decode_exactly_one(bytes, CANDID_LIMITS).ok(),
    ));
    Case {
        name: name.into(),
        input_json,
        lanes,
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[serde(transparent)]
struct Blob(#[serde(with = "serde_bytes")] Vec<u8>);

// Use Candid's bulk blob primitive, matching Serde's byte-oriented path.
// Keep the byte contract explicit; Candid can also optimize generic vectors.
#[cfg(feature = "candid")]
impl CandidType for Blob {
    fn _ty() -> ic_platform::candid::contract::types::Type {
        <Vec<u8> as CandidType>::ty()
    }
    fn idl_serialize<S: ic_platform::candid::contract::types::Serializer>(
        &self,
        serializer: S,
    ) -> Result<(), S::Error> {
        serializer.serialize_blob(&self.0)
    }
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct Scalars {
    flag: bool,
    small: u8,
    signed: i64,
    unsigned: u64,
    wide: u128,
    single: f32,
    double: f64,
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct Message {
    sequence: u64,
    topic: String,
    payload: Blob,
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
enum Event {
    Idle,
    Data(Message),
    Failure { code: u32, detail: String },
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct Manifest {
    object: u128,
    size: u64,
    content_type: String,
    hashes: Vec<[u8; 32]>,
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct PackedManifest {
    object: u128,
    size: u64,
    content_type: String,
    hashes: Blob,
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct Node {
    value: u64,
    children: Vec<Node>,
}
fn tree(depth: usize) -> Node {
    Node {
        value: depth as u64,
        children: if depth == 0 {
            vec![]
        } else {
            vec![tree(depth - 1), tree(depth - 1)]
        },
    }
}
fn data(n: usize) -> Vec<u8> {
    let mut state = 0x1234_5678_9abc_def0_u64;
    (0..n)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}
fn message(n: usize) -> Message {
    Message {
        sequence: 16_384,
        topic: "relay/events".into(),
        payload: Blob(data(n)),
    }
}

/// Deterministic corpus, including tiny controls and representation boundaries.
pub fn cases() -> Vec<Case> {
    cases_selected(None)
}
/// Construct only the selected workload for IC calls, preserving corpus indices.
pub fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let mut out = Vec::new();
    let mut index = 0;
    macro_rules! push {
        ($value:expr) => {{
            if selected.is_none_or(|s| s == index) {
                out.push($value);
            }
            index += 1;
        }};
    }
    push!(case("unit", ()));
    push!(case("bool", true));
    macro_rules! integers {
        ($($ty:ty),*) => {$({
            push!(case(concat!(stringify!($ty), "-zero"), 0 as $ty));
            push!(case(concat!(stringify!($ty), "-max"), <$ty>::MAX));
        })*};
    }
    integers!(u8, u16, u32, i8, i16, i32);
    push!(case("f32-subnormal", f32::from_bits(1)));
    push!(case("f64-negative-zero", -0.0_f64));
    push!(case("f64-max", f64::MAX));
    push!(case(
        "principal-anonymous",
        ic_platform::candid::contract::Principal::anonymous(),
    ));
    push!(case(
        "principal-29-bytes",
        ic_platform::candid::contract::Principal::from_slice(&[7; 29]),
    ));
    for n in [0, 1, 127, 128, 16_383, 16_384, u32::MAX as u64, u64::MAX] {
        push!(case(&format!("u64-{n}"), n));
    }
    for n in [i64::MIN, -65, -64, -1, 0, 63, 64, i64::MAX] {
        push!(case(&format!("i64-{n}"), n));
    }
    push!(case("u128-max", u128::MAX));
    push!(case("i128-min", i128::MIN));
    push!(case(
        "scalar-record",
        Scalars {
            flag: true,
            small: 255,
            signed: -65,
            unsigned: 128,
            wide: u128::MAX,
            single: 1.5,
            double: -0.0,
        },
    ));
    push!(case("tuple", (true, 128_u64, "hello".to_owned())));
    push!(case("option-none", None::<u64>));
    push!(case("option-some", Some(128_u64)));
    push!(case("result-ok", Ok::<u64, String>(128)));
    push!(case(
        "result-error",
        Err::<u64, String>("permission denied".into()),
    ));
    push!(case("nested-option", Some(Some(128_u64))));
    push!(case("hash-array", [0x95_u8; 32]));
    for n in [4096, 1_048_576] {
        push!(case(&format!("generic-byte-vector-{n}"), data(n)));
    }
    for n in [0, 1, 32, 127, 128, 4096, 65_536, 1_048_576] {
        push!(case(&format!("blob-random-{n}"), Blob(data(n))));
        #[cfg(feature = "wire")]
        if selected.is_none_or(|s| s + 1 == index) {
            add_blob_lanes(out.last_mut().unwrap(), data(n));
        }
        push!(case(&format!("blob-zero-{n}"), Blob(vec![0; n])));
    }
    for n in [0, 1, 32, 127, 128, 4096, 65_536] {
        push!(case(&format!("text-ascii-{n}"), "a".repeat(n)));
        #[cfg(feature = "wire")]
        if selected.is_none_or(|s| s + 1 == index) {
            add_finite(
                out.last_mut().unwrap(),
                ic_wire::Text::<65536>::new("a".repeat(n)).unwrap(),
            );
        }
        push!(case(&format!("text-unicode-{n}"), "é水🌍".repeat(n / 9)));
    }
    for n in [0, 1, 16, 1024, 16_384] {
        push!(case(
            &format!("u64-list-small-{n}"),
            (0..n).map(|i| (i % 128) as u64).collect::<Vec<_>>(),
        ));
        push!(case(
            &format!("u64-list-wide-{n}"),
            (0..n).map(|i| u64::MAX - i as u64).collect::<Vec<_>>(),
        ));
    }
    for n in [0, 1, 16, 128] {
        push!(case(
            &format!("sorted-map-{n}"),
            (0..n)
                .map(|i| (format!("key-{i:04}"), i as u64))
                .collect::<std::collections::BTreeMap<_, _>>(),
        ));
        push!(case(
            &format!("text-pairs-{n}"),
            (0..n)
                .map(|i| (format!("key-{i}"), "value".repeat(4)))
                .collect::<Vec<_>>(),
        ));
        push!(case(
            &format!("relay-batch-{n}"),
            (0..n).map(|_| message(256)).collect::<Vec<_>>(),
        ));
        #[cfg(feature = "wire")]
        if selected.is_none_or(|s| s + 1 == index) {
            add_finite(
                out.last_mut().unwrap(),
                ic_wire::List::<_, 128>::new((0..n).map(|_| finite_message(256)).collect())
                    .unwrap(),
            );
        }
    }
    for n in [0, 1, 32, 1024] {
        push!(case(
            &format!("blob-manifest-{n}"),
            Manifest {
                object: u128::MAX,
                size: n as u64 * 1_048_576,
                content_type: "application/octet-stream".into(),
                hashes: (0..n).map(|i| [i as u8; 32]).collect(),
            },
        ));
        #[cfg(feature = "wire")]
        if selected.is_none_or(|s| s + 1 == index) {
            add_finite(
                out.last_mut().unwrap(),
                FiniteManifest {
                    object: u128::MAX,
                    size: n as u64 * 1_048_576,
                    content_type: ic_wire::Text::new("application/octet-stream".into()).unwrap(),
                    hashes: ic_wire::List::new((0..n).map(|i| [i as u8; 32]).collect()).unwrap(),
                },
            );
        }
    }
    for n in [0, 1, 32, 1024] {
        push!(case(
            &format!("packed-manifest-{n}"),
            PackedManifest {
                object: u128::MAX,
                size: n as u64 * 1_048_576,
                content_type: "application/octet-stream".into(),
                hashes: Blob((0..n).flat_map(|i| [i as u8; 32]).collect())
            }
        ));
    }
    push!(case("variant-unit", Event::Idle));
    #[cfg(feature = "wire")]
    if selected.is_none_or(|s| s + 1 == index) {
        add_finite(out.last_mut().unwrap(), FiniteEvent::Idle);
    }
    push!(case("variant-record", Event::Data(message(128))));
    #[cfg(feature = "wire")]
    if selected.is_none_or(|s| s + 1 == index) {
        add_finite(
            out.last_mut().unwrap(),
            FiniteEvent::Data(finite_message(128)),
        );
    }
    push!(case(
        "variant-error",
        Event::Failure {
            code: 403,
            detail: "permission denied".into(),
        },
    ));
    for depth in [0, 1, 4, 8] {
        push!(case(&format!("tree-depth-{depth}"), tree(depth)));
    }
    assert!(selected.is_none_or(|s| s < index), "unknown case");
    out
}

#[cfg(feature = "wire")]
struct Binding<T>(std::marker::PhantomData<T>);
#[cfg(feature = "wire")]
impl<T: ic_wire::Wire> ic_wire::Method for Binding<T> {
    type Request = T;
    type Response = T;
    const ID: u64 = METHOD;
    const NAME: &'static str = "benchmark";
    const REQUEST_LIMITS: ic_wire::Limits = LIMITS;
    const RESPONSE_LIMITS: ic_wire::Limits = LIMITS;
}
#[cfg(feature = "wire")]
fn add_finite<T: ic_wire::Wire + DeserializeOwned + PartialEq + Debug + 'static>(
    case: &mut Case,
    value: T,
) {
    let finite = lane(
        "wire-finite",
        Rc::new(value),
        |v| {
            ic_wire::encode_request::<Binding<T>>(v)
                .unwrap()
                .into_bytes()
        },
        |bytes| ic_wire::decode_request::<Binding<T>>(bytes).ok(),
    );
    assert_eq!(finite.bytes, case.lanes[0].bytes);
    case.lanes.push(finite);
}
#[cfg(feature = "wire")]
fn add_blob_lanes(case: &mut Case, value: Vec<u8>) {
    add_finite(
        case,
        ic_wire::Bytes::<1_048_576>::new(value.clone()).unwrap(),
    );
    let bytes = case.lanes[0].bytes.clone();
    let view: &serde_bytes::Bytes =
        ic_wire::native::decode_request(METHOD, &bytes, LIMITS).unwrap();
    assert_eq!(view.as_ref(), value);
    if !view.is_empty() {
        let address = view.as_ptr() as usize;
        assert!(
            (bytes.as_ptr() as usize..bytes.as_ptr() as usize + bytes.len()).contains(&address)
        );
    }
    case.lanes.push(Lane {
        name: "wire-borrowed",
        bytes,
        encode: Box::new(move || {
            ic_wire::native::encode_request(METHOD, serde_bytes::Bytes::new(&value), LIMITS)
                .unwrap()
        }),
        decode: Box::new(|bytes| {
            let result = ic_wire::native::decode_request::<&serde_bytes::Bytes>(
                METHOD,
                black_box(bytes),
                LIMITS,
            );
            let ok = result.is_ok();
            black_box(result).ok();
            ok
        }),
        reuse: None,
    });
}

#[cfg(feature = "wire")]
#[derive(Debug, PartialEq, ic_wire::Wire)]
struct FiniteMessage {
    #[wire(id = 1)]
    sequence: u64,
    #[wire(id = 2)]
    topic: ic_wire::Text<64>,
    #[wire(id = 3)]
    payload: ic_wire::Bytes<256>,
}
#[cfg(feature = "wire")]
#[derive(Debug, PartialEq, ic_wire::Wire)]
struct FiniteManifest {
    #[wire(id = 1)]
    object: u128,
    #[wire(id = 2)]
    size: u64,
    #[wire(id = 3)]
    content_type: ic_wire::Text<64>,
    #[wire(id = 4)]
    hashes: ic_wire::List<[u8; 32], 1024>,
}
#[cfg(feature = "wire")]
#[derive(Debug, PartialEq, ic_wire::Wire)]
enum FiniteEvent {
    #[wire(id = 0)]
    Idle,
    #[wire(id = 1)]
    Data(FiniteMessage),
}
#[cfg(feature = "wire")]
fn finite_message(n: usize) -> FiniteMessage {
    FiniteMessage {
        sequence: 16_384,
        topic: ic_wire::Text::new("relay/events".into()).unwrap(),
        payload: ic_wire::Bytes::new(data(n)).unwrap(),
    }
}
