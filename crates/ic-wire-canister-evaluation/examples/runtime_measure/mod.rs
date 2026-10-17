//! Installed production-runtime comparison, separate from historical private fixtures.
use candid::Principal;
use ic_host_artifacts::wasm::{InspectionLimits, inspect};
use ic_testkit::pic::{
    CanisterInstallExt, InstallSpec, PocketIc, PocketIcBuilder, PocketIcStartupConfig,
};
use ic_wire::{Method, decode_response, encode_request, encode_response};
use ic_wire_canister_evaluation::runtime_benchmark::shapes::{Frame, Page, PageEcho, Scalar};
use ic_wire_canister_evaluation::runtime_benchmark::{Exchange, Packet};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn measure_shape(
    pic: &PocketIc,
    id: Principal,
    method: &str,
    request: Vec<u8>,
    response: Vec<u8>,
    wrong_identity: Vec<u8>,
) -> serde_json::Value {
    let mut samples = Vec::new();
    for _ in 0..3 {
        assert_eq!(
            pic.update_call(id, Principal::anonymous(), method, request.clone())
                .unwrap(),
            response
        );
        let raw = pic
            .query_call(id, Principal::anonymous(), "measurements", vec![])
            .unwrap();
        assert_eq!(raw.len(), 24);
        samples.push(
            raw.as_chunks::<8>()
                .0
                .iter()
                .map(|b| u64::from_le_bytes(*b))
                .collect::<Vec<_>>(),
        );
    }
    let mut trailing = request.clone();
    trailing.push(0);
    for invalid in [
        vec![0; 5121],
        request[..request.len() - 1].to_vec(),
        trailing,
        response.clone(),
        wrong_identity,
    ] {
        assert!(
            pic.update_call(id, Principal::anonymous(), method, invalid)
                .is_err()
        );
    }
    json!({"request_bytes": request.len(), "response_bytes": response.len(), "request_sha256": hash(&request), "response_sha256": hash(&response), "instruction_samples_decode_strict_handler_encode": samples, "rejected_calls": 5})
}
fn main() {
    let directory = PathBuf::from(env::args_os().nth(1).expect("artifact directory"));
    let server = PathBuf::from(env::var_os("POCKET_IC_BIN").expect("prepared PocketIC"));
    let owned = PocketIcStartupConfig::Spawn {
        server_binary: server.clone(),
        timeout: std::time::Duration::from_secs(60),
        server_hard_ttl: None,
        server_idle_ttl: None,
        server_output_files: None,
    }
    .build_instance(PocketIcBuilder::new().with_application_subnet())
    .unwrap();
    let pic = &owned.instance;
    let mut artifacts = Vec::new();
    for management in [false, true] {
        for candid in [true, false] {
            let name = format!(
                "{}-{}",
                if candid { "candid" } else { "wire" },
                if management { "management" } else { "internal" }
            );
            let wasm = fs::read(directory.join(format!("{name}.wasm"))).unwrap();
            let facts = inspect(
                &wasm,
                InspectionLimits {
                    module_bytes: 16 * 1024 * 1024,
                    sections: 128,
                    exports: 128,
                    custom_sections: 64,
                },
            )
            .unwrap();
            let code_bytes = facts.code_section_bytes;
            let exports: Vec<_> = facts
                .exports
                .keys()
                .filter(|name| name.starts_with("canister_"))
                .map(|name| (*name).to_owned())
                .collect();
            let mut expected_exports = vec![
                "canister_query measurements",
                "canister_update runtime_benchmark",
                "canister_update benchmark_scalar",
                "canister_update benchmark_page",
            ];
            if management {
                expected_exports.push("canister_update management_probe");
            }
            expected_exports.sort();
            assert_eq!(exports, expected_exports);
            let id = pic
                .try_create_and_install(InstallSpec {
                    wasm: wasm.clone(),
                    init_bytes: vec![],
                    cycles: 10_000_000_000_000,
                    install_sender: None,
                    label: None,
                })
                .unwrap();
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
                if candid {
                    assert_eq!(
                        candid::decode_one::<
                            ic_wire_canister_evaluation::runtime_benchmark::CandidPacket,
                        >(&response)
                        .unwrap(),
                        expected.candid(1)
                    );
                } else {
                    assert_eq!(decode_response::<Exchange>(&response).unwrap(), expected);
                }
                let wrong_identity = if candid {
                    let mut bad = value.candid(0);
                    bad.method += 1;
                    candid::encode_one(bad).unwrap()
                } else {
                    let mut bad = request.clone();
                    bad[2] ^= 1;
                    bad
                };
                let mut row =
                    measure_shape(pic, id, Exchange::NAME, request, response, wrong_identity);
                row["case"] = json!(case);
                row["blob_bytes"] = json!(blob);
                row["text_utf8_bytes"] = json!(text);
                row["f64_count"] = json!(samples);
                rows.push(row);
            }
            let mut scalar_rows = Vec::new();
            for value in [0_u64, 127, u64::MAX] {
                let reply = value.wrapping_add(1);
                let (request, response, wrong) = if candid {
                    (
                        candid::encode_one(Frame {
                            method: Scalar::ID,
                            direction: 0,
                            value,
                        })
                        .unwrap(),
                        candid::encode_one(Frame {
                            method: Scalar::ID,
                            direction: 1,
                            value: reply,
                        })
                        .unwrap(),
                        candid::encode_one(Frame {
                            method: Scalar::ID + 1,
                            direction: 0,
                            value,
                        })
                        .unwrap(),
                    )
                } else {
                    let request = encode_request::<Scalar>(&value).unwrap().into_bytes();
                    let mut wrong = request.clone();
                    wrong[2] ^= 1;
                    (
                        request,
                        encode_response::<Scalar>(&reply).unwrap().into_bytes(),
                        wrong,
                    )
                };
                let mut row = measure_shape(pic, id, Scalar::NAME, request, response, wrong);
                row["input_u64"] = json!(value.to_string());
                scalar_rows.push(row);
            }
            let mut page_rows = Vec::new();
            for count in [0, 1, 8] {
                let value = Page::fixture(count);
                let mut expected = value.clone();
                expected.next += 1;
                let (request, response, wrong) = if candid {
                    (
                        candid::encode_one(Frame {
                            method: PageEcho::ID,
                            direction: 0,
                            value: value.candid(),
                        })
                        .unwrap(),
                        candid::encode_one(Frame {
                            method: PageEcho::ID,
                            direction: 1,
                            value: expected.candid(),
                        })
                        .unwrap(),
                        candid::encode_one(Frame {
                            method: PageEcho::ID + 1,
                            direction: 0,
                            value: value.candid(),
                        })
                        .unwrap(),
                    )
                } else {
                    let request = encode_request::<PageEcho>(&value).unwrap().into_bytes();
                    let mut wrong = request.clone();
                    wrong[2] ^= 1;
                    (
                        request,
                        encode_response::<PageEcho>(&expected).unwrap().into_bytes(),
                        wrong,
                    )
                };
                let mut row = measure_shape(pic, id, PageEcho::NAME, request, response, wrong);
                row["row_count"] = json!(count);
                page_rows.push(row);
            }
            // The raw malformed producer deliberately bypasses bounded constructors.
            let page = Page::fixture(1);
            for outer in [true, false] {
                let invalid = if candid {
                    let mut value = page.candid();
                    if outer {
                        value.rows = vec![value.rows[0].clone(); 9];
                    } else {
                        value.rows[0] = vec![value.rows[0][0].clone(); 9];
                    }
                    candid::encode_one(Frame {
                        method: PageEcho::ID,
                        direction: 0,
                        value,
                    })
                    .unwrap()
                } else {
                    let mut rows: Vec<Vec<_>> = page
                        .rows
                        .as_slice()
                        .iter()
                        .map(|r| r.as_slice().to_vec())
                        .collect();
                    if outer {
                        rows = vec![rows[0].clone(); 9];
                    } else {
                        rows[0] = vec![rows[0][0].clone(); 9];
                    }
                    let mut bytes =
                        encode_request::<PageEcho>(&page).unwrap().as_bytes()[..11].to_vec();
                    bytes.extend(
                        postcard::to_allocvec(&(page.columns.as_slice(), rows, page.next)).unwrap(),
                    );
                    bytes
                };
                assert!(
                    pic.update_call(id, Principal::anonymous(), PageEcho::NAME, invalid)
                        .is_err()
                );
            }
            artifacts.push(json!({"name": name, "wasm_bytes": wasm.len(), "code_section_payload_bytes": code_bytes, "wasm_sha256": hash(&wasm), "exports": exports, "management_call_verified": management, "rows": rows, "scalar_rows": scalar_rows, "page_rows": page_rows, "page_bound_rejections": 2}));
        }
    }
    let builds: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("build-records.json")).unwrap()).unwrap();
    for artifact in &artifacts {
        let receipt = builds
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == artifact["name"])
            .unwrap();
        assert_eq!(receipt["wasm_sha256"], artifact["wasm_sha256"]);
    }
    let report = json!({
        "target": "wasm32-unknown-unknown",
        "profile": "canister-evaluation: release, opt-level=z, lto=true, codegen-units=1, panic=abort, strip=debuginfo",
        "subnet": "application; PocketIC defaults",
        "scope": "IC performance_counter(0) deltas: strict decode including canonical re-encoding; handler; response encode. Counter overhead included. Argument copy, reply delivery and scheduling excluded. No native-time or compression measurements.",
        "candid_decoder": {"decoding_quota":100000, "skipping_quota":1000, "max_type_len":64, "max_header_len":1024, "max_frame_bytes":5120, "collection_bounds":"blob before copy; text/f64 count and finiteness after decode; canonical re-encode required"},
        "wire_limits": {"bytes":5120, "allocation":16384, "nodes":128, "depth":8},
        "scalar_wire_limits": {"bytes":21, "allocation":21, "nodes":1, "depth":1},
        "page_wire_limits": {"bytes":5120, "allocation":16384, "nodes":256, "depth":8},
        "page_candid_bounds": "8 columns, 8 rows, 8 cells/row, 32 UTF-8/blob bytes, finite floats; checked after quota-limited decode",
        "artifacts": artifacts
    });
    fs::write(
        directory.join("report.pending.json"),
        format!("{}\n", serde_json::to_string_pretty(&report).unwrap()),
    )
    .unwrap();
}
