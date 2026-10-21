//! Native allocation/admission experiment with calibrated batches and raw samples.
use ic_wire_qualification::allocation::{CASE_COUNT, name, run_with_scan};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = clap::Command::new("allocation")
        .arg(
            clap::Arg::new("directory")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            clap::Arg::new("profile")
                .required(true)
                .value_parser(["evaluation", "speed"]),
        )
        .arg(
            clap::Arg::new("codec-only")
                .long("codec-only")
                .help("Exclude the application payload checksum from decoding time")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();
    let scan = !arguments.get_flag("codec-only");
    let output = arguments.get_one::<PathBuf>("directory").unwrap();
    let profile = arguments.get_one::<String>("profile").unwrap();
    fs::create_dir(output)?;
    let mut results = Vec::new();
    let mut repetitions = vec![[1_u64; 2]; CASE_COUNT];
    let reference = (0..CASE_COUNT)
        .map(|case| run_with_scan(case, [0; 2], &mut || 0, scan))
        .collect::<Vec<_>>();
    for (case, reps) in repetitions.iter_mut().enumerate() {
        loop {
            let origin = Instant::now();
            let s = run_with_scan(
                case,
                *reps,
                &mut || origin.elapsed().as_nanos() as u64,
                scan,
            );
            let elapsed = [s.encode, s.decode];
            let mut ready = true;
            for operation in 0..2 {
                if elapsed[operation] < 2_000_000 && reps[operation] < 1_048_576 {
                    reps[operation] *= 2;
                    ready = false;
                }
            }
            if ready {
                break;
            }
        }
    }
    let mut samples = vec![Vec::new(); CASE_COUNT];
    for batch in 0..11 {
        for position in 0..CASE_COUNT {
            let case = (position + batch * 7) % CASE_COUNT;
            let origin = Instant::now();
            let s = run_with_scan(
                case,
                repetitions[case],
                &mut || origin.elapsed().as_nanos() as u64,
                scan,
            );
            assert_eq!(s.bytes, reference[case].bytes);
            assert_eq!(s.capacity, reference[case].capacity);
            samples[case].push([s.encode, s.decode]);
        }
        eprintln!("batch {} complete", batch + 1);
    }
    for case in 0..CASE_COUNT {
        results.push(json!({"case_id":case,"case":name(case),"encoded_bytes":reference[case].bytes.len(),"output_capacity":reference[case].capacity,
            "payload_sha256":format!("{:x}",Sha256::digest(&reference[case].bytes)),"encode_repetitions":repetitions[case][0],"decode_repetitions":repetitions[case][1],"samples_ns":samples[case]}));
    }
    let mut storage = Vec::new();
    let mut history = Vec::new();
    let budget = ic_wire::Limits {
        bytes: 2 * 1024 * 1024,
        allocation: 64 * 1024 * 1024,
        nodes: 262_144,
        depth: 32,
    };
    for size in [0, 1048576, 32, 0, 4096] {
        let value = serde_bytes::ByteBuf::from(vec![0; size]);
        ic_wire::native::encode_request_into(3, &value, budget, &mut storage).unwrap();
        history.push(json!({"input_bytes":size,"output_bytes":storage.len(),"retained_capacity":storage.capacity()}));
    }
    let rustc = Command::new("rustc").arg("-vV").output()?;
    assert!(rustc.status.success());
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"format":2,"decode_work":if scan { "codec-and-checksum" } else { "codec-only" },"reuse_capacity_history":history,"profile":profile,"compiler":String::from_utf8(rustc.stdout)?,
        "lock_sha256":format!("{:x}",Sha256::digest(include_bytes!("../../../../Cargo.lock"))),
        "fixture_sha256":format!("{:x}",Sha256::digest(include_bytes!("../../src/allocation/mod.rs"))),
        "driver_sha256":format!("{:x}",Sha256::digest(include_bytes!("mod.rs"))),"results":results}),
        )?,
    )?;
    Ok(())
}
