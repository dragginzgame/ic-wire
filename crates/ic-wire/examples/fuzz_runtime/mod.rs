//! Coverage-guided checks for bounded frames; see docs/development.md.
#![no_main]

use ic_wire::{Bytes, FiniteF32, FiniteF64, Limits, List, Text, native};
use libfuzzer_sys::fuzz_target;

#[derive(serde::Serialize, serde::Deserialize)]
enum Cell {
    Empty,
    Unsigned(u128),
    Signed(i128),
    Single(FiniteF32),
    Double(FiniteF64),
    Text(Text<32>),
    Bytes(Bytes<32>),
    Nested(List<Option<[bool; 3]>, 4>),
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

fn check_frame(bytes: &[u8]) {
    if let Ok(value) = native::decode_request::<Request>(ID, bytes, LIMITS) {
        assert_eq!(
            native::encode_request(ID, &value, LIMITS)
                .unwrap()
                .as_slice(),
            bytes
        );
        assert_eq!(
            native::encoded_frame_len(&value, LIMITS).unwrap(),
            bytes.len()
        );
        assert!(native::decode_response::<Response>(ID, bytes, LIMITS).is_err());
    }
    if let Ok(value) = native::decode_response::<Response>(ID, bytes, LIMITS) {
        assert_eq!(
            native::encode_response(ID, &value, LIMITS)
                .unwrap()
                .as_slice(),
            bytes
        );
        assert_eq!(
            native::encoded_frame_len(&value, LIMITS).unwrap(),
            bytes.len()
        );
        assert!(native::decode_request::<Request>(ID, bytes, LIMITS).is_err());
    }
}

fuzz_target!(|data: &[u8]| {
    // Exercise arbitrary envelope failures as well as payload parsing without
    // requiring the engine to discover an eleven-byte method identity first.
    check_frame(data);
    if data.len() > 2048 {
        return;
    }
    let request =
        native::encode_request::<Request>(ID, &List::new(vec![]).unwrap(), LIMITS).unwrap();
    let response = native::encode_response::<Response>(ID, &Ok(Cell::Empty), LIMITS).unwrap();
    let mut frame = [0; 2059];
    frame[11..11 + data.len()].copy_from_slice(data);
    for header in [request.as_slice(), response.as_slice()] {
        frame[..11].copy_from_slice(&header[..11]);
        check_frame(&frame[..11 + data.len()]);
    }
});
