//! Explicit PocketIC admission study; does not alter the production limits.
mod admission;
use ic_platform::{candid::contract::Principal, pocket_ic::PocketIcBuilder};
use ic_testkit::pic::PocketIcStartupConfig;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let wasm_path = PathBuf::from(args.next().expect("Wasm artifact"));
    let output = PathBuf::from(args.next().expect("new output directory"));
    fs::create_dir(&output)?;
    assert!(std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_none());
    let server =
        PathBuf::from(std::env::var_os("POCKET_IC_BIN").expect("prepared PocketIC server"));
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))?
        .build_instance(PocketIcBuilder::new().with_application_subnet())?;
    let pic = owned.instance();
    let wasm = fs::read(wasm_path)?;
    fs::write(
        output.join("identity.json"),
        serde_json::to_vec_pretty(&json!({
            "wasm_sha256": format!("{:x}", Sha256::digest(&wasm)),
            "server_sha256": format!("{:x}", Sha256::digest(fs::read(server)?)),
            "lock_sha256": format!("{:x}", Sha256::digest(include_bytes!("../../../Cargo.lock"))),
            "fixture_sha256": format!("{:x}", Sha256::digest(include_bytes!("admission/mod.rs"))),
            "samples":3,"counter":"ic0.performance_counter(0)",
            "notes":"Warm decode, including destruction; setup/transport excluded. Budgets are illustrative, not equivalent to Candid quotas."
        }))?,
    )?;
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(canister, wasm, vec![], None);
    let mut file = fs::File::create(output.join("samples.jsonl"))?;
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
                    for _ in 0..3 {
                        let reply = pic
                            .query_call(
                                canister,
                                Principal::anonymous(),
                                "benchmark",
                                vec![kind as u8, index as u8, policy as u8, u8::from(truncated)],
                            )
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
                        "frame_sha256":format!("{:x}", Sha256::digest(&bytes)),
                        "result":expected,"samples":samples})
                    )?;
                }
            }
        }
    }
    pic.stop_canister(canister, None).unwrap();
    pic.delete_canister(canister, None).unwrap();
    Ok(())
}
