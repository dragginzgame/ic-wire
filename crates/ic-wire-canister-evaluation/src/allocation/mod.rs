//! Matched bounded Postcard workloads; layouts have distinct experiment identities.
use ic_wire::{Bytes, Limits, List, Method, Text, Wire, native};
use serde::{Deserialize, Serialize};
use std::{hint::black_box, marker::PhantomData};

const LIMITS: Limits = Limits {
    bytes: 2 * 1024 * 1024,
    allocation: 64 * 1024 * 1024,
    nodes: 262_144,
    depth: 32,
};
const ARRAY: u64 = 0x6172_7261_7900_0001;
const PACKED: u64 = 0x7061_636b_6564_0001;
const BLOB: u64 = 0x626c_6f62_0000_0001;
/// Number of independently measured workload/lane combinations.
pub const CASE_COUNT: usize = 44;
/// One timed batch; units belong to the caller's clock.
#[derive(Debug, Deserialize, Serialize)]
pub struct Sample {
    /// Exact frame, returned outside the timed interval for identity checks.
    pub bytes: Vec<u8>,
    /// Encode time, including fresh allocation/drop or reuse as labelled.
    pub encode: u64,
    /// Decode and full hash/blob byte checksum time, including owned-value drop.
    pub decode: u64,
    /// Observed output capacity after one successful call, not peak heap usage.
    pub capacity: usize,
}
#[derive(Debug, PartialEq, Wire)]
struct Header {
    #[wire(id = 1)]
    upload: u128,
    #[wire(id = 2)]
    object: u128,
    #[wire(id = 3)]
    root: [u8; 32],
    #[wire(id = 4)]
    bytes: u64,
    #[wire(id = 5)]
    content_type: Text<64>,
}
#[derive(Debug, PartialEq, Wire)]
struct ArrayManifest {
    #[wire(id = 1)]
    header: Header,
    #[wire(id = 2)]
    chunks: List<[u8; 32], 1024>,
}
#[derive(Debug, PartialEq, Wire)]
struct PackedManifest {
    #[wire(id = 1)]
    header: Header,
    #[wire(id = 2)]
    chunks: Bytes<32768>,
}
#[derive(Deserialize, Serialize)]
struct OwnedHeader {
    upload: u128,
    object: u128,
    root: [u8; 32],
    bytes: u64,
    content_type: String,
}
#[derive(Deserialize, Serialize)]
struct OwnedArray {
    header: OwnedHeader,
    chunks: Vec<[u8; 32]>,
}
#[derive(Deserialize, Serialize)]
struct OwnedPacked {
    header: OwnedHeader,
    chunks: serde_bytes::ByteBuf,
}
fn owned_header(n: usize) -> OwnedHeader {
    OwnedHeader {
        upload: u128::MAX - 1,
        object: u128::MAX - 2,
        root: [0x95; 32],
        bytes: n as u64 * 1024 * 1024,
        content_type: "application/octet-stream".into(),
    }
}
#[derive(Deserialize, Serialize)]
struct HeaderView<'a> {
    upload: u128,
    object: u128,
    root: [u8; 32],
    bytes: u64,
    content_type: &'a str,
}
#[derive(Deserialize, Serialize)]
struct PackedView<'a> {
    #[serde(borrow)]
    header: HeaderView<'a>,
    #[serde(borrow)]
    chunks: &'a serde_bytes::Bytes,
}
struct Binding<T, const ID: u64>(PhantomData<T>);
impl<T: Wire, const ID: u64> Method for Binding<T, ID> {
    type Request = T;
    type Response = T;
    const ID: u64 = ID;
    const NAME: &'static str = "allocation_experiment";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}
fn header(n: usize) -> Header {
    Header {
        upload: u128::MAX - 1,
        object: u128::MAX - 2,
        root: [0x95; 32],
        bytes: n as u64 * 1024 * 1024,
        content_type: Text::new("application/octet-stream".into()).unwrap(),
    }
}
fn hash(i: usize) -> [u8; 32] {
    let mut x = (i as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut out = [0; 32];
    for byte in &mut out {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *byte = x as u8;
    }
    out
}
fn checksum(bytes: impl Iterator<Item = u8>) -> u64 {
    bytes.fold(0, |sum, byte| sum + u64::from(byte))
}
fn packed_checksum(bytes: &[u8]) -> Result<u64, ic_wire::Error> {
    // Consumer-owned layout validation; never silently accept a partial hash.
    if bytes.len() > 32768 {
        return Err(ic_wire::Error::Limit);
    }
    if !bytes.len().is_multiple_of(32) {
        return Err(ic_wire::Error::Malformed);
    }
    Ok(checksum(bytes.iter().copied()))
}
fn measure<T: Serialize, const ID: u64>(
    value: &T,
    mode: u8,
    repetitions: u64,
    clock: &mut impl FnMut() -> u64,
    mut decode: impl FnMut(&[u8]) -> u64,
    fresh: impl Fn() -> Vec<u8>,
) -> Sample {
    let bytes = fresh();
    let capacity = bytes.capacity();
    let expected = decode(&bytes);
    let mut reuse = bytes.clone();
    native::encode_request_into(ID, value, LIMITS, &mut reuse).unwrap();
    assert_eq!(bytes, reuse);
    let start = clock();
    for _ in 0..repetitions {
        if mode == 2 {
            native::encode_request_into(ID, black_box(value), LIMITS, &mut reuse).unwrap();
            black_box(&reuse);
        } else {
            black_box(fresh());
        }
    }
    let encode = clock() - start;
    let start = clock();
    for _ in 0..repetitions {
        black_box(decode(black_box(&bytes)));
    }
    let decode_time = clock() - start;
    assert_eq!(decode(&bytes), expected);
    Sample {
        bytes,
        encode,
        decode: decode_time,
        capacity: if mode == 2 {
            reuse.capacity()
        } else {
            capacity
        },
    }
}
/// Stable labels for the forty-four workload/lane combinations.
///
/// # Panics
/// Panics for an unknown case.
pub fn name(case: usize) -> String {
    if case < 28 {
        format!(
            "manifest-{}-{}",
            [0, 1, 32, 1024][case / 7],
            [
                "native-array",
                "finite-array",
                "reuse-array",
                "native-packed",
                "finite-packed",
                "reuse-packed",
                "borrow-packed"
            ][case % 7]
        )
    } else {
        let c = case - 28;
        format!(
            "blob-{}-{}",
            [0, 32, 4096, 1048576][c / 4],
            ["native-owned", "finite-owned", "reuse-owned", "borrow"][c % 4]
        )
    }
}
/// Measure encoding and decoding plus a complete byte scan; setup stays outside the clock.
///
/// # Panics
/// Panics on an invalid case, failed admission or inconsistent roundtrip.
pub fn run(case: usize, repetitions: u64, clock: &mut impl FnMut() -> u64) -> Sample {
    assert!(case < CASE_COUNT);
    if case < 28 {
        let n = [0, 1, 32, 1024][case / 7];
        let lane = case % 7;
        if lane < 3 {
            let finite = ArrayManifest {
                header: header(n),
                chunks: List::new((0..n).map(hash).collect()).unwrap(),
            };
            let owned = OwnedArray {
                header: owned_header(n),
                chunks: (0..n).map(hash).collect(),
            };
            let sample = measure::<_, ARRAY>(
                &owned,
                lane as u8,
                repetitions,
                clock,
                |bytes| {
                    if lane == 1 {
                        let v = ic_wire::decode_request::<Binding<ArrayManifest, ARRAY>>(bytes)
                            .unwrap();
                        checksum(v.chunks.as_slice().iter().flatten().copied())
                    } else {
                        let v: OwnedArray = native::decode_request(ARRAY, bytes, LIMITS).unwrap();
                        assert!(v.chunks.len() <= 1024);
                        checksum(v.chunks.iter().flatten().copied())
                    }
                },
                || {
                    if lane == 1 {
                        ic_wire::encode_request::<Binding<ArrayManifest, ARRAY>>(black_box(&finite))
                            .unwrap()
                            .into_bytes()
                    } else {
                        native::encode_request(ARRAY, black_box(&owned), LIMITS).unwrap()
                    }
                },
            );
            assert_eq!(
                native::decode_request::<ArrayManifest>(ARRAY, &sample.bytes, LIMITS).unwrap(),
                finite
            );
            sample
        } else {
            let finite = PackedManifest {
                header: header(n),
                chunks: Bytes::new((0..n).flat_map(hash).collect()).unwrap(),
            };
            let owned = OwnedPacked {
                header: owned_header(n),
                chunks: serde_bytes::ByteBuf::from((0..n).flat_map(hash).collect::<Vec<_>>()),
            };
            let mode = if lane == 4 {
                1
            } else if lane == 5 {
                2
            } else {
                0
            };
            let sample = measure::<_, PACKED>(
                &owned,
                mode,
                repetitions,
                clock,
                |bytes| {
                    if lane == 6 {
                        let v: PackedView<'_> =
                            native::decode_request(PACKED, bytes, LIMITS).unwrap();
                        packed_checksum(v.chunks).unwrap()
                    } else if lane == 4 {
                        let v = ic_wire::decode_request::<Binding<PackedManifest, PACKED>>(bytes)
                            .unwrap();
                        packed_checksum(v.chunks.as_slice()).unwrap()
                    } else {
                        let v: OwnedPacked = native::decode_request(PACKED, bytes, LIMITS).unwrap();
                        packed_checksum(&v.chunks).unwrap()
                    }
                },
                || {
                    if lane == 4 {
                        ic_wire::encode_request::<Binding<PackedManifest, PACKED>>(black_box(
                            &finite,
                        ))
                        .unwrap()
                        .into_bytes()
                    } else {
                        native::encode_request(PACKED, black_box(&owned), LIMITS).unwrap()
                    }
                },
            );
            assert_eq!(
                native::decode_request::<PackedManifest>(PACKED, &sample.bytes, LIMITS).unwrap(),
                finite
            );
            if lane == 6 {
                let view: PackedView<'_> =
                    native::decode_request(PACKED, &sample.bytes, LIMITS).unwrap();
                let range = sample.bytes.as_ptr() as usize
                    ..sample.bytes.as_ptr() as usize + sample.bytes.len();
                assert!(range.contains(&(view.header.content_type.as_ptr() as usize)));
                if n != 0 {
                    assert!(range.contains(&(view.chunks.as_ptr() as usize)));
                }
            }
            sample
        }
    } else {
        let c = case - 28;
        let n = [0, 32, 4096, 1048576][c / 4];
        let lane = c % 4;
        let finite = Bytes::<1048576>::new((0..n).map(|i| (i % 251) as u8).collect()).unwrap();
        let owned = serde_bytes::ByteBuf::from(finite.as_slice().to_vec());
        let mode = if lane == 1 {
            1
        } else if lane == 2 {
            2
        } else {
            0
        };
        let sample = measure::<_, BLOB>(
            &owned,
            mode,
            repetitions,
            clock,
            |bytes| {
                if lane == 3 {
                    let v: &serde_bytes::Bytes =
                        native::decode_request(BLOB, bytes, LIMITS).unwrap();
                    checksum(v.iter().copied())
                } else if lane == 1 {
                    let v =
                        ic_wire::decode_request::<Binding<Bytes<1048576>, BLOB>>(bytes).unwrap();
                    checksum(v.as_slice().iter().copied())
                } else {
                    let v: serde_bytes::ByteBuf =
                        native::decode_request(BLOB, bytes, LIMITS).unwrap();
                    checksum(v.iter().copied())
                }
            },
            || {
                if lane == 1 {
                    ic_wire::encode_request::<Binding<Bytes<1048576>, BLOB>>(black_box(&finite))
                        .unwrap()
                        .into_bytes()
                } else {
                    native::encode_request(BLOB, black_box(&owned), LIMITS).unwrap()
                }
            },
        );
        assert_eq!(
            native::decode_request::<Bytes<1048576>>(BLOB, &sample.bytes, LIMITS).unwrap(),
            finite
        );
        sample
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn packed_hash_shape_refuses_partial_and_oversized_elements() {
        assert_eq!(
            super::packed_checksum(&[0; 31]),
            Err(ic_wire::Error::Malformed)
        );
        assert_eq!(
            super::packed_checksum(&[0; 33]),
            Err(ic_wire::Error::Malformed)
        );
        assert_eq!(
            super::packed_checksum(&vec![0; 32769]),
            Err(ic_wire::Error::Limit)
        );
        assert_eq!(super::packed_checksum(&[1; 32]), Ok(32));
    }

    #[test]
    fn same_layout_lanes_match_and_layout_changes_have_distinct_identities() {
        for group in 0..4 {
            let samples = (0..7)
                .map(|lane| super::run(group * 7 + lane, 1, &mut || 0))
                .collect::<Vec<_>>();
            for lane in [1, 2] {
                assert_eq!(samples[0].bytes, samples[lane].bytes);
            }
            for lane in [4, 5, 6] {
                assert_eq!(samples[3].bytes, samples[lane].bytes);
            }
            assert_ne!(&samples[0].bytes[..11], &samples[3].bytes[..11]);
            let blobs = (0..4)
                .map(|lane| super::run(28 + group * 4 + lane, 1, &mut || 0))
                .collect::<Vec<_>>();
            for lane in 1..4 {
                assert_eq!(blobs[0].bytes, blobs[lane].bytes);
            }
        }
    }
}
