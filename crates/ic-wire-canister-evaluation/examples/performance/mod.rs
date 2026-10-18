//! Installed instruction counts and separately labelled native timings.
use ic_host_artifacts::wasm::{InspectionLimits, inspect};
use ic_platform::Principal;
use ic_testkit::pic::{CanisterInstallExt, InstallSpec, PocketIcBuilder, PocketIcStartupConfig};
use ic_wire_canister_evaluation::performance::{CASES, LANES, RECIPES, iterations, run_case};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

fn main() {
    let directory = PathBuf::from(env::args_os().nth(1).expect("retained artifact directory"));
    let owned = PocketIcStartupConfig::Spawn {
        server_binary: env::var_os("POCKET_IC_BIN")
            .expect("prepared server")
            .into(),
        timeout: Duration::from_secs(60),
        server_hard_ttl: None,
        server_idle_ttl: None,
        server_output_files: None,
    }
    .build_instance(PocketIcBuilder::new().with_application_subnet())
    .unwrap();
    let pic = &owned.instance;
    let caller = Principal::anonymous();
    let mut artifacts = Vec::new();
    let mut results = Vec::new();
    for ((feature, lanes), name) in RECIPES
        .iter()
        .zip(["wire", "postcard", "bitcode", "wincode"])
    {
        let wasm = fs::read(directory.join(format!("{name}.wasm"))).unwrap();
        let facts = inspect(
            &wasm,
            InspectionLimits {
                module_bytes: 64 * 1024 * 1024,
                sections: 128,
                exports: 128,
                custom_sections: 64,
            },
        )
        .unwrap();
        let exports: Vec<_> = facts
            .exports
            .keys()
            .copied()
            .filter(|s| s.starts_with("canister_"))
            .collect();
        assert_eq!(exports, ["canister_query performance"]);
        artifacts.push(json!({"name":name,"feature":feature,"wasm_bytes":wasm.len(),
            "code_section_bytes":facts.code_section_bytes,"sha256":format!("{:x}", Sha256::digest(&wasm))}));
        let id = pic
            .try_create_and_install(InstallSpec {
                wasm,
                init_bytes: vec![],
                cycles: 10_000_000_000_000,
                install_sender: None,
                label: None,
            })
            .unwrap();
        for &lane in *lanes {
            for (case, case_name) in CASES.iter().enumerate() {
                let reference = run_case(lane, case as u8, &mut || 0).bytes;
                // Logical equality is checked by every local/installed execution.
                let finite = run_case(0, case as u8, &mut || 0).bytes;
                if lane < 2 || lane == 6 {
                    assert_eq!(reference, finite);
                }
                if lane == 2 || lane == 5 {
                    assert_eq!(reference, finite[11..]);
                }
                let mut installed = Vec::new();
                let mut native = Vec::new();
                for _ in 0..7 {
                    let response = pic
                        .query_call(id, caller, "performance", vec![lane, case as u8])
                        .unwrap();
                    assert!(response.len() >= 24);
                    assert_eq!(&response[24..], reference);
                    let counters: Vec<_> = response[..24]
                        .as_chunks::<8>()
                        .0
                        .iter()
                        .map(|b| u64::from_le_bytes(*b))
                        .collect();
                    installed.push(counters);
                    let origin = Instant::now();
                    let sample = run_case(lane, case as u8, &mut || {
                        origin.elapsed().as_nanos().try_into().unwrap()
                    });
                    assert_eq!(sample.bytes, reference);
                    native.push([sample.encode, sample.decode, sample.sizing]);
                }
                let median = |column: usize| {
                    let mut values: Vec<_> = installed.iter().map(|v| v[column]).collect();
                    values.sort_unstable();
                    values[values.len() / 2] as f64 / iterations() as f64
                };
                results.push(json!({"case":case_name,"lane":LANES[lane as usize],
                    "encoded_bytes":reference.len(),"payload_sha256":format!("{:x}",Sha256::digest(&reference)),
                    "median_instructions":{"encode":median(0),"decode":median(1),"sizing":median(2)},
                    "installed_instruction_batches":installed,"native_nanosecond_batches":native}));
            }
        }
    }
    let version = Command::new("rustc").arg("-vV").output().unwrap();
    assert!(version.status.success());
    let report = json!({"scope":"valid-input microbenchmarks; raw controls do not implement Wire resource/canonicality contracts",
        "compiler":String::from_utf8(version.stdout).unwrap(),"target":"wasm32-unknown-unknown",
        "native_target":env::consts::ARCH,"profile":"canister-evaluation","iterations_per_batch":iterations(),
        "samples":7,"columns":["encode","owned_decode","native_sizing"],
        "measurement":"allocations, destruction, loop and counter overhead included; value/input setup, correctness assertions, argument copy and reply delivery excluded",
        "wire_limits":{"bytes":16384,"allocation":8388608,"nodes":65536,"depth":32},
        "configuration":{"postcard":"alloc; defaults disabled","bitcode":"default features; derived codec","wincode":"default configuration; derive; deserialize_exact"},
        "artifacts":artifacts,"results":results});
    fs::write(
        directory.join("report.pending.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
