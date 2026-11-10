//! IC instruction counter measurements using an explicitly prepared PocketIC server.
#[expect(
    dead_code,
    reason = "This driver consumes only the selected operations from the shared corpus."
)]
mod fixtures;
#[expect(
    dead_code,
    reason = "The host names guest counters without matching operation indices."
)]
mod operations;
#[cfg(all(feature = "coverage-shapes", feature = "wire"))]
mod unsigned_magnitude;
use ic_platform_types::Principal;
use ic_testkit::pic::PocketIcStartupConfig;
use ic_testkit::pocket_ic::PocketIcBuilder;
use operations::{COUNTS_BYTES, FRAME_OFFSET, OPERATIONS};
use serde_json::json;
mod evidence;
use evidence::hash;
use std::{fs, io::Write, path::PathBuf, process::Command, time::Duration};
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = clap::Command::new("codec-instructions")
        .about("Measure completed native/Wasm evidence using a prepared PocketIC server")
        .arg(
            clap::Arg::new("directory")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();
    let root = arguments.get_one::<PathBuf>("directory").unwrap();
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
    let compiler = Command::new("rustc").arg("-vV").output()?;
    assert!(compiler.status.success());
    let compiler = String::from_utf8(compiler.stdout)?;
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))?
        .build_instance(PocketIcBuilder::new(Duration::from_secs(300)).with_application_subnet())
        .await?;
    let pic = owned.instance();
    let cases = fixtures::cases();
    let samples = 3;
    let mut sample_caller = 0_u64;
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
            let canister = pic.create_canister().await?;
            pic.add_cycles(canister, 100_000_000_000_000).await?;
            pic.install_canister(canister, wasm, vec![], None).await?;
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
                    let mut counts = vec![Vec::new(); OPERATIONS.len()];
                    let mut sample_caller_ids = Vec::with_capacity(samples);
                    for _ in 0..samples {
                        // An identical query can hit PocketIC's query cache. Give
                        // every sample a distinct caller so it executes the guest;
                        // the measured fixture does not read the caller identity.
                        sample_caller += 1;
                        sample_caller_ids.push(sample_caller);
                        let caller = Principal::from_slice(&sample_caller.to_le_bytes());
                        let args = [case_id as u32, lane_id as u32, repetitions]
                            .into_iter()
                            .flat_map(u32::to_le_bytes)
                            .collect();
                        let reply = pic
                            .query_call(canister, caller, "benchmark", args)
                            .await
                            .unwrap();
                        assert!(reply.len() >= FRAME_OFFSET);
                        let length = u64::from_le_bytes(
                            reply[COUNTS_BYTES..FRAME_OFFSET].try_into().unwrap(),
                        ) as usize;
                        assert_eq!(length, lane.bytes.len());
                        assert_eq!(
                            &reply[FRAME_OFFSET..],
                            lane.bytes,
                            "{profile}/{codec}/{}",
                            case.name
                        );
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
                            OPERATIONS[op],
                            lane.bytes.len(),
                            sorted[samples / 2] as f64 / f64::from(repetitions)
                        )?;
                        writeln!(
                            results,
                            "{}",
                            json!({"profile":profile,"case_id":case_id,"case":case.name,"codec":lane.name,"operation":OPERATIONS[op],
                            "encoded_bytes":lane.bytes.len(),"payload_sha256":hash(&lane.bytes),"input_sha256":hash(&case.input_json),"repetitions":repetitions,
                            "samples_instructions":values,"sample_caller_ids":sample_caller_ids,"median_instructions_per_op":sorted[samples/2] as f64 / f64::from(repetitions)})
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
            pic.stop_canister(canister, None).await.unwrap();
            pic.delete_canister(canister, None).await.unwrap();
        }
    }
    fs::write(
        output.join("identity.json"),
        serde_json::to_vec_pretty(&evidence::with_fixtures(
            json!({"format":1,"counter":"ic0.performance_counter(0)","samples":samples,"small_repetitions":4,"large_repetitions":1,
        "sample_caller":"unique 8-byte little-endian principal per query; caller not read by guest",
        "server_version":String::from_utf8(version.stdout)?,"server_sha256":hash(&fs::read(server)?),"artifacts":artifacts,
        "driver_sha256":hash(include_bytes!("instructions.rs")),"wasm_driver_sha256":hash(include_bytes!("wasm.rs")),
        "compiler":compiler,"workspace_manifest_sha256":hash(include_bytes!("../../Cargo.toml")),
        "build_environment":std::env::vars().filter(|(key,_)| key.starts_with("CARGO_PROFILE_") || matches!(key.as_str(),"RUSTFLAGS"|"CARGO_ENCODED_RUSTFLAGS"|"RUSTC_WRAPPER"|"RUSTC_WORKSPACE_WRAPPER")).collect::<std::collections::BTreeMap<_,_>>(),

        "native_encoder_sha256":hash(include_bytes!("../../../crates/ic-wire/src/native/encode/mod.rs")),"native_decoder_sha256":hash(include_bytes!("../../../crates/ic-wire/src/native/decode/mod.rs")),"native_admission_sha256":hash(include_bytes!("../../../crates/ic-wire/src/native/mod.rs")),
        "contract_sha256":hash(include_bytes!("../../../crates/ic-wire/src/contract/mod.rs")),
        "native_parser":"upstream-postcard-locked",
        "lock_sha256":hash(include_bytes!("../../Cargo.lock")),"notes":"Warm operations; result destruction and loop overhead included. Loop control reported, not subtracted. Setup/reply transport excluded. Exact frames checked against native fixtures."}),
        ))?,
    )?;
    owned.close().await?;
    Ok(())
}
