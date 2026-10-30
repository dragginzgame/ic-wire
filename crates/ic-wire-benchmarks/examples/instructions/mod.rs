//! IC instruction counter measurements using an explicitly prepared PocketIC server.
use ic_platform::{candid::contract::Principal, pocket_ic::PocketIcBuilder};
use ic_testkit::pic::PocketIcStartupConfig;
use ic_wire_benchmarks::fixtures;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf, process::Command, time::Duration};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("completed native/Wasm evidence directory"),
    );
    let output = root.join("instructions");
    fs::create_dir(&output)?;
    // Testkit admits explicit binary compatibility and owns process lifetime.
    // No implicit download or fallback server selection is permitted.
    let server = PathBuf::from(
        std::env::var_os("POCKET_IC_BIN").expect("set POCKET_IC_BIN to a prepared server"),
    );
    assert!(
        std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_none(),
        "use an owned server for reproducible identity"
    );
    let version = Command::new(&server).arg("--version").output()?;
    assert!(version.status.success());
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))?
        .build_instance(PocketIcBuilder::new().with_application_subnet())?;
    let pic = owned.instance();
    let cases = fixtures::cases();
    let operations = [
        "encode-fresh",
        "decode-owned-or-labelled-borrowed",
        "encode-reuse",
        "reject-bad-header",
        "reject-truncated",
        "loop-control",
    ];
    let samples = 3;
    let mut results = fs::File::create(output.join("samples.jsonl"))?;
    let mut summary = fs::File::create(output.join("summary.csv"))?;
    writeln!(
        summary,
        "profile,case,codec,operation,encoded_bytes,median_instructions_per_op"
    )?;
    let mut artifacts = Vec::new();
    for profile in ["evaluation", "benchmark-speed"] {
        for codec in ["wire", "candid"] {
            let wasm = fs::read(root.join(format!("{profile}-{codec}.wasm")))?;
            artifacts.push(json!({"profile":profile,"codec":codec,"sha256":hash(&wasm)}));
            let canister = pic.create_canister();
            pic.add_cycles(canister, 100_000_000_000_000);
            pic.install_canister(canister, wasm, vec![], None);
            for (case_id, case) in cases.iter().enumerate() {
                let lanes = case.lanes.iter().filter(|lane| {
                    if codec == "wire" {
                        lane.name.starts_with("wire-")
                    } else {
                        lane.name == "candid"
                    }
                });
                for (lane_id, lane) in lanes.enumerate() {
                    let repetitions = if lane.bytes.len() > 65_536 {
                        1_u32
                    } else {
                        4_u32
                    };
                    let mut counts = vec![Vec::new(); 6];
                    for _ in 0..samples {
                        let args = [case_id as u32, lane_id as u32, repetitions]
                            .into_iter()
                            .flat_map(u32::to_le_bytes)
                            .collect();
                        let reply = pic
                            .query_call(canister, Principal::anonymous(), "benchmark", args)
                            .unwrap();
                        assert!(reply.len() >= 56);
                        let length = u64::from_le_bytes(reply[48..56].try_into().unwrap()) as usize;
                        assert_eq!(length, lane.bytes.len());
                        assert_eq!(&reply[56..], lane.bytes, "{profile}/{codec}/{}", case.name);
                        for (op, values) in counts.iter_mut().enumerate() {
                            let count =
                                u64::from_le_bytes(reply[op * 8..op * 8 + 8].try_into().unwrap());
                            if count != u64::MAX {
                                values.push(count);
                            }
                        }
                    }
                    for (op, values) in counts.iter().enumerate() {
                        if values.is_empty() {
                            continue;
                        }
                        assert_eq!(values.len(), samples);
                        let mut sorted = values.clone();
                        sorted.sort_unstable();
                        writeln!(
                            summary,
                            "{profile},{},{},{},{},{}",
                            case.name,
                            lane.name,
                            operations[op],
                            lane.bytes.len(),
                            sorted[samples / 2] as f64 / f64::from(repetitions)
                        )?;
                        writeln!(
                            results,
                            "{}",
                            json!({"profile":profile,"case_id":case_id,"case":case.name,"codec":lane.name,"operation":operations[op],
                            "encoded_bytes":lane.bytes.len(),"payload_sha256":hash(&lane.bytes),"input_sha256":hash(&case.input_json),"repetitions":repetitions,
                            "samples_instructions":values,"median_instructions_per_op":sorted[samples/2] as f64 / f64::from(repetitions)})
                        )?;
                    }
                }
                eprintln!(
                    "{profile}/{codec} {}/{} {}",
                    case_id + 1,
                    cases.len(),
                    case.name
                );
            }
            pic.stop_canister(canister, None).unwrap();
            pic.delete_canister(canister, None).unwrap();
        }
    }
    fs::write(
        output.join("identity.json"),
        serde_json::to_vec_pretty(
            &json!({"format":1,"counter":"ic0.performance_counter(0)","samples":samples,"small_repetitions":4,"large_repetitions":1,
        "server_version":String::from_utf8(version.stdout)?,"server_sha256":hash(&fs::read(server)?),"artifacts":artifacts,
        "fixture_sha256":hash(include_bytes!("../../src/fixtures/mod.rs")),"hotspots_fixture_sha256":hash(include_bytes!("../../src/fixtures/hotspots/mod.rs")),"bulk_fixture_sha256":hash(include_bytes!("../../src/fixtures/bulk/mod.rs")),"packed_i16_fixture_sha256":hash(include_bytes!("../../src/fixtures/packed_i16/mod.rs")),"toko_fixture_sha256":hash(include_bytes!("../../src/fixtures/toko/mod.rs")),"derive_fixture_sha256":hash(include_bytes!("../../src/fixtures/scale/mod.rs")),"driver_sha256":hash(include_bytes!("mod.rs")),"wasm_driver_sha256":hash(include_bytes!("../wasm/mod.rs")),
        "lock_sha256":hash(include_bytes!("../../../../Cargo.lock")),"notes":"Warm operations; result destruction and loop overhead included. Loop control reported, not subtracted. Setup/reply transport excluded. Exact frames checked against native fixtures."}),
        )?,
    )?;
    Ok(())
}
