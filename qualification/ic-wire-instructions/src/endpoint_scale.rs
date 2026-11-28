//! Query the metered copy of a generated canister with byte-exact frame oracles.
use ic_testkit::pocket_ic::{CanisterId, nonblocking::PocketIc};
use serde_json::{Value, json};
use std::{error::Error, fs, path::Path};

pub(super) async fn run(pic: &PocketIc, directory: &Path) -> Result<Vec<Value>, Box<dyn Error>> {
    let wasm = fs::read(directory.join("metered.wasm"))?;
    let artifact = super::hash(&wasm);
    let canister = pic.create_canister().await;
    pic.add_cycles(canister, 100_000_000_000_000).await;
    pic.install_canister(canister, wasm, vec![], None).await;
    let cases: Vec<Value> = serde_json::from_slice(&fs::read(directory.join("cases.json"))?)?;
    let mut rows = Vec::new();
    for case in cases {
        let name = case["name"].as_str().ok_or("missing method")?;
        let input: Vec<u8> = serde_json::from_value(case["request"].clone())?;
        let expected: Vec<u8> = serde_json::from_value(case["reply"].clone())?;
        for sample in 1..=3 {
            let reply = pic
                .query_call(
                    canister,
                    CanisterId::from_slice(&[sample]),
                    name,
                    input.clone(),
                )
                .await
                .map_err(|e| format!("query {name}: {e:?}"))?;
            let (counter, frame) = reply.split_at_checked(8).ok_or("missing counter")?;
            if frame != expected {
                return Err(format!("unexpected frame from {name}").into());
            }
            rows.push(json!({
                "method": name, "case": case["kind"], "sample":sample,
                "instructions": u64::from_le_bytes(counter.try_into()?),
                "request_bytes": input.len(), "reply_bytes": frame.len(),
                "artifact_sha256": artifact
            }));
        }
    }
    Ok(rows)
}
