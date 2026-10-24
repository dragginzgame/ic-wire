//! Native codec measurements. Allocator instrumentation uses a separate build.
#[path = "../../fixtures/mod.rs"]
mod fixtures;
use serde_json::json;
use sha2::{Digest, Sha256};
#[cfg(not(any(feature = "allocations", feature = "heap")))]
use std::time::Instant;
use std::{fs, hint::black_box, io::Write, path::PathBuf, process::Command};
#[cfg(all(feature = "allocations", feature = "heap"))]
compile_error!("Run allocations and heap in separate builds");
#[cfg(feature = "heap")]
#[global_allocator]
static HEAP: dhat::Alloc = dhat::Alloc;
#[cfg(feature = "allocations")]
#[global_allocator]
static ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;

fn command(program: &str, args: &[&str]) -> String {
    let output = Command::new(program).args(args).output().unwrap();
    assert!(output.status.success(), "{program} failed");
    String::from_utf8(output.stdout).unwrap()
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[cfg(not(any(feature = "allocations", feature = "heap")))]
fn elapsed(repetitions: u64, operation: &mut dyn FnMut()) -> u128 {
    let start = Instant::now();
    for _ in 0..repetitions {
        operation();
    }
    start.elapsed().as_nanos()
}
fn measure(operation: &mut dyn FnMut(), samples: usize, millis: u128) -> serde_json::Value {
    operation(); // Warm caches; cold process startup is a separate cost.
    #[cfg(feature = "heap")]
    {
        let profiler = dhat::Profiler::builder()
            .testing()
            .trim_backtraces(Some(0))
            .build();
        operation();
        let stats = dhat::HeapStats::get();
        drop(profiler);
        let _ = (samples, millis);
        json!({"peak_live_bytes":stats.max_bytes,"peak_live_blocks":stats.max_blocks,
            "total_bytes":stats.total_bytes,"total_blocks":stats.total_blocks,"retained_bytes":stats.curr_bytes})
    }
    #[cfg(feature = "allocations")]
    {
        let region = stats_alloc::Region::new(ALLOCATOR);
        operation();
        let s = region.change();
        let _ = (samples, millis);
        json!({"allocations":s.allocations,"deallocations":s.deallocations,"reallocations":s.reallocations,
            "bytes_allocated":s.bytes_allocated,"bytes_deallocated":s.bytes_deallocated,"net_reallocated_bytes":s.bytes_reallocated})
    }
    #[cfg(not(any(feature = "allocations", feature = "heap")))]
    {
        let mut repetitions = 1;
        while elapsed(repetitions, operation) < millis * 1_000_000 && repetitions < 1_048_576 {
            repetitions *= 2;
        }
        let raw: Vec<_> = (0..samples)
            .map(|_| elapsed(repetitions, operation) as f64 / repetitions as f64)
            .collect();
        let mut sorted = raw.clone();
        sorted.sort_by(f64::total_cmp);
        json!({"repetitions":repetitions,"samples_ns_per_op":raw,"min_ns":sorted[0],
            "median_ns":sorted[samples/2],"p95_batch_mean_ns":sorted[((samples*95).div_ceil(100)-1).min(samples-1)],
            "max_ns":sorted[samples-1]})
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args()
        .skip(1)
        .filter(|a| a != "--bench")
        .collect();
    assert!(
        (2..=4).contains(&args.len()),
        "usage: suite OUTPUT PROFILE [SAMPLES=11] [BATCH_MS=2]"
    );
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(&args[0]);
    let profile = &args[1];
    assert!(["evaluation", "benchmark-speed"].contains(&profile.as_str()));
    let samples: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(11);
    let millis: u128 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(2);
    assert!((3..=101).contains(&samples) && (1..=100).contains(&millis));
    fs::create_dir(&directory)?;
    fs::write(
        directory.join("fixture.rs"),
        include_bytes!("../../fixtures/mod.rs"),
    )?;
    fs::write(directory.join("runner.rs"), include_bytes!("mod.rs"))?;
    fs::write(
        directory.join("Cargo.lock"),
        include_bytes!("../../../../Cargo.lock"),
    )?;
    fs::write(
        directory.join("workspace.toml"),
        include_bytes!("../../../../Cargo.toml"),
    )?;
    fs::write(
        directory.join("package.toml"),
        include_bytes!("../../Cargo.toml"),
    )?;
    // Never replace an earlier run.
    let mut output = fs::File::create(directory.join("samples.jsonl"))?;
    let cases = fixtures::cases();
    for (case_id, case) in cases.iter().enumerate() {
        fs::write(
            directory.join(format!("{case_id}-input.json")),
            &case.input_json,
        )?;
        // Rotate codec ordering across cases to reduce a fixed first-codec bias.
        for offset in 0..case.lanes.len() {
            let lane = &case.lanes[(offset + case_id) % case.lanes.len()];
            assert_eq!((lane.encode)(), lane.bytes);
            assert!((lane.decode)(&lane.bytes));
            fs::write(
                directory.join(format!("{case_id}-{}.bin", lane.name)),
                &lane.bytes,
            )?;
            let mut record = |operation: &str,
                              op: &mut dyn FnMut()|
             -> Result<(), Box<dyn std::error::Error>> {
                let measured = measure(op, samples, millis);
                let throughput = measured
                    .get("median_ns")
                    .and_then(|v| v.as_f64())
                    .map(|ns| lane.bytes.len() as f64 * 1e9 / ns);
                writeln!(
                    output,
                    "{}",
                    json!({"case_id":case_id,"case":case.name,"codec":lane.name,"operation":operation,
                    "encoded_bytes":lane.bytes.len(),"fresh_output_capacity":lane.bytes.capacity(),"encoded_sha256":hash(&lane.bytes),"input_sha256":hash(&case.input_json),
                    "encoded_bytes_per_second":throughput,"measurement":measured})
                )?;
                Ok(())
            };
            record("encode-fresh", &mut || {
                black_box((lane.encode)());
            })?;
            record("decode-owned-or-labelled-borrowed", &mut || {
                assert!(black_box((lane.decode)(&lane.bytes)));
            })?;
            if let Some(reuse) = &lane.reuse {
                let mut buffer = lane.bytes.clone();
                reuse(&mut buffer);
                assert_eq!(buffer, lane.bytes);
                record("encode-reuse", &mut || {
                    reuse(&mut buffer);
                    black_box(&buffer);
                })?;
            }
            let mut bad_header = lane.bytes.clone();
            bad_header[0] ^= 0xff;
            assert!(!(lane.decode)(&bad_header));
            record("reject-bad-header", &mut || {
                assert!(!black_box((lane.decode)(&bad_header)));
            })?;
            // A truncated empty payload is still a malformed frame/header.
            let truncated = &lane.bytes[..lane.bytes.len() - 1];
            assert!(
                !(lane.decode)(truncated),
                "{} {} accepted truncation",
                case.name,
                lane.name
            );
            record("reject-truncated", &mut || {
                assert!(!black_box((lane.decode)(truncated)));
            })?;
        }
        eprintln!("{}/{} {}", case_id + 1, cases.len(), case.name);
    }
    let manifest = json!({"format":1,"profile":profile,"mode":if cfg!(feature="allocations") {"allocation-counts"} else if cfg!(feature="heap") {"peak-heap"} else {"native-time"},
        "samples":samples,"minimum_batch_ms":millis,"case_count":cases.len(),"compiler":command("rustc", &["-vV"]),
        "host":command("uname", &["-a"]),"git_head":command("git", &["rev-parse","HEAD"]),
        "git_status":command("git", &["status","--short"]),"lock_sha256":hash(include_bytes!("../../../../Cargo.lock")),
        "fixture_sha256":hash(include_bytes!("../../fixtures/mod.rs")),"bulk_fixture_sha256":hash(include_bytes!("../../fixtures/bulk/mod.rs")),"packed_i16_fixture_sha256":hash(include_bytes!("../../fixtures/packed_i16/mod.rs")),"toko_fixture_sha256":hash(include_bytes!("../../fixtures/toko/mod.rs")),"derive_fixture_sha256":hash(include_bytes!("../../fixtures/scale/mod.rs")),"runner_sha256":hash(include_bytes!("mod.rs")),
        "executable_sha256":hash(&fs::read(std::env::current_exe()?)?),
        "native_executable_bytes":fs::metadata(std::env::current_exe()?)?.len(),
        "cpu_info":fs::read_to_string("/proc/cpuinfo").ok(),
        "process_status":fs::read_to_string("/proc/self/status").ok(),
        "rustflags":std::env::var("RUSTFLAGS").ok(),"wire_limits":{"bytes":fixtures::LIMITS.bytes,"allocation":fixtures::LIMITS.allocation,"nodes":fixtures::LIMITS.nodes,"depth":fixtures::LIMITS.depth},
        "candid_limits":{"reply_bytes":fixtures::CANDID_LIMITS.reply_bytes,"decoding_quota":fixtures::CANDID_LIMITS.decoding_quota,"skipping_quota":fixtures::CANDID_LIMITS.skipping_quota,"type_entries":fixtures::CANDID_LIMITS.type_entries,"header_bytes":fixtures::CANDID_LIMITS.header_bytes},
        "limitations":["Serial warm-cache microbenchmarks; not application latency or cold-start cost", "Timing includes result destruction and loop/dynamic-call overhead", "Allocation counts are a separate instrumented build, not timing results or peak heap", "p95 describes batch means, not request tail latency", "Candid quotas and Wire budgets have different semantics", "Input setup, roundtrip equality and reporting are outside measurements"]});
    fs::write(
        directory.join("identity.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}
