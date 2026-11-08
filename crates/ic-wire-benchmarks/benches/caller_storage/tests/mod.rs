use super::*;
use serde_json::json;
use std::{fs, path::PathBuf};

#[cfg(feature = "allocations")]
#[global_allocator]
static ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;
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
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires an explicitly prepared PocketIC server and Wasm artifacts"]
async fn ic_measurement() {
    use ic_platform::candid::contract::Principal;
    use ic_testkit::pic::PocketIcStartupConfig;
    use ic_testkit::pocket_ic::PocketIcBuilder;
    let output = PathBuf::from(std::env::var_os("IC_WIRE_STORAGE_EVIDENCE").unwrap());
    let owned = PocketIcStartupConfig::from_env(std::time::Duration::from_secs(60))
        .unwrap()
        .build_instance(
            PocketIcBuilder::new(std::time::Duration::from_secs(300)).with_application_subnet(),
        )
        .await
        .unwrap();
    let pic = owned.instance();
    let mut rows = Vec::new();
    let mut sample_caller = 0_u64;
    for profile in ["evaluation", "benchmark-speed"] {
        for lane in ["before-vec", "after-vec", "after-slice", "after-scoped"] {
            let canister = pic.create_canister().await.unwrap();
            pic.add_cycles(canister, 100_000_000_000_000).await.unwrap();
            pic.install_canister(
                canister,
                fs::read(output.join(format!("{profile}-{lane}.wasm"))).unwrap(),
                vec![],
                None,
            )
            .await
            .unwrap();
            for (case, label) in CASES.iter().enumerate() {
                let mut samples = Vec::new();
                let mut length = 0;
                let mut sample_caller_ids = Vec::new();
                for _ in 0..3 {
                    // Distinct callers prevent identical queries hitting the query cache.
                    sample_caller += 1;
                    sample_caller_ids.push(sample_caller);
                    let reply = pic
                        .query_call(
                            canister,
                            Principal::from_slice(&sample_caller.to_le_bytes()),
                            "benchmark",
                            vec![case as u8],
                        )
                        .await
                        .unwrap();
                    assert_eq!(reply.len(), 16);
                    samples.push(u64::from_le_bytes(reply[..8].try_into().unwrap()));
                    length = u64::from_le_bytes(reply[8..].try_into().unwrap());
                }
                rows.push(json!({"profile":profile,"lane":lane,"case":label,"bytes":length,"ic_instructions":samples,"sample_caller_ids":sample_caller_ids}));
            }
            pic.stop_canister(canister, None).await.unwrap();
            pic.delete_canister(canister, None).await.unwrap();
        }
    }
    fs::write(
        output.join("instructions.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    owned.close().await.unwrap();
}
