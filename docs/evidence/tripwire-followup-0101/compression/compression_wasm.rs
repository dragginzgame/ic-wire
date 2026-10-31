//! Research-only transport cost and marginal Wasm fixture.
#![cfg(target_family = "wasm")]
#[cfg(feature = "compression")]
mod compression_transport;
fn run(decode: bool) {
    let length = ic_platform::ic0::msg_arg_data_size();
    assert!(length <= 2 * 1024 * 1024 + 65536);
    let mut input = vec![0; length];
    ic_platform::ic0::msg_arg_data_copy(&mut input, 0);
    // Body counter includes output allocation and codec, excludes input copying,
    // export setup, reply system calls and transmission. Unique callers bypass cache.
    let start = ic_platform::ic0::performance_counter(0);
    #[cfg(feature = "compression")]
    let output = if decode { compression_transport::decompress(&input).unwrap() }
        else { compression_transport::compress(&input) };
    #[cfg(not(feature = "compression"))]
    let output = { let _ = decode; input.clone() };
    let cost = ic_platform::ic0::performance_counter(0) - start;
    ic_platform::ic0::msg_reply_data_append(&cost.to_le_bytes());
    ic_platform::ic0::msg_reply_data_append(&output);
    ic_platform::ic0::msg_reply();
}
#[expect(unsafe_code, reason = "Only the IC export attribute; all system calls are safe wrappers.")]
#[unsafe(export_name = "canister_query compress")]
/// Research compression entry.
pub extern "C" fn compress() { run(false); }
#[expect(unsafe_code, reason = "Only the IC export attribute; all system calls are safe wrappers.")]
#[unsafe(export_name = "canister_query decompress")]
/// Research decompression entry.
pub extern "C" fn decompress() { run(true); }
