//! Explicit caller-storage study. Run ignored host tests through the evidence runner.
#![cfg(any(target_family = "wasm", test))]
use ic_wire::{Limits, native};
use serde::Serialize;
use std::hint::black_box;

const LIMITS: Limits = Limits {
    bytes: 131072,
    allocation: 8 * 1024 * 1024,
    nodes: 131072,
    depth: 64,
};
const CASES: [&str; 9] = [
    "unit",
    "u64",
    "vec-empty",
    "vec-8",
    "vec-4096",
    "blob-empty",
    "blob-32",
    "blob-65536",
    "record",
];
const CAPACITY: usize = 131072;

#[derive(Serialize)]
struct Record {
    counts: [u64; 2],
    name: String,
}
struct Input {
    case: usize,
    words: Vec<u32>,
    bytes: ic_wire::ByteBuf,
    record: Record,
}
impl Input {
    fn new(case: usize) -> Self {
        let length = match case {
            3 => 8,
            4 => 4096,
            6 => 32,
            7 => 65536,
            _ => 0,
        };
        Self {
            case,
            words: vec![u32::MAX; if case < 5 { length } else { 0 }],
            bytes: ic_wire::ByteBuf::from(vec![0xa5; if case >= 5 { length } else { 0 }]),
            record: Record {
                counts: [u64::MAX, 1],
                name: "é🦀".into(),
            },
        }
    }
    fn run(&self, lane: usize, scratch: &mut [u8], reuse: &mut Vec<u8>) -> usize {
        match self.case {
            0 => operation(&(), lane, scratch, reuse),
            1 => operation(&7_u64, lane, scratch, reuse),
            2..=4 => operation(&self.words, lane, scratch, reuse),
            5..=7 => operation(&self.bytes, lane, scratch, reuse),
            _ => operation(&self.record, lane, scratch, reuse),
        }
    }
}
fn operation<T: Serialize>(
    value: &T,
    lane: usize,
    _scratch: &mut [u8],
    reuse: &mut Vec<u8>,
) -> usize {
    match lane {
        0 => {
            // All lanes admit the same borrowed root (&T), including scoped
            // encoding which takes that reference by value. This matters to
            // the size-based admission ledger, not just payload equality.
            let bytes = native::encode(&black_box(value), LIMITS).unwrap();
            black_box(bytes.as_slice());
            bytes.len()
        }
        1 => {
            native::encode_into(&black_box(value), LIMITS, reuse).unwrap();
            black_box(reuse.as_slice());
            reuse.len()
        }
        2 => {
            #[cfg(any(test, feature = "caller-slice"))]
            {
                let bytes = native::encode_to_slice(&black_box(value), LIMITS, _scratch).unwrap();
                black_box(&bytes);
                bytes.len()
            }
            #[cfg(not(any(test, feature = "caller-slice")))]
            unreachable!("slice lane is not selected")
        }
        _ => {
            #[cfg(any(test, feature = "caller-scoped"))]
            {
                native::with_encoded(black_box(value), LIMITS, &mut _scratch[..128], |bytes| {
                    black_box(bytes);
                    bytes.len()
                })
                .unwrap()
            }
            #[cfg(not(any(test, feature = "caller-scoped")))]
            unreachable!("scoped lane is not selected")
        }
    }
}

#[cfg(target_family = "wasm")]
#[expect(
    unsafe_code,
    reason = "This fixture owns the IC export; system calls use safe wrappers."
)]
#[unsafe(export_name = "canister_query benchmark")]
/// One codec operation, including owned-output destruction; setup/transport excluded.
pub extern "C" fn benchmark() {
    let mut argument = [0];
    assert_eq!(ic_platform::ic0::msg_arg_data_size(), 1);
    ic_platform::ic0::msg_arg_data_copy(&mut argument, 0);
    let case = usize::from(argument[0]);
    assert!(case < CASES.len());
    let input = Input::new(case);
    let mut scratch = vec![0; CAPACITY];
    let mut reuse = Vec::new();
    let start = ic_platform::ic0::performance_counter(0);
    let length = input.run(
        if cfg!(feature = "caller-scoped") {
            3
        } else if cfg!(feature = "caller-slice") {
            2
        } else {
            0
        },
        &mut scratch,
        &mut reuse,
    );
    let count = ic_platform::ic0::performance_counter(0) - start;
    ic_platform::ic0::msg_reply_data_append(&count.to_le_bytes());
    ic_platform::ic0::msg_reply_data_append(&(length as u64).to_le_bytes());
    ic_platform::ic0::msg_reply();
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests;
