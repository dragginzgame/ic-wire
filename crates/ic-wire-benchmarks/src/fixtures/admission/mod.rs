//! Illustrative admission budgets, not recommendations for an installed endpoint.
use ic_wire::{ByteBuf, Error, Limits, native};
use std::hint::black_box;

pub const LENGTHS: [usize; 6] = [0, 1, 64, 1024, 65536, 1048576];
pub const POLICIES: [&str; 4] = ["generous", "nodes-1024", "heap-headroom-64k", "frame-64k"];
pub const KINDS: [&str; 3] = ["ordinary-u8", "byte-buffer", "ordinary-u64"];
const ID: u64 = 7;

pub fn limits(policy: usize) -> Limits {
    let mut limits = Limits {
        bytes: 16 * 1024 * 1024,
        allocation: 256 * 1024 * 1024,
        nodes: 8 * 1024 * 1024,
        depth: 64,
    };
    match policy {
        0 => {}
        1 => limits.nodes = 1024,
        2 => limits.allocation = limits.bytes + 65536,
        3 => limits.bytes = 65536,
        _ => panic!("unknown budget"),
    }
    limits
}

pub fn frame(kind: usize, length: usize, truncated: bool) -> Vec<u8> {
    let mut bytes = match kind {
        0 => native::encode_request(ID, &vec![42_u8; length], limits(0)),
        1 => native::encode_request(ID, &ByteBuf::from(vec![42_u8; length]), limits(0)),
        2 => native::encode_request(ID, &vec![u64::MAX; length], limits(0)),
        _ => panic!("unknown field kind"),
    }
    .unwrap();
    if truncated {
        bytes.pop();
    }
    bytes
}

pub fn decode(kind: usize, bytes: &[u8], limits: Limits) -> u8 {
    let result = match kind {
        0 => native::decode_request::<Vec<u8>>(ID, bytes, limits).map(|v| {
            black_box(v);
        }),
        1 => native::decode_request::<ByteBuf>(ID, bytes, limits).map(|v| {
            black_box(v);
        }),
        2 => native::decode_request::<Vec<u64>>(ID, bytes, limits).map(|v| {
            black_box(v);
        }),
        _ => panic!("unknown field kind"),
    };
    match result {
        Ok(()) => 0,
        Err(Error::Limit) => 1,
        Err(Error::Malformed) => 2,
        Err(error) => panic!("unexpected error: {error}"),
    }
}
