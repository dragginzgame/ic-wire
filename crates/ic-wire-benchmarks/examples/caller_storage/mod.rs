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
    scratch: &mut [u8],
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
                let bytes = native::encode_to_slice(&black_box(value), LIMITS, scratch).unwrap();
                black_box(&bytes);
                bytes.len()
            }
            #[cfg(not(any(test, feature = "caller-slice")))]
            unreachable!("slice lane is not selected")
        }
        _ => {
            #[cfg(any(test, feature = "caller-scoped"))]
            {
                native::with_encoded(black_box(value), LIMITS, &mut scratch[..128], |bytes| {
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
#[allow(unsafe_code)]
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
mod tests {
    use super::*;
    use serde_json::json;
    use std::{fs, path::PathBuf};

    #[cfg(feature = "allocations")]
    #[global_allocator]
    static ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> =
        &stats_alloc::INSTRUMENTED_SYSTEM;
    #[cfg(feature = "heap")]
    #[global_allocator]
    static HEAP: dhat::Alloc = dhat::Alloc;
    #[cfg(all(feature = "allocations", feature = "heap"))]
    compile_error!("Use separate allocation and heap runs");

    #[test]
    #[ignore = "explicit matched measurement, not a correctness gate"]
    fn native_measurement() {
        let mut rows = Vec::new();
        for (case, label) in CASES.iter().enumerate() {
            let input = Input::new(case);
            let mut scratch = vec![0; CAPACITY];
            let mut reuse = Vec::new();
            let expected = input.run(1, &mut scratch, &mut reuse);
            for lane in 0..4 {
                assert_eq!(input.run(lane, &mut scratch, &mut reuse), expected);
                if lane == 2 || (lane == 3 && expected <= 128) {
                    assert_eq!(&scratch[..expected], reuse);
                }
                let capacity = if lane == 3 {
                    128
                } else if lane == 2 {
                    scratch.len()
                } else {
                    reuse.capacity()
                };
                let mut samples = Vec::new();
                for _ in 0..7 {
                    #[cfg(feature = "allocations")]
                    let sample = {
                        let region = stats_alloc::Region::new(ALLOCATOR);
                        input.run(lane, &mut scratch, &mut reuse);
                        let s = region.change();
                        json!({"allocations":s.allocations,"reallocations":s.reallocations,"requested_bytes":s.bytes_allocated,"deallocated_bytes":s.bytes_deallocated})
                    };
                    #[cfg(feature = "heap")]
                    let sample = {
                        let profiler = dhat::Profiler::builder()
                            .testing()
                            .trim_backtraces(Some(0))
                            .build();
                        input.run(lane, &mut scratch, &mut reuse);
                        let s = dhat::HeapStats::get();
                        drop(profiler);
                        json!({"peak_live_bytes":s.max_bytes,"total_bytes":s.total_bytes,"retained_bytes":s.curr_bytes})
                    };
                    #[cfg(not(any(feature = "allocations", feature = "heap")))]
                    let sample = {
                        let repetitions = if expected < 1024 { 4096 } else { 64 };
                        let start = std::time::Instant::now();
                        for _ in 0..repetitions {
                            black_box(input.run(lane, &mut scratch, &mut reuse));
                        }
                        json!({"repetitions":repetitions,"native_ns":start.elapsed().as_nanos()})
                    };
                    samples.push(sample);
                }
                rows.push(json!({"case":label,"lane":(["vec-fresh","vec-reuse","slice","scoped"][lane]),"bytes":expected,"caller_capacity":capacity,"samples":samples}));
            }
        }
        let output = PathBuf::from(
            std::env::var_os("IC_WIRE_STORAGE_EVIDENCE").expect("fresh evidence directory"),
        );
        let mode = if cfg!(feature = "allocations") {
            "allocations"
        } else if cfg!(feature = "heap") {
            "heap"
        } else {
            "time"
        };
        fs::write(
            output.join(format!("native-{mode}.json")),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }

    #[cfg(feature = "ic")]
    #[test]
    #[ignore = "requires an explicitly prepared PocketIC server and Wasm artifacts"]
    fn ic_measurement() {
        use ic_platform::{candid::contract::Principal, pocket_ic::PocketIcBuilder};
        use ic_testkit::pic::PocketIcStartupConfig;
        let output = PathBuf::from(std::env::var_os("IC_WIRE_STORAGE_EVIDENCE").unwrap());
        let owned = PocketIcStartupConfig::from_env(std::time::Duration::from_secs(60))
            .unwrap()
            .build_instance(PocketIcBuilder::new().with_application_subnet())
            .unwrap();
        let pic = owned.instance();
        let mut rows = Vec::new();
        for profile in ["evaluation", "benchmark-speed"] {
            for lane in ["before-vec", "after-vec", "after-slice", "after-scoped"] {
                let canister = pic.create_canister();
                pic.add_cycles(canister, 100_000_000_000_000);
                pic.install_canister(
                    canister,
                    fs::read(output.join(format!("{profile}-{lane}.wasm"))).unwrap(),
                    vec![],
                    None,
                );
                for (case, label) in CASES.iter().enumerate() {
                    let mut samples = Vec::new();
                    let mut length = 0;
                    for _ in 0..3 {
                        let reply = pic
                            .query_call(
                                canister,
                                Principal::anonymous(),
                                "benchmark",
                                vec![case as u8],
                            )
                            .unwrap();
                        assert_eq!(reply.len(), 16);
                        samples.push(u64::from_le_bytes(reply[..8].try_into().unwrap()));
                        length = u64::from_le_bytes(reply[8..].try_into().unwrap());
                    }
                    rows.push(json!({"profile":profile,"lane":lane,"case":label,"bytes":length,"ic_instructions":samples}));
                }
                pic.stop_canister(canister, None).unwrap();
                pic.delete_canister(canister, None).unwrap();
            }
        }
        fs::write(
            output.join("instructions.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}
