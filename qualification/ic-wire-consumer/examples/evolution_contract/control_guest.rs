//! Installed instruction control for the nested evolution workload.
#[cfg_attr(
    feature = "evolution-b-only",
    allow(dead_code, reason = "The B-only guest omits revisions A and C")
)]
mod control;

use control::{Batch, CASES, LIMITS, REQUEST};
use ic_platform_runtime::raw;
use serde::Serialize;
use std::hint::black_box;

fn probe<R: Serialize>(
    count: usize,
    dense: bool,
    make_row: impl Fn(usize, bool) -> Result<R, wire::Error>,
    decode: impl Fn(&[u8]),
    suffix_ok: impl Fn(&[u8]) -> bool,
) -> [u64; 4] {
    let count = black_box(count);
    let dense = black_box(dense);
    let build = || {
        control::build(count, dense, &make_row)
            .unwrap_or_else(|_| raw::trap(b"construct positional fixture"))
    };
    #[cfg(not(feature = "evolution-caller-buffer"))]
    let encode = || {
        let batch = build();
        let mut bytes = Vec::new();
        wire::encode_into(&batch, REQUEST, LIMITS, &mut bytes)
            .unwrap_or_else(|_| raw::trap(b"encode positional fixture"));
        bytes
    };
    #[cfg(feature = "evolution-caller-buffer")]
    let encode = |buffer: &mut [u8]| {
        let batch = build();
        let frame = wire::encode_to_slice(&batch, REQUEST, LIMITS, buffer)
            .unwrap_or_else(|_| raw::trap(b"encode positional fixture into caller buffer"));
        black_box(frame).len()
    };
    #[cfg(feature = "evolution-caller-buffer")]
    let mut bytes = vec![0; LIMITS.bytes];
    #[cfg(feature = "evolution-caller-buffer")]
    let len = encode(&mut bytes);
    #[cfg(not(feature = "evolution-caller-buffer"))]
    let bytes = encode();
    #[cfg(not(feature = "evolution-caller-buffer"))]
    let len = bytes.len();

    if !suffix_ok(&bytes[..len]) {
        raw::trap(b"fixture suffix");
    }
    let start = raw::instruction_counter();
    for _ in 0..8 {
        #[cfg(feature = "evolution-caller-buffer")]
        let _ = encode(&mut bytes);
        #[cfg(not(feature = "evolution-caller-buffer"))]
        drop(black_box(encode()));
    }
    let middle = raw::instruction_counter();
    for _ in 0..8 {
        decode(black_box(&bytes[..len]));
    }
    let end = raw::instruction_counter();
    [len as u64, middle - start, end - middle, 0]
}

#[expect(
    unsafe_code,
    reason = "Fixed IC query export for installed qualification"
)]
#[unsafe(export_name = "canister_query measure")]
extern "C" fn measure() {
    if raw::msg_arg_data_size() != 0 {
        raw::trap(b"expected empty request");
    }
    for (_, revision, count, dense) in CASES {
        macro_rules! run {
            ($make:ident, $row:ident) => {{
                type Row<'a> = control::$row<'a>;
                fn decode(bytes: &[u8]) -> Batch<'_, Row<'_>> {
                    wire::decode(bytes, REQUEST, LIMITS)
                        .unwrap_or_else(|_| raw::trap(b"decode positional fixture"))
                }
                probe(
                    count,
                    dense,
                    control::$make,
                    |bytes| drop(black_box(decode(bytes))),
                    |bytes| decode(bytes).suffix.as_str() == "tail ✅",
                )
            }};
        }
        #[cfg(feature = "evolution-b-only")]
        let counts = if revision == 1 {
            run!(row_b, RowB)
        } else {
            continue;
        };
        #[cfg(not(feature = "evolution-b-only"))]
        let counts = match revision {
            0 => run!(row_a, RowA),
            1 => run!(row_b, RowB),
            _ => run!(row_c, RowC),
        };
        for count in counts {
            raw::msg_reply_data_append(&count.to_le_bytes());
        }
    }
    raw::msg_reply();
}
