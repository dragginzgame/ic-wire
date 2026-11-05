//! IC instruction costs for admission budgets and valid/truncated controls.
mod admission;

#[cfg_attr(
    target_arch = "wasm32",
    expect(
        unsafe_code,
        reason = "This fixture owns the IC export; system calls use safe wrappers."
    )
)]
#[cfg_attr(
    target_arch = "wasm32",
    unsafe(export_name = "canister_query benchmark")
)]
/// Measure one decode; setup and reply transport are outside the counter.
pub extern "C" fn benchmark() {
    let mut args = [0_u8; 4];
    assert_eq!(ic_platform::ic0::msg_arg_data_size(), args.len());
    ic_platform::ic0::msg_arg_data_copy(&mut args, 0);
    let [kind, length, policy, truncated] = args.map(usize::from);
    assert!(kind < admission::KINDS.len() && policy < admission::POLICIES.len());
    assert!(truncated <= 1);
    let bytes = admission::frame(kind, admission::LENGTHS[length], truncated != 0);
    let limits = admission::limits(policy);
    let expected = admission::decode(kind, &bytes, limits);
    let start = ic_platform::ic0::performance_counter(0);
    let result = admission::decode(kind, std::hint::black_box(&bytes), limits);
    let count = ic_platform::ic0::performance_counter(0) - start;
    assert_eq!(result, expected);
    ic_platform::ic0::msg_reply_data_append(&count.to_le_bytes());
    ic_platform::ic0::msg_reply_data_append(&[result]);
    ic_platform::ic0::msg_reply();
}
