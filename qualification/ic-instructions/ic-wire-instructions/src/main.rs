//! Installed consumer counters; published Testkit owns server/instance lifetime.
use ic_testkit::{
    pic::PocketIcStartupConfig,
    pocket_ic::{CanisterId, PocketIcBuilder},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, time::Duration};
const OPERATIONS: [&str; 7] = [
    "encode_request",
    "decode_request",
    "encode_response",
    "decode_response",
    "bad_header",
    "truncated",
    "loop",
];
fn hash(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("provide artifact directory")?,
    );
    let server = PathBuf::from(
        std::env::var_os("POCKET_IC_BIN").ok_or("set explicit prepared POCKET_IC_BIN")?,
    );
    if std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_some() {
        return Err("owned prepared server required".into());
    }
    let server_hash = hash(&fs::read(&server)?);
    let version = std::process::Command::new(&server)
        .arg("--version")
        .output()?;
    if !version.status.success() {
        return Err("server version failed".into());
    }
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))?
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .await?;
    let pic = owned.instance();
    let result = async {
        let expected = [73,87,7,0,0,0,0,0,0,0,0,128,1,6,115,101,110,115,111,114,2,0,255,6,0,128,0,0,255,127,2,1,128,1,
            73,87,7,0,0,0,0,0,0,0,1,129,1,1];
        let mut rows = Vec::new();
        let mut caller = 0_u64;
        for profile in ["wire-size", "wire-speed"] {
            let wasm = fs::read(directory.join(format!("{profile}.wasm")))?;
            let artifact_hash = hash(&wasm);
            let canister = pic.create_canister().await;
            pic.add_cycles(canister, 100_000_000_000_000).await;
            pic.install_canister(canister, wasm, vec![], None).await;
            let mut samples = Vec::new();
            for _ in 0..3 {
                caller += 1;
                let reply = pic.query_call(canister, CanisterId::from_slice(&caller.to_le_bytes()), "measure", vec![]).await.map_err(|e| format!("query failed: {e:?}"))?;
                if reply.len() != 56 + expected.len() || reply[56..] != expected { return Err("counter frame oracle mismatch".into()); }
                let counts: Vec<u64> = reply[..56].as_chunks::<8>().0.iter().map(|bytes| u64::from_le_bytes(*bytes)).collect();
                samples.push(counts);
            }
            pic.stop_canister(canister, None).await.map_err(|e| format!("stop: {e:?}"))?;
            pic.delete_canister(canister, None).await.map_err(|e| format!("delete: {e:?}"))?;
            for (operation, name) in OPERATIONS.iter().enumerate() {
                let values: Vec<u64> = samples.iter().map(|s| s[operation]).collect();
                let mut sorted = values.clone(); sorted.sort_unstable();
                rows.push(json!({"profile":profile,"operation":name,"samples":values,"median_batch":sorted[1],"repetitions":8,"artifact_sha256":artifact_hash}));
            }
        }
        Ok::<_, Box<dyn std::error::Error>>(rows)
    }.await;
    owned.close().await;
    let rows = result?;
    let report = json!({"counter":"ic0.performance_counter(0)","server_version":String::from_utf8(version.stdout)?,"server_sha256":server_hash,"rows":rows,
        "host_lock_sha256":hash(include_bytes!("../../Cargo.lock")),"guest_lock_sha256":hash(include_bytes!("../../../Cargo.lock")),"driver_sha256":hash(include_bytes!("main.rs")),"guest_sha256":hash(include_bytes!("../../../ic-wire-consumer/src/ic.rs")),
        "consumer_sha256":hash(include_bytes!("../../../ic-wire-consumer/src/lib.rs")),"meter_sha256":hash(include_bytes!("../../../../crates/ic-wire/src/meter.rs")),
        "compiler":String::from_utf8(std::process::Command::new("rustc").arg("-Vv").output()?.stdout)?});
    fs::write(
        directory.join("instructions.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
