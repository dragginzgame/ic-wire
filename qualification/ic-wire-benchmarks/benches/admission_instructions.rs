//! Explicit PocketIC admission study; does not alter the production limits.
mod admission;
use ic_testkit::pic::PocketIcStartupConfig;
use ic_testkit::pocket_ic::CanisterId;
use ic_testkit::pocket_ic::PocketIcBuilder;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf, time::Duration};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = clap::Command::new("admission-instructions")
        .about("Measure admission costs using a Wasm artifact and a new output directory")
        .arg(
            clap::Arg::new("wasm")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            clap::Arg::new("output")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();
    let wasm_path = arguments.get_one::<PathBuf>("wasm").unwrap();
    let output = arguments.get_one::<PathBuf>("output").unwrap();
    fs::create_dir(output)?;
    assert!(std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_none());
    let server =
        PathBuf::from(std::env::var_os("POCKET_IC_BIN").expect("prepared PocketIC server"));
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))?
        .build_instance(
            PocketIcBuilder::new()
                .with_max_request_time_ms(Some(300_000))
                .with_application_subnet(),
        )
        .await?;
    let pic = owned.instance();
    let wasm = fs::read(wasm_path)?;
    fs::write(
        output.join("identity.json"),
        serde_json::to_vec_pretty(&json!({
            "wasm_sha256": Sha256::digest(&wasm).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "server_sha256": Sha256::digest(fs::read(server)?).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "lock_sha256": Sha256::digest(include_bytes!("../../Cargo.lock")).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "fixture_sha256": Sha256::digest(include_bytes!("admission/mod.rs")).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "sample_caller":"unique 8-byte little-endian principal per query; caller not read by guest",
            "samples":3,"counter":"ic0.performance_counter(0)",
            "notes":"Warm decode, including destruction; setup/transport excluded. Budgets are illustrative, not equivalent to Candid quotas."
        }))?,
    )?;
    let canister = pic.create_canister().await;
    pic.add_cycles(canister, 100_000_000_000_000).await;
    pic.install_canister(canister, wasm, vec![], None).await;
    let mut file = fs::File::create(output.join("samples.jsonl"))?;
    let mut sample_caller = 0_u64;
    for (kind, name) in admission::KINDS.iter().enumerate() {
        for (index, length) in admission::LENGTHS.into_iter().enumerate() {
            for truncated in [false, true] {
                let bytes = admission::frame(kind, length, truncated);
                for (policy, label) in admission::POLICIES.iter().enumerate() {
                    let limits = admission::limits(policy);
                    let expected = admission::decode(kind, &bytes, limits);
                    if !truncated && length <= 64 {
                        assert_eq!(expected, 0);
                    }
                    if truncated {
                        assert_ne!(expected, 0);
                    }
                    let mut samples = Vec::new();
                    let mut sample_caller_ids = Vec::new();
                    for _ in 0..3 {
                        // Distinct callers prevent identical queries hitting the query cache.
                        sample_caller += 1;
                        sample_caller_ids.push(sample_caller);
                        let reply = pic
                            .query_call(
                                canister,
                                CanisterId::from_slice(&sample_caller.to_le_bytes()),
                                "benchmark",
                                vec![kind as u8, index as u8, policy as u8, u8::from(truncated)],
                            )
                            .await
                            .unwrap();
                        assert_eq!(reply.len(), 9);
                        assert_eq!(reply[8], expected);
                        samples.push(u64::from_le_bytes(reply[..8].try_into().unwrap()));
                    }
                    writeln!(
                        file,
                        "{}",
                        json!({"kind":name,"length":length,"truncated":truncated,
                        "policy":label,"bytes":limits.bytes,"allocation":limits.allocation,
                        "nodes":limits.nodes,"depth":limits.depth,"frame_bytes":bytes.len(),
                        "frame_sha256":Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                        "result":expected,"samples":samples,"sample_caller_ids":sample_caller_ids})
                    )?;
                }
            }
        }
    }
    pic.stop_canister(canister, None).await.unwrap();
    pic.delete_canister(canister, None).await.unwrap();
    owned.close().await;
    Ok(())
}
