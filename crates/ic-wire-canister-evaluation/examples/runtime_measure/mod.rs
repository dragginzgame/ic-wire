//! Installed production-runtime comparison, separate from historical private fixtures.
use candid::Principal;
use ic_wire::{Method, decode_response, encode_request, encode_response};
use ic_wire_canister_evaluation::runtime_benchmark::{Exchange, Packet};
use pocket_ic::PocketIcBuilder;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf, process::Command};
use wasmparser::{Parser, Payload};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn main() {
    let directory = PathBuf::from(env::args_os().nth(1).expect("artifact directory"));
    let server = PathBuf::from(env::var_os("POCKET_IC_BIN").expect("prepared PocketIC"));
    let version = Command::new(&server).arg("--version").output().unwrap();
    assert!(version.status.success());
    let pic = PocketIcBuilder::new()
        .with_server_binary(server.clone())
        .with_application_subnet()
        .build();
    let mut artifacts = Vec::new();
    for management in [false, true] {
        for candid in [true, false] {
            let name = format!(
                "{}-{}",
                if candid { "candid" } else { "wire" },
                if management { "management" } else { "internal" }
            );
            let wasm = fs::read(directory.join(format!("{name}.wasm"))).unwrap();
            let mut exports = Vec::new();
            let mut code_bytes = 0;
            for payload in Parser::new(0).parse_all(&wasm) {
                match payload.unwrap() {
                    Payload::CodeSectionStart { range, .. } => code_bytes = range.len(),
                    Payload::ExportSection(section) => {
                        for export in section {
                            let name = export.unwrap().name;
                            if name.starts_with("canister_") {
                                exports.push(name.to_owned());
                            }
                        }
                    }
                    _ => {}
                }
            }
            exports.sort();
            let mut expected_exports = vec![
                "canister_query measurements",
                "canister_update runtime_benchmark",
            ];
            if management {
                expected_exports.push("canister_update management_probe");
            }
            expected_exports.sort();
            assert_eq!(exports, expected_exports);
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
            for (case, blob, text, samples) in [
                ("empty collections", 0, 0, 0),
                ("blob 128", 128, 0, 0),
                ("blob 4096", 4096, 0, 0),
                ("UTF-8 text 256", 0, 256, 0),
                ("64 finite f64", 0, 0, 64),
                ("all bounds", 4096, 256, 64),
            ] {
                let value = Packet::fixture(blob, text, samples);
                let request = if candid {
                    candid::encode_one(value.candid(0)).unwrap()
                } else {
                    encode_request::<Exchange>(&value).unwrap().into_bytes()
                };
                let mut expected = value.clone();
                expected.revision += 1;
                let response = if candid {
                    candid::encode_one(expected.candid(1)).unwrap()
                } else {
                    encode_response::<Exchange>(&expected).unwrap().into_bytes()
                };
                let mut counts = Vec::new();
                for _ in 0..3 {
                    let reply = pic
                        .update_call(id, Principal::anonymous(), Exchange::NAME, request.clone())
                        .unwrap();
                    assert_eq!(reply, response);
                    if candid {
                        assert_eq!(
                            candid::decode_one::<
                                ic_wire_canister_evaluation::runtime_benchmark::CandidPacket,
                            >(&reply)
                            .unwrap(),
                            expected.candid(1)
                        );
                    } else {
                        assert_eq!(decode_response::<Exchange>(&reply).unwrap(), expected);
                    }
                    let raw = pic
                        .query_call(id, Principal::anonymous(), "measurements", vec![])
                        .unwrap();
                    assert_eq!(raw.len(), 24);
                    counts.push(
                        raw.as_chunks::<8>()
                            .0
                            .iter()
                            .map(|b| u64::from_le_bytes(*b))
                            .collect::<Vec<_>>(),
                    );
                }
                let mut trailing = request.clone();
                trailing.push(0);
                let wrong_identity = if candid {
                    let mut bad = value.candid(0);
                    bad.method += 1;
                    candid::encode_one(bad).unwrap()
                } else {
                    let mut bad = request.clone();
                    bad[2] ^= 1;
                    bad
                };
                for bad in [
                    vec![0; 5121],
                    request[..request.len() - 1].to_vec(),
                    trailing,
                    response.clone(),
                    wrong_identity,
                ] {
                    assert!(
                        pic.update_call(id, Principal::anonymous(), Exchange::NAME, bad)
                            .is_err()
                    );
                }
                rows.push(json!({"case": case, "blob_bytes": blob, "text_utf8_bytes": text, "f64_count": samples,
                    "request_bytes": request.len(), "response_bytes": response.len(), "request_sha256": hash(&request), "response_sha256": hash(&response),
                    "instruction_samples_decode_strict_handler_encode": counts, "rejected_calls": 5}));
            }
            artifacts.push(json!({"name": name, "wasm_bytes": wasm.len(), "code_section_payload_bytes": code_bytes, "wasm_sha256": hash(&wasm), "exports": exports, "management_call_verified": management, "rows": rows}));
        }
    }
    let source_paths = [
        "Cargo.toml",
        "Cargo.lock",
        "crates/ic-wire-canister-evaluation/Cargo.toml",
        "crates/ic-wire-canister-evaluation/src/lib.rs",
        "crates/ic-wire-canister-evaluation/src/authority/mod.rs",
        "crates/ic-wire-canister-evaluation/src/runtime_benchmark/mod.rs",
        "crates/ic-wire-canister-evaluation/examples/runtime_measure/mod.rs",
        "scripts/ci/evaluate-runtime.sh",
        "crates/ic-wire/src/lib.rs",
        "crates/ic-wire/src/bounded/mod.rs",
        "crates/ic-wire/src/contract/mod.rs",
        "crates/ic-wire/src/float/mod.rs",
        "crates/ic-wire/src/shape/mod.rs",
        "crates/ic-wire/src/schema/mod.rs",
        "crates/ic-wire-derive/src/lib.rs",
    ];
    let sources: serde_json::Map<String, serde_json::Value> = source_paths
        .into_iter()
        .map(|p| (p.into(), json!(hash(&fs::read(p).unwrap()))))
        .collect();
    let report = json!({
        "compiler_recorded_at_build": fs::read_to_string(directory.join("build-rustc.txt")).unwrap(),
        "flags_recorded_at_build": fs::read_to_string(directory.join("build-flags.txt")).unwrap(),
        "source_sha256": sources, "target": "wasm32-unknown-unknown",
        "profile": "canister-evaluation: release, opt-level=z, lto=true, codegen-units=1, panic=abort, strip=debuginfo",
        "pocket_ic_server_version": String::from_utf8(version.stdout).unwrap(), "pocket_ic_server_sha256": hash(&fs::read(server).unwrap()),
        "subnet": "application; PocketIC defaults",
        "scope": "IC performance_counter(0) deltas: strict decode including canonical re-encoding; handler; response encode. Counter overhead included. Argument copy, reply delivery and scheduling excluded. No native-time or compression measurements.",
        "candid_decoder": {"decoding_quota":100000, "skipping_quota":1000, "max_type_len":64, "max_header_len":1024, "max_frame_bytes":5120, "collection_bounds":"blob before copy; text/f64 count and finiteness after decode; canonical re-encode required"},
        "wire_limits": {"bytes":5120, "allocation":16384, "nodes":128, "depth":8},
        "artifacts": artifacts
    });
    fs::write(
        directory.join("report.pending.json"),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
}
