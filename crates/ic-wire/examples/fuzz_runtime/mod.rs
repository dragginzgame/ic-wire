//! Coverage-guided checks for bounded frames; see docs/development.md.
#![no_main]

use ic_wire::{
    Bytes, FiniteF32, FiniteF64, Limits, List, Method, Text, Wire, decode_request, decode_response,
    encode_request, encode_response, encoded_request_len, encoded_response_len,
};
use libfuzzer_sys::fuzz_target;

#[derive(Wire)]
enum Cell {
    #[wire(id = 0)]
    Empty,
    #[wire(id = 1)]
    Unsigned(u128),
    #[wire(id = 2)]
    Signed(i128),
    #[wire(id = 3)]
    Single(FiniteF32),
    #[wire(id = 4)]
    Double(FiniteF64),
    #[wire(id = 5)]
    Text(Text<32>),
    #[wire(id = 6)]
    Bytes(Bytes<32>),
    #[wire(id = 7)]
    Nested(List<Option<[bool; 3]>, 4>),
}

struct Probe;
impl Method for Probe {
    type Request = List<Cell, 8>;
    type Response = Result<Cell, Text<32>>;
    const ID: u64 = 0x8877_6655_4433_2211;
    const NAME: &'static str = "fuzz_probe";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 4096,
        allocation: 16384,
        nodes: 1024,
        depth: 16,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}

fn check_frame(bytes: &[u8]) {
    if let Ok(value) = decode_request::<Probe>(bytes) {
        assert_eq!(encode_request::<Probe>(&value).unwrap().as_bytes(), bytes);
        assert_eq!(encoded_request_len::<Probe>(&value).unwrap(), bytes.len());
        assert!(decode_response::<Probe>(bytes).is_err());
    }
    if let Ok(value) = decode_response::<Probe>(bytes) {
        assert_eq!(encode_response::<Probe>(&value).unwrap().as_bytes(), bytes);
        assert_eq!(encoded_response_len::<Probe>(&value).unwrap(), bytes.len());
        assert!(decode_request::<Probe>(bytes).is_err());
    }
}

fuzz_target!(|data: &[u8]| {
    // Exercise arbitrary envelope failures as well as payload parsing without
    // requiring the engine to discover an eleven-byte method identity first.
    check_frame(data);
    if data.len() > 2048 {
        return;
    }
    let request = encode_request::<Probe>(&List::new(vec![]).unwrap()).unwrap();
    let response = encode_response::<Probe>(&Ok(Cell::Empty)).unwrap();
    let mut frame = [0; 2059];
    frame[11..11 + data.len()].copy_from_slice(data);
    for header in [request.as_bytes(), response.as_bytes()] {
        frame[..11].copy_from_slice(&header[..11]);
        check_frame(&frame[..11 + data.len()]);
    }
});
