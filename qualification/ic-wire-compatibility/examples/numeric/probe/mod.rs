//! Matched integer workloads for the native allocator and IC instruction probes.

#[cfg(not(target_arch = "wasm32"))]
pub(super) mod host;

use super::{Value, decode, encode, prepare};
use std::hint::black_box;

fn workload(len: usize, stride: usize) -> Value {
    prepare(
        (0..len)
            .map(|i| {
                if stride != 0 && i % stride == 0 {
                    u32::MAX
                } else {
                    1
                }
            })
            .collect(),
    )
}

#[cfg(target_arch = "wasm32")]
#[expect(
    unsafe_code,
    reason = "Fixed IC query export for the qualification guest"
)]
#[unsafe(export_name = "canister_query measure")]
extern "C" fn measure() {
    use ic_platform_runtime::raw;
    if raw::msg_arg_data_size() != 0 {
        raw::trap(b"expected empty request");
    }
    for (len, stride) in [1, 16, 127, 128, 4096]
        .into_iter()
        .flat_map(|len| [0, 16, 1].map(|stride| (len, stride)))
    {
        let value = workload(black_box(len), black_box(stride));
        let bytes = encode(&value);
        assert_eq!(decode(&bytes), value);
        let start = raw::instruction_counter();
        drop(black_box(encode(black_box(&value))));
        let middle = raw::instruction_counter();
        drop(black_box(decode(black_box(&bytes))));
        let end = raw::instruction_counter();
        for count in [bytes.len() as u64, middle - start, end - middle] {
            raw::msg_reply_data_append(&count.to_le_bytes());
        }
    }
    raw::msg_reply();
}
