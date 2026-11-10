//! Coverage-guided checks for bounded frames; see docs/guides/development.md.
#![no_main]

use ic_wire::{Bytes, FiniteF32, FiniteF64, Limits, List, Packed, Text, native};
use libfuzzer_sys::fuzz_target;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum Cell {
    Empty,
    Unsigned(u128),
    Signed(i128),
    Single(FiniteF32),
    Double(FiniteF64),
    Text(Text<32>),
    Bytes(Bytes<32>),
    Nested(List<Option<[bool; 3]>, 4>),
    PackedI16(Packed<i16>),
    PackedU16(Packed<u16>),
    PackedI32(Packed<i32>),
    PackedU32(Packed<u32>),
    PackedI64(Packed<i64>),
    PackedU64(Packed<u64>),
    PackedHashes(Packed<[u8; 32]>),
    PackedI8(Packed<i8>),
}

type Request = List<Cell, 8>;
type Response = Result<Cell, Text<32>>;
const ID: u64 = 0x8877_6655_4433_2211;
const LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 16384,
    nodes: 1024,
    depth: 16,
};

// Decode no longer applies the encoder budget. Give re-encoding its own
// allowance when checking normalized values; input bytes may be nonminimal.
const ENCODE_LIMITS: Limits = Limits {
    // Zero-sized decoded elements can still become sized references during
    // Serde encoding. Leave headroom for that target-dependent visitor work.
    allocation: 1_048_576,
    nodes: 8192,
    depth: 128,
    ..LIMITS
};

fn check_frame(bytes: &[u8], limits: Limits) {
    if let Ok(value) = native::decode_request::<Request>(ID, bytes, limits) {
        let encoded = native::encode_request(ID, &value, ENCODE_LIMITS).unwrap();
        assert_eq!(
            native::decode_request::<Request>(ID, &encoded, ENCODE_LIMITS).unwrap(),
            value
        );
        assert_eq!(
            native::encoded_frame_len(&value, ENCODE_LIMITS).unwrap(),
            encoded.len()
        );
        assert!(native::decode_response::<Response>(ID, bytes, LIMITS).is_err());
    }
    if let Ok(value) = native::decode_response::<Response>(ID, bytes, limits) {
        let encoded = native::encode_response(ID, &value, ENCODE_LIMITS).unwrap();
        assert_eq!(
            native::decode_response::<Response>(ID, &encoded, ENCODE_LIMITS).unwrap(),
            value
        );
        assert_eq!(
            native::encoded_frame_len(&value, ENCODE_LIMITS).unwrap(),
            encoded.len()
        );
        assert!(native::decode_request::<Request>(ID, bytes, LIMITS).is_err());
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum Tree {
    Empty,
    Children(Vec<Tree>),
    Next(Box<Tree>),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct Borrowed<'a> {
    #[serde(borrow)]
    label: ic_wire::TextRef<'a, 1024>,
    #[serde(borrow)]
    data: ic_wire::BytesRef<'a, 1024>,
}

fn semantic_payload<T: serde::Serialize + serde::de::DeserializeOwned + Debug + PartialEq>(
    data: &[u8],
    limits: Limits,
) {
    if let Ok(value) = native::decode::<T>(data, limits) {
        // Maps/sets may normalize duplicates/order: compare values, not original bytes.
        let encoded = native::encode(&value, ENCODE_LIMITS).unwrap();
        assert_eq!(native::decode::<T>(&encoded, ENCODE_LIMITS).unwrap(), value);
        let mut reused = vec![255; 16];
        native::encode_into(&value, ENCODE_LIMITS, &mut reused).unwrap();
        assert_eq!(reused, encoded);
        let mut storage = vec![0; encoded.len()];
        assert_eq!(
            native::encode_to_slice(&value, ENCODE_LIMITS, &mut storage).unwrap(),
            encoded
        );
        let mut scratch = [0; 32];
        native::with_encoded(&value, ENCODE_LIMITS, &mut scratch, |bytes| {
            assert_eq!(bytes, encoded)
        })
        .unwrap();
    }
}

fuzz_target!(|data: &[u8]| {
    // Exercise arbitrary envelope failures as well as payload parsing without
    // requiring the engine to discover an eleven-byte method identity first.
    check_frame(data, LIMITS);
    let byte = |index| usize::from(data.get(index).copied().unwrap_or(0));
    let limits = Limits {
        bytes: (byte(0) * 16).min(LIMITS.bytes),
        allocation: byte(1) * 64,
        nodes: byte(2) * 4,
        depth: byte(3) % 17,
    };
    check_frame(data, limits);
    for payload_limits in [LIMITS, limits] {
        semantic_payload::<BTreeMap<u16, bool>>(data, payload_limits);
        semantic_payload::<BTreeSet<u16>>(data, payload_limits);
        semantic_payload::<Tree>(data, payload_limits);
        semantic_payload::<Vec<()>>(data, payload_limits);
        if let Ok(view) = native::decode::<Borrowed<'_>>(data, payload_limits) {
            for bytes in [view.label.as_str().as_bytes(), view.data.as_slice()] {
                assert!(bytes.as_ptr() as usize >= data.as_ptr() as usize);
                assert!(
                    bytes.as_ptr() as usize + bytes.len() <= data.as_ptr() as usize + data.len()
                );
            }
            let encoded = native::encode(&view, ENCODE_LIMITS).unwrap();
            assert_eq!(
                native::decode::<Borrowed<'_>>(&encoded, ENCODE_LIMITS).unwrap(),
                view
            );
        }
    }
    if data.len() > 4096 - 11 {
        return;
    }
    let request =
        native::encode_request::<Request>(ID, &List::new(vec![]).unwrap(), LIMITS).unwrap();
    let response = native::encode_response::<Response>(ID, &Ok(Cell::Empty), LIMITS).unwrap();
    let mut frame = [0; 4096];
    frame[11..11 + data.len()].copy_from_slice(data);
    for header in [request.as_slice(), response.as_slice()] {
        frame[..11].copy_from_slice(&header[..11]);
        check_frame(&frame[..11 + data.len()], LIMITS);
        check_frame(&frame[..11 + data.len()], limits);
    }
});
