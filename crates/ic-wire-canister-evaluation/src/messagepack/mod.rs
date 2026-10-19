//! Valid-input codec experiment. Raw lanes do not implement Wire admission.
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_bytes::ByteBuf;
use std::{fmt::Debug, hint::black_box};

/// Stable workload names, shared by native and external Wasm harnesses.
pub const CASES: &[&str] = &[
    "rpc-small",
    "rpc-wide",
    "u64-small-1024",
    "u64-random-1024",
    "u64-max-1024",
    "i64-small-1024",
    "u128-small-128",
    "u128-wide-128",
    "metadata-ascii",
    "metadata-utf8",
    "manifest-array-1",
    "manifest-array-32",
    "manifest-array-1024",
    "manifest-binary-1",
    "manifest-binary-32",
    "manifest-binary-1024",
    "bytes-0",
    "bytes-31",
    "bytes-32",
    "bytes-255",
    "bytes-256",
    "bytes-4096",
    "bytes-65536",
    "bytes-1048576",
    "vec-u8-4096",
    "result-ok",
    "result-error",
];
/// One measured batch. Clock units are selected by the caller.
#[derive(Debug, Deserialize, Serialize)]
pub struct Sample {
    /// Exact encoded input, used for identity checks outside measured intervals.
    pub bytes: Vec<u8>,
    /// Encode elapsed units for all repetitions, including allocation and drop.
    pub encode: u64,
    /// Owned decode elapsed units for all repetitions, including drop.
    pub decode: u64,
}
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Rpc {
    id: u128,
    offset: u64,
    length: u32,
    committed: bool,
}
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Metadata {
    content_type: String,
    path: String,
    headers: Vec<(String, String)>,
}
#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Manifest<H> {
    service: ByteBuf,
    tenant: ByteBuf,
    uploader: ByteBuf,
    namespace: u128,
    upload: u128,
    object: u128,
    incarnation: u128,
    first_reference: u128,
    root: H,
    bytes: u64,
    expires_at_ns: u64,
    chunks: Vec<H>,
    headers: Vec<(String, String)>,
}
fn random(index: usize) -> u64 {
    let mut x = (index as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}
fn hash(index: usize) -> [u8; 32] {
    let mut out = [0; 32];
    for (i, word) in out.as_chunks_mut::<8>().0.iter_mut().enumerate() {
        word.copy_from_slice(&random(index * 4 + i).to_le_bytes());
    }
    out
}
fn manifest<H>(n: usize, convert: impl Fn([u8; 32]) -> H) -> Manifest<H> {
    Manifest {
        service: ByteBuf::from(vec![1, 1]),
        tenant: ByteBuf::from(vec![2, 1]),
        uploader: ByteBuf::from(vec![3, 1]),
        namespace: u128::MAX - 1,
        upload: u128::MAX - 2,
        object: u128::MAX - 3,
        incarnation: u128::MAX - 4,
        first_reference: u128::MAX - 5,
        root: convert(hash(2000)),
        bytes: n as u64 * 1024 * 1024,
        expires_at_ns: u64::MAX - 1,
        chunks: (0..n).map(|i| convert(hash(i))).collect(),
        headers: vec![
            ("Content-Length".into(), (n * 1024 * 1024).to_string()),
            ("Content-Type".into(), "application/octet-stream".into()),
        ],
    }
}
fn encode<T: Serialize>(lane: u8, value: &T) -> Vec<u8> {
    match lane {
        #[cfg(feature = "comparison-postcard")]
        0 => postcard::to_allocvec(value).unwrap(),
        #[cfg(feature = "comparison-array")]
        1 => rmp_serde::to_vec(value).unwrap(),
        #[cfg(feature = "comparison-map")]
        2 => rmp_serde::to_vec_named(value).unwrap(),
        #[cfg(feature = "comparison-wire")]
        3 => ic_wire::native::encode_request(0x6265_6e63_686d_6172, value, limits()).unwrap(),
        _ => panic!("codec feature not selected: {lane}"),
    }
}
#[cfg(feature = "comparison-wire")]
fn limits() -> ic_wire::Limits {
    ic_wire::Limits {
        bytes: 2 * 1024 * 1024,
        allocation: 64 * 1024 * 1024,
        nodes: 262_144,
        depth: 32,
    }
}
fn decode<T: Serialize + DeserializeOwned>(lane: u8, bytes: &[u8]) -> T {
    match lane {
        #[cfg(feature = "comparison-postcard")]
        0 => {
            let (value, rest) = postcard::take_from_bytes(bytes).unwrap();
            assert!(rest.is_empty());
            value
        }
        #[cfg(any(feature = "comparison-array", feature = "comparison-map"))]
        1 | 2 => rmp_serde::from_slice(bytes).unwrap(),
        #[cfg(feature = "comparison-wire")]
        3 => ic_wire::native::decode_request(0x6265_6e63_686d_6172, bytes, limits()).unwrap(),
        _ => panic!("codec feature not selected: {lane}"),
    }
}
fn measure<T: Serialize + DeserializeOwned + Debug + PartialEq>(
    lane: u8,
    value: &T,
    repetitions: u64,
    clock: &mut impl FnMut() -> u64,
) -> Sample {
    let bytes = encode(lane, value);
    assert_eq!(&decode::<T>(lane, &bytes), value);
    let start = clock();
    for _ in 0..repetitions {
        black_box(encode(lane, black_box(value)));
    }
    let encode = clock() - start;
    let start = clock();
    for _ in 0..repetitions {
        black_box(decode::<T>(lane, black_box(&bytes)));
    }
    let decode = clock() - start;
    Sample {
        bytes,
        encode,
        decode,
    }
}
/// Construct a deterministic workload and measure only codec operations.
///
/// # Panics
/// Panics on an unknown case/lane, missing feature, or failed exact roundtrip.
pub fn run(lane: u8, case: usize, repetitions: u64, clock: &mut impl FnMut() -> u64) -> Sample {
    match case {
        0 | 1 => measure(
            lane,
            &Rpc {
                id: if case == 0 { 42 } else { u128::MAX - 42 },
                offset: if case == 0 { 64 } else { u64::MAX - 64 },
                length: 1024,
                committed: true,
            },
            repetitions,
            clock,
        ),
        2..=4 => measure(
            lane,
            &(0..1024)
                .map(|i| match case {
                    2 => i as u64 % 128,
                    3 => random(i),
                    _ => u64::MAX,
                })
                .collect::<Vec<_>>(),
            repetitions,
            clock,
        ),
        5 => measure(
            lane,
            &(0..1024).map(|i| i as i64 % 127 - 63).collect::<Vec<_>>(),
            repetitions,
            clock,
        ),
        6 | 7 => measure(
            lane,
            &(0..128)
                .map(|i| {
                    if case == 6 {
                        i as u128
                    } else {
                        (u128::from(random(i)) << 64) | u128::from(random(i + 128))
                    }
                })
                .collect::<Vec<_>>(),
            repetitions,
            clock,
        ),
        8 | 9 => measure(
            lane,
            &Metadata {
                content_type: "application/octet-stream".into(),
                path: if case == 8 {
                    "/assets/sample.bin".repeat(8)
                } else {
                    "/写真/échantillon/🚀".repeat(8)
                },
                headers: (0..16)
                    .map(|i| (format!("header-{i}"), format!("value-{i}")))
                    .collect(),
            },
            repetitions,
            clock,
        ),
        10..=12 => measure(
            lane,
            &manifest([1, 32, 1024][case - 10], |h| h),
            repetitions,
            clock,
        ),
        13..=15 => measure(
            lane,
            &manifest([1, 32, 1024][case - 13], |h| ByteBuf::from(h.to_vec())),
            repetitions,
            clock,
        ),
        16..=23 => {
            let n = [0, 31, 32, 255, 256, 4096, 65536, 1048576][case - 16];
            measure(
                lane,
                &ByteBuf::from((0..n).map(|i| (i % 251) as u8).collect::<Vec<_>>()),
                repetitions,
                clock,
            )
        }
        24 => measure(
            lane,
            &(0..4096).map(|i| (i % 251) as u8).collect::<Vec<_>>(),
            repetitions,
            clock,
        ),
        25 | 26 => {
            let value: Result<Rpc, String> = if case == 25 {
                Ok(Rpc {
                    id: 42,
                    offset: 64,
                    length: 1024,
                    committed: true,
                })
            } else {
                Err("upload expired".into())
            };
            measure(lane, &value, repetitions, clock)
        }
        _ => panic!("unknown case {case}"),
    }
}
#[cfg(all(
    test,
    feature = "comparison-postcard",
    feature = "comparison-messagepack",
    feature = "comparison-wire"
))]
mod tests {
    #[test]
    fn messagepack_representation_and_acceptance() {
        let encoded = rmp_serde::to_vec(&1_u128).unwrap();
        assert_eq!(&encoded[..2], &[0xc4, 16]);
        assert_eq!(&encoded[2..], &1_u128.to_be_bytes());
        // Equivalent integers and trailing data are accepted by the raw decoder.
        assert_eq!(rmp_serde::from_slice::<u64>(&[1]).unwrap(), 1);
        assert_eq!(rmp_serde::from_slice::<u64>(&[0xcc, 1]).unwrap(), 1);
        assert_eq!(rmp_serde::from_slice::<u64>(&[1, 2]).unwrap(), 1);
        assert_eq!(rmp_serde::to_vec(&[255_u8; 32]).unwrap().len(), 67);
        assert_eq!(
            rmp_serde::to_vec(&serde_bytes::ByteBuf::from(vec![255; 32]))
                .unwrap()
                .len(),
            34
        );
    }

    #[test]
    fn exact_roundtrips() {
        for case in 0..super::CASES.len() {
            let raw = super::run(0, case, 1, &mut || 0);
            for lane in 1..=3 {
                let sample = super::run(lane, case, 1, &mut || 0);
                if lane == 3 {
                    assert_eq!(&sample.bytes[11..], raw.bytes);
                }
            }
        }
    }
}
