//! Isolated codec footprint and instruction fixture, without an endpoint SDK.
#[path = "../../fixtures/mod.rs"]
mod fixtures;
use std::hint::black_box;

// This fixture owns this export; all IC calls use safe wrappers.
#[allow(unsafe_code)]
#[cfg_attr(
    target_arch = "wasm32",
    unsafe(export_name = "canister_query benchmark")
)]
/// Measure all operations for one workload using the IC instruction counter.
pub extern "C" fn benchmark() {
    let mut args = [0_u8; 12];
    assert_eq!(ic_platform::ic0::msg_arg_data_size(), args.len());
    ic_platform::ic0::msg_arg_data_copy(&mut args, 0);
    let get = |i| u32::from_le_bytes(args[i..i + 4].try_into().unwrap()) as usize;
    let (case_id, lane_id, repetitions) = (get(0), get(4), get(8));
    assert!((1..=1024).contains(&repetitions));
    let cases = fixtures::cases_selected(Some(case_id));
    let lane = &cases[0].lanes[lane_id];
    let mut buffer = lane.bytes.clone();
    let mut invalid = lane.bytes.clone();
    invalid[0] ^= 0xff;
    let mut counts = [u64::MAX; 6];
    for (operation, count) in counts.iter_mut().enumerate() {
        if operation == 2 && lane.reuse.is_none() {
            continue;
        }
        let mut run = || match operation {
            0 => {
                black_box((lane.encode)());
            }
            1 => {
                assert!(black_box((lane.decode)(&lane.bytes)));
            }
            2 => {
                lane.reuse.as_ref().unwrap()(&mut buffer);
                black_box(&buffer);
            }
            3 => {
                assert!(!black_box((lane.decode)(&invalid)));
            }
            4 => {
                assert!(!black_box((lane.decode)(
                    &lane.bytes[..lane.bytes.len() - 1]
                )));
            }
            5 => {
                black_box(&lane.bytes);
            }
            _ => unreachable!(),
        };
        run();
        let start = ic_platform::ic0::performance_counter(0);
        for _ in 0..repetitions {
            run();
        }
        *count = ic_platform::ic0::performance_counter(0) - start;
    }
    for count in counts {
        ic_platform::ic0::msg_reply_data_append(&count.to_le_bytes());
    }
    ic_platform::ic0::msg_reply_data_append(&(lane.bytes.len() as u64).to_le_bytes());
    ic_platform::ic0::msg_reply_data_append(&lane.bytes);
    ic_platform::ic0::msg_reply();
}
