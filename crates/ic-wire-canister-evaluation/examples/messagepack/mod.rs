//! Interleaved native benchmark with calibrated batches and retained raw samples.
use ic_wire_canister_evaluation::messagepack::{CASES, run};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf, process::Command, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = PathBuf::from(env::args_os().nth(1).expect("fresh output directory"));
    let profile = env::args()
        .nth(2)
        .expect("profile label (evaluation or speed)");
    assert!(matches!(profile.as_str(), "evaluation" | "speed"));
    fs::create_dir(&directory)?;
    let mut rows = Vec::new();
    for (case, name) in CASES.iter().enumerate() {
        let mut repetitions = [1_u64; 4];
        let references = (0..4)
            .map(|lane| run(lane, case, 1, &mut || 0).bytes)
            .collect::<Vec<_>>();
        // Calibrate each lane to at least 3 ms for the faster operation.
        for (lane, repetitions) in repetitions.iter_mut().enumerate() {
            loop {
                let origin = Instant::now();
                let s = run(lane as u8, case, *repetitions, &mut || {
                    origin.elapsed().as_nanos() as u64
                });
                if s.encode.min(s.decode) >= 3_000_000 || *repetitions >= 1_048_576 {
                    break;
                }
                *repetitions *= 2;
            }
        }
        let mut samples = [const { Vec::<[u64; 2]>::new() }; 4];
        // Rotate lane order to avoid consistently giving one codec a warm/cold position.
        for batch in 0..15 {
            for position in 0..4 {
                let lane = (position + batch + case) % 4;
                let origin = Instant::now();
                let s = run(lane as u8, case, repetitions[lane], &mut || {
                    origin.elapsed().as_nanos() as u64
                });
                assert_eq!(s.bytes, references[lane]);
                samples[lane].push([s.encode, s.decode]);
            }
        }
        for lane in 0..4 {
            rows.push(json!({"case":name,"case_id":case,"lane":(["postcard","messagepack-array","messagepack-map","wire-native"][lane]),
                "encoded_bytes":references[lane].len(),"payload_sha256":format!("{:x}",Sha256::digest(&references[lane])),
                "repetitions":repetitions[lane],"samples_ns":samples[lane]}));
        }
        eprintln!("{name}: complete");
    }
    let compiler = Command::new("rustc").arg("-vV").output()?;
    assert!(compiler.status.success());
    fs::write(
        directory.join("report.json"),
        serde_json::to_vec_pretty(&json!({
            "profile":profile,"compiler":String::from_utf8(compiler.stdout)?, "architecture":env::consts::ARCH,"os":env::consts::OS,
            "rustflags":env::var("RUSTFLAGS").unwrap_or_default(),"encoded_rustflags":env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default(),
            "lock_sha256":format!("{:x}",Sha256::digest(include_bytes!("../../../../Cargo.lock"))),
            "fixture_sha256":format!("{:x}",Sha256::digest(include_bytes!("../../src/messagepack/mod.rs"))),
            "driver_sha256":format!("{:x}",Sha256::digest(include_bytes!("mod.rs"))),
            "samples":15,"columns":["encode","owned_decode"],"results":rows
        }))?,
    )?;
    Ok(())
}
