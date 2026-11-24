//! Coverage-guided decode of bounded payloads and request frames.
#![no_main]

use libfuzzer_sys::fuzz_target;
use wire::{Direction, Encoding, Limits, Packed};

fuzz_target!(|input: &[u8]| {
    let limits = Limits {
        bytes: 64,
        allocation: 2048,
        nodes: 16,
        depth: 4,
    };
    let frame = Encoding::Frame {
        method: 7,
        direction: Direction::Request,
    };
    for encoding in [Encoding::Payload, frame] {
        let _ = wire::decode::<(Vec<Option<u8>>, Packed<i16, 8>)>(input, encoding, limits);
    }
});
