//! Native timings for private raw-codec controls and the production codec.
use ic_wire_canister_evaluation::performance::{CASES, LANES, iterations, run_case};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf, process::Command, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = clap::Command::new("performance")
        .arg(
            clap::Arg::new("directory")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();
    let directory = arguments.get_one::<PathBuf>("directory").unwrap();
    // Refuse to overwrite a retained experiment.
    fs::create_dir(directory)?;
    let mut results = Vec::new();
    for (lane, lane_name) in LANES.iter().enumerate() {
        for (case, case_name) in CASES.iter().enumerate() {
            let reference = run_case(lane as u8, case as u8, &mut || 0).bytes;
            let finite = run_case(0, case as u8, &mut || 0).bytes;
            if lane < 2 || lane == 6 {
                assert_eq!(reference, finite);
            }
            if lane == 2 || lane == 5 {
                assert_eq!(reference, finite[11..]);
            }
            let mut native = Vec::new();
            for _ in 0..7 {
                let origin = Instant::now();
                let sample = run_case(lane as u8, case as u8, &mut || {
                    origin.elapsed().as_nanos().try_into().unwrap()
                });
                assert_eq!(sample.bytes, reference);
                native.push([sample.encode, sample.decode, sample.sizing]);
            }
            results.push(json!({"case":case_name,"lane":lane_name,
                "encoded_bytes":reference.len(),"payload_sha256":format!("{:x}",Sha256::digest(&reference)),
                "native_nanosecond_batches":native}));
        }
    }
    let version = Command::new("rustc").arg("-vV").output()?;
    assert!(version.status.success());
    let report = json!({"scope":"valid-input native microbenchmarks; raw controls do not implement Wire resource/canonicality contracts",
        "compiler":String::from_utf8(version.stdout).unwrap(),
        "native_target":{"architecture":env::consts::ARCH,"os":env::consts::OS},
        "profile":"evaluation","iterations_per_batch":iterations(),"samples":7,
        "rustflags":env::var("RUSTFLAGS").unwrap_or_default(),
        "encoded_rustflags":env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default(),
        "lock_sha256":format!("{:x}",Sha256::digest(include_bytes!("../../../../Cargo.lock"))),
        "cases_sha256":format!("{:x}",Sha256::digest(include_bytes!("../../src/performance/mod.rs"))),
        "driver_sha256":format!("{:x}",Sha256::digest(include_bytes!("mod.rs"))),
        "columns":["encode","owned_decode","native_sizing"],
        "measurement":"native nanoseconds only; allocations, destruction, loop and clock overhead included; value/input setup and correctness assertions excluded",
        "wire_limits":{"bytes":16384,"allocation":8388608,"nodes":65536,"depth":32},
        "configuration":{"postcard":"alloc; defaults disabled","bitcode":"default features; derived codec","wincode":"default configuration; derive; deserialize_exact"},
        "results":results});
    fs::write(
        directory.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )?;
    Ok(())
}
