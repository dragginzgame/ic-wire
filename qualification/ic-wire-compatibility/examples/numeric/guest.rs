use super::{
    decode, encode,
    workload::{cases, workload},
};
use ic_platform_runtime::raw;
use std::hint::black_box;

// measure
// Return payload sizes and instruction counts for 15 fixed workloads.
// limits : empty request, 360-byte reply
#[expect(
    unsafe_code,
    reason = "Fixed IC query export for the qualification guest"
)]
#[unsafe(export_name = "canister_query measure")]
extern "C" fn measure() {
    if raw::msg_arg_data_size() != 0 {
        raw::trap(b"expected empty request");
    }
    for (len, stride) in cases() {
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
