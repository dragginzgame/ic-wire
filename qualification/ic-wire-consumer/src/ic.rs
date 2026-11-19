//! Private IC counter wrapper; SDK is confined to this optional qualification guest.
// Rust 2024 requires an unsafe attribute for this fixed IC ABI export.
// All memory and system-call operations remain in published Runtime.
// measure
// Return instruction counts and frames for the upload workload.
// scope  : public
// limits : empty request, fixed workload
#[cfg(target_arch = "wasm32")]
#[expect(unsafe_code, reason = "Fixed no-argument IC query export")]
#[unsafe(export_name = "canister_query measure")]
extern "C" fn measure() {
    use ic_platform_runtime::raw;

    if raw::msg_arg_data_size() != 0 {
        raw::trap(b"expected empty counter request");
    }
    let reply = ic_wire_consumer::instruction_reply(raw::instruction_counter);
    raw::msg_reply_data_append(&reply);
    raw::msg_reply();
}
