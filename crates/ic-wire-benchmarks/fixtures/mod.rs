//! Matched values; setup and semantic roundtrip assertions are outside measurement.
#![allow(dead_code)]
#[cfg(feature = "packed-i16")]
mod packed_i16;
#[cfg(feature = "derive-scale")]
mod scale;
#[cfg(feature = "toko")]
mod toko;
#[cfg(all(feature = "toko", feature = "derive-scale"))]
compile_error!("select one benchmark corpus");
#[cfg(all(
    feature = "packed-i16",
    any(feature = "toko", feature = "derive-scale")
))]
compile_error!("select one benchmark corpus");
#[cfg(all(feature = "derive-scale-1", feature = "derive-scale-10"))]
compile_error!("select one derive cohort size");
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
    // Keep per-type diagnostic formatting out of the measured Wasm artifact.
    #[cfg(not(target_family = "wasm"))]
    assert_eq!(decode(&bytes).as_ref(), Some(value.as_ref()), "{name}");
    #[cfg(target_family = "wasm")]
    assert!(
        decode(&bytes).as_ref() == Some(value.as_ref()),
        "{name}: roundtrip mismatch"
    );
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
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct AccountSnapshot {
    id: u64,
    owner: String,
    balance: u128,
    delta: i64,
    tags: std::collections::BTreeMap<String, String>,
    events: Vec<Event>,
    samples: Vec<Option<i64>>,
    proofs: Vec<Blob>,
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
    #[cfg(feature = "toko")]
    {
        toko::cases_selected(selected)
    }
    #[cfg(feature = "derive-scale")]
    {
        scale::cases_selected(selected)
    }
    #[cfg(feature = "packed-i16")]
    {
        packed_i16::cases_selected(selected)
    }
    #[cfg(not(any(feature = "toko", feature = "derive-scale", feature = "packed-i16")))]
    {
        standard_cases_selected(selected)
    }
}
fn standard_cases_selected(selected: Option<usize>) -> Vec<Case> {
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
    macro_rules! integer_lists {
        ($($ty:ty),*) => {$({
            for n in [0, 1, 1024, 16384] {
                push!(case(&format!("{}-list-small-{n}", stringify!($ty)),
                    (0..n).map(|i| (i % 64) as $ty).collect::<Vec<_>>()));
                push!(case(&format!("{}-list-wide-{n}", stringify!($ty)),
                    vec![<$ty>::MAX; n]));
            }
        })*};
    }
    integer_lists!(u16, u32, i16, i32, i64);

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
        #[cfg(feature = "wire")]
        if selected.is_none_or(|s| s + 1 == index) {
            // Same input and bytes; compare bounded Serde fields under the same runtime admission.
            add_bounded(
                out.last_mut().unwrap(),
                ic_wire::List::<_, 1_048_576>::new(data(n)).unwrap(),
            );
        }
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
            add_bounded(
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
        #[cfg(feature = "wire")]
        if selected.is_none_or(|s| s + 1 == index) {
            // Same input and bytes; compare bounded Serde fields under the same runtime admission.
            add_bounded(
                out.last_mut().unwrap(),
                ic_wire::List::<_, 16_384>::new((0..n).map(|i| (i % 128) as u64).collect())
                    .unwrap(),
            );
        }
        push!(case(
            &format!("u64-list-wide-{n}"),
            (0..n).map(|i| u64::MAX - i as u64).collect::<Vec<_>>(),
        ));
        #[cfg(feature = "wire")]
        if selected.is_none_or(|s| s + 1 == index) {
            // Same input and bytes; compare bounded Serde fields under the same runtime admission.
            add_bounded(
                out.last_mut().unwrap(),
                ic_wire::List::<_, 16_384>::new((0..n).map(|i| u64::MAX - i as u64).collect())
                    .unwrap(),
            );
        }
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
            add_bounded(
                out.last_mut().unwrap(),
                ic_wire::List::<_, 128>::new((0..n).map(|_| bounded_message(256)).collect())
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
            add_bounded(
                out.last_mut().unwrap(),
                BoundedManifest {
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
        add_bounded(out.last_mut().unwrap(), BoundedEvent::Idle);
    }
    push!(case("variant-record", Event::Data(message(128))));
    #[cfg(feature = "wire")]
    if selected.is_none_or(|s| s + 1 == index) {
        add_bounded(
            out.last_mut().unwrap(),
            BoundedEvent::Data(bounded_message(128)),
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
    // Append new cases so existing benchmark identities retain their indices.
    macro_rules! signed_distributions {
        ($($ty:ty),*) => {$({
            for n in [1, 128, 4096] {
                push!(case(&format!("{}-list-negative-{n}", stringify!($ty)),
                    (0..n).map(|i| -1 - (i % 64) as $ty).collect::<Vec<_>>()));
                let values = [<$ty>::MIN, -16385, -65, -64, -1, 0, 63, 64, 16384, <$ty>::MAX];
                push!(case(&format!("{}-list-mixed-{n}", stringify!($ty)),
                    (0..n).map(|i| values[i % values.len()]).collect::<Vec<_>>()));
            }
        })*};
    }
    signed_distributions!(i16, i32, i64);
    macro_rules! unsigned_distributions {
        ($($ty:ty),*) => {$({
            for n in [1, 128, 4096] {
                let values = [0, 127, 128, 16383, 16384, <$ty>::MAX];
                push!(case(&format!("{}-list-mixed-{n}", stringify!($ty)),
                    (0..n).map(|i| values[i % values.len()]).collect::<Vec<_>>()));
            }
        })*};
    }
    unsigned_distributions!(u16, u32, u64);
    for n in [63, 64, 65, 129, 16383, 16384, 16385, 65535, 65537] {
        push!(case(&format!("blob-random-{n}"), Blob(data(n))));
    }
    for n in [0, 1, 32, 127, 128] {
        push!(case(&format!("generic-byte-vector-{n}"), data(n)));
    }
    for n in [0, 1, 1024] {
        push!(case(
            &format!("bool-list-{n}"),
            (0..n).map(|i| i % 2 == 0).collect::<Vec<_>>()
        ));
        push!(case(
            &format!("optional-i64-list-{n}"),
            (0..n)
                .map(|i| if i % 3 == 0 { None } else { Some(-(i as i64)) })
                .collect::<Vec<_>>()
        ));
        let singles = [0.0_f32, -0.0, f32::MIN_POSITIVE, f32::MAX, -1.5];
        let doubles = [0.0_f64, -0.0, f64::MIN_POSITIVE, f64::MAX, -1.5];
        push!(case(
            &format!("f32-list-{n}"),
            (0..n)
                .map(|i| singles[i % singles.len()])
                .collect::<Vec<_>>()
        ));
        push!(case(
            &format!("f64-list-{n}"),
            (0..n)
                .map(|i| doubles[i % doubles.len()])
                .collect::<Vec<_>>()
        ));
    }
    for n in [0, 1, 16, 128] {
        push!(case(
            &format!("nested-blobs-{n}"),
            (0..n)
                .map(|i| Blob(data([0, 1, 63, 64, 128, 4096][i % 6])))
                .collect::<Vec<_>>()
        ));
    }
    for n in [0, 1, 128] {
        push!(case(
            &format!("account-snapshot-{n}"),
            AccountSnapshot {
                id: 42,
                owner: "account:benchmark".into(),
                balance: 1_000_000_000_000,
                delta: -65,
                tags: [
                    ("tier".into(), "standard".into()),
                    ("region".into(), "eu".into())
                ]
                .into(),
                events: (0..n)
                    .map(|i| match i % 3 {
                        0 => Event::Idle,
                        1 => Event::Data(message(128)),
                        _ => Event::Failure {
                            code: 403,
                            detail: "permission denied".into()
                        },
                    })
                    .collect(),
                samples: (0..n)
                    .map(|i| if i % 2 == 0 { None } else { Some(-(i as i64)) })
                    .collect(),
                proofs: (0..n).map(|i| Blob(vec![i as u8; 32])).collect(),
            }
        ));
    }
    assert!(selected.is_none_or(|s| s < index), "unknown case");
    out
}

#[cfg(feature = "wire")]
fn add_bounded<T: Serialize + DeserializeOwned + PartialEq + Debug + 'static>(
    case: &mut Case,
    value: T,
) {
    let bounded = lane(
        "wire-bounded",
        Rc::new(value),
        |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
        |bytes| ic_wire::native::decode_request::<T>(METHOD, bytes, LIMITS).ok(),
    );
    assert_eq!(bounded.bytes, case.lanes[0].bytes);
    case.lanes.push(bounded);
}
#[cfg(feature = "wire")]
fn add_blob_lanes(case: &mut Case, value: Vec<u8>) {
    add_bounded(
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
#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
struct BoundedMessage {
    sequence: u64,

    topic: ic_wire::Text<64>,

    payload: ic_wire::Bytes<256>,
}
#[cfg(feature = "wire")]
#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
struct BoundedManifest {
    object: u128,

    size: u64,

    content_type: ic_wire::Text<64>,

    hashes: ic_wire::List<[u8; 32], 1024>,
}
#[cfg(feature = "wire")]
#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
enum BoundedEvent {
    Idle,

    Data(BoundedMessage),
}
#[cfg(feature = "wire")]
fn bounded_message(n: usize) -> BoundedMessage {
    BoundedMessage {
        sequence: 16_384,
        topic: ic_wire::Text::new("relay/events".into()).unwrap(),
        payload: ic_wire::Bytes::new(data(n)).unwrap(),
    }
}
