//! Install matched artifacts in PocketIC and report separate size/instruction evidence.
use candid::Principal;
use ic_wire_canister_evaluation::{Blob, CONTRACT, MAX_WIRE, Message};
use pocket_ic::PocketIcBuilder;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf, process::Command};
use wasmparser::{Parser, Payload};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn encode(candid: bool, value: &Message) -> Vec<u8> {
    if candid {
        candid::encode_one(value).unwrap()
    } else {
        postcard::to_allocvec(value).unwrap()
    }
}
fn decode(candid: bool, bytes: &[u8]) -> Message {
    if candid {
        candid::decode_one(bytes).unwrap()
    } else {
        let (value, rest) = postcard::take_from_bytes(bytes).unwrap();
        assert!(rest.is_empty());
        value
    }
}
fn main() {
    let directory = PathBuf::from(env::args_os().nth(1).expect("artifact directory argument"));
    let server = PathBuf::from(
        env::var_os("POCKET_IC_BIN").expect("set POCKET_IC_BIN to a prepared PocketIC server"),
    );
    let server_version = Command::new(&server).arg("--version").output().unwrap();
    assert!(server_version.status.success());
    let pic = PocketIcBuilder::new()
        .with_server_binary(server.clone())
        .with_application_subnet()
        .build();
    let mut artifacts = Vec::new();
    for management in [false, true] {
        for candid in [true, false] {
            let name = format!(
                "{}-{}",
                if candid { "candid" } else { "postcard" },
                if management { "management" } else { "internal" }
            );
            let wasm = fs::read(directory.join(format!("{name}.wasm"))).unwrap();
            let code_bytes: usize = Parser::new(0)
                .parse_all(&wasm)
                .map(|payload| match payload.unwrap() {
                    Payload::CodeSectionStart { range, .. } => range.len(),
                    _ => 0,
                })
                .sum();
            let id = pic.create_canister();
            pic.add_cycles(id, 10_000_000_000_000);
            pic.install_canister(id, wasm.clone(), vec![], None);
            if management {
                assert_eq!(
                    pic.update_call(id, Principal::anonymous(), "management_probe", vec![])
                        .unwrap(),
                    b"ok"
                );
            }
            let mut rows = Vec::new();
            for size in [0, 128, 4096] {
                let payload: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
                let value = Message {
                    contract: CONTRACT,
                    direction: 0,
                    account: 7,
                    revision: 42,
                    payload: Blob::new(&payload).unwrap(),
                };
                let request = encode(candid, &value);
                let mut expected = value.clone();
                expected.direction = 1;
                expected.revision = 43;
                let mut samples = Vec::new();
                let mut response_bytes = None;
                for _ in 0..3 {
                    // The exact retained request is reused. No state-changing business
                    // operation is modeled, so this is not a deduplication claim.
                    let reply = pic
                        .update_call(id, Principal::anonymous(), "evaluate", request.clone())
                        .unwrap();
                    assert_eq!(decode(candid, &reply), expected);
                    assert_eq!(reply, encode(candid, &expected));
                    if let Some(previous) = &response_bytes {
                        assert_eq!(&reply, previous);
                    }
                    response_bytes = Some(reply);
                    let counters = pic
                        .query_call(id, Principal::anonymous(), "measurements", vec![])
                        .unwrap();
                    assert_eq!(counters.len(), 24);
                    let sample: Vec<u64> = counters
                        .as_chunks::<8>()
                        .0
                        .iter()
                        .map(|chunk| u64::from_le_bytes(*chunk))
                        .collect();
                    samples.push(sample);
                }
                let reply = response_bytes.unwrap();
                rows.push(json!({ "blob_bytes": size, "request_bytes": request.len(), "request_sha256": hash(&request), "response_bytes": reply.len(), "response_sha256": hash(&reply), "instruction_samples_decode_strict_handler_encode": samples }));
                let mut wrong_identity = value.clone();
                wrong_identity.contract += 1;
                let mut wrong_direction = value.clone();
                wrong_direction.direction = 1;
                let mut trailing = request.clone();
                trailing.push(0);
                for invalid in [
                    vec![0; MAX_WIRE + 1],
                    request[..request.len() - 1].to_vec(),
                    trailing,
                    encode(candid, &wrong_identity),
                    encode(candid, &wrong_direction),
                ] {
                    assert!(
                        pic.update_call(id, Principal::anonymous(), "evaluate", invalid)
                            .is_err(),
                        "malformed input accepted by {name}"
                    );
                }
            }
            artifacts.push(json!({ "name": name, "wasm_bytes": wasm.len(), "code_section_payload_bytes": code_bytes, "wasm_sha256": hash(&wasm), "management_call_verified": management, "rows": rows }));
        }
    }
    let rustc = Command::new("rustc").arg("-vV").output().unwrap();
    assert!(rustc.status.success());
    let report = json!({
        "compiler_observed_at_execution": String::from_utf8(rustc.stdout).unwrap(),
        "compiler_recorded_at_build": fs::read_to_string(directory.join("build-rustc.txt")).unwrap(),
        "flags_recorded_at_build": fs::read_to_string(directory.join("build-flags.txt")).unwrap(),
        "workspace_manifest_sha256": hash(include_bytes!("../../../../Cargo.toml")),
        "package_manifest_sha256": hash(include_bytes!("../../Cargo.toml")),
        "build_script_sha256": hash(include_bytes!("../../../../scripts/ci/evaluate-canisters.sh")),
        "target": "wasm32-unknown-unknown", "profile": "canister-evaluation: release, opt-level=z, lto=true, codegen-units=1, panic=abort, strip=debuginfo",
        "pocket_ic_server_version": String::from_utf8(server_version.stdout).unwrap(), "pocket_ic_server_sha256": hash(&fs::read(server).unwrap()),
        "subnet": "application; PocketIC defaults", "lock_sha256": hash(include_bytes!("../../../../Cargo.lock")),
        "canister_source_sha256": hash(include_bytes!("../../src/lib.rs")), "runner_source_sha256": hash(include_bytes!("mod.rs")),
        "scope": "IC performance_counter(0) deltas: strict decode includes canonical re-encoding; handler; encode. Counter overhead included; no whole-call, native timing or compressed-size claims.",
        "artifacts": artifacts
    });
    fs::write(
        directory.join("report.pending.json"),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
}
