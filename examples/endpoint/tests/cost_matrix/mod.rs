//! Complete exported-endpoint costs; no instrumentation in the measured canister.
use ic_platform::{
    pocket_ic::{PocketIcBuilder, RejectCode},
    principal::Principal,
};
use ic_testkit::pic::PocketIcStartupConfig;
use ic_wire::{ByteBuf, Limits, native};
use ic_wire_endpoint_fixture::{
    DEFAULT_BLOB, DEFAULT_MUTATE, LIMITS, NAMED, RESPONSE_ONLY, Request,
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, io::Write, path::PathBuf, process::Command, time::Duration};

#[derive(Clone)]
enum Expected {
    Reply(Vec<u8>),
    Count(u64),
    Trap,
    Reject,
}
#[derive(Clone)]
struct Case {
    method: &'static str,
    mode: &'static str,
    label: String,
    id: u64,
    limits: Limits,
    frame: Vec<u8>,
    expected: Expected,
    mutation: u64,
}
fn case<T: Serialize, R: Serialize>(
    method: &'static str,
    mode: &'static str,
    id: u64,
    limits: Limits,
    label: &str,
    request: &T,
    reply: &R,
) -> Case {
    Case {
        method,
        mode,
        id,
        limits,
        label: label.into(),
        frame: native::encode_request(id, request, limits).unwrap(),
        expected: Expected::Reply(native::encode_response(id, reply, limits).unwrap()),
        mutation: 0,
    }
}
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (method, id) in [("echo", 7), ("borrowed_echo", 17)] {
        for size in [0, 32] {
            let value = Request {
                value: 300,
                bytes: ic_wire::Bytes::new(vec![42; size]).unwrap(),
            };
            cases.push(case(
                method,
                "query",
                id,
                LIMITS,
                &format!("blob-{size}"),
                &value,
                &value,
            ));
        }
    }
    let mut count = case("count", "query", 8, LIMITS, "unit", &(), &0u64);
    count.expected = Expected::Count(0);
    cases.push(count);
    cases.push(case("unit-call", "update", 10, LIMITS, "unit", &(), &()));
    for value in [0, u64::MAX] {
        cases.push(case(
            "async_query",
            "query",
            11,
            LIMITS,
            &format!("value-{value}"),
            &value,
            &(value, true),
        ));
        cases.push(case(
            "stable-name",
            "query",
            NAMED.id,
            NAMED.request,
            &format!("value-{value}"),
            &value,
            &value,
        ));
    }
    cases.push(case(
        "response_only",
        "query",
        RESPONSE_ONLY.id,
        RESPONSE_ONLY.request,
        "scalar",
        &42u64,
        &42u64,
    ));
    let mut composite = case(
        "composite",
        "composite_query",
        12,
        LIMITS,
        "self-call",
        &(),
        &0u64,
    );
    composite.expected = Expected::Count(0);
    cases.push(composite);
    for (method, id) in [
        ("mutate", 9),
        ("async_mutate", 13),
        ("async_ready", 14),
        ("borrowed_mutate", 18),
    ] {
        for size in [0u32, 32, 116, 117] {
            let mut row = case(
                method,
                "update",
                id,
                LIMITS,
                &format!("reply-{size}"),
                &size,
                &ByteBuf::new(),
            );
            if id == 18 {
                row.frame = native::encode_request(
                    id,
                    &Request {
                        value: u64::from(size),
                        bytes: ic_wire::Bytes::new(vec![42; 32]).unwrap(),
                    },
                    LIMITS,
                )
                .unwrap();
            }
            if size == 117 {
                row.expected = Expected::Trap;
                row.mutation = if id == 13 { 10 } else { 0 };
            } else {
                row.expected = Expected::Reply(
                    native::encode_response(id, &ByteBuf::from(vec![42; size as usize]), LIMITS)
                        .unwrap(),
                );
                row.mutation = if id == 13 { 110 } else { 1 };
            }
            cases.push(row);
        }
    }
    for length in [0, 1, 2] {
        let mut row = case(
            "nodes",
            "query",
            15,
            LIMITS,
            &format!("nodes-{length}"),
            &vec![1u8; length],
            &(length as u64),
        );
        if length == 2 {
            row.expected = Expected::Trap;
        }
        cases.push(row);
    }
    for size in [0usize, 32, 65_522, 65_523] {
        let limits = Limits {
            bytes: 65_537,
            ..DEFAULT_BLOB.request
        };
        let value = ByteBuf::from(vec![42; size]);
        let mut blob = case(
            "default_blob",
            "query",
            DEFAULT_BLOB.id,
            limits,
            &format!("blob-{size}"),
            &value,
            &value,
        );
        blob.limits = DEFAULT_BLOB.request;
        if size == 65_523 {
            blob.expected = Expected::Trap;
        }
        cases.push(blob);
        let mut output = case(
            "default_mutate",
            "update",
            DEFAULT_MUTATE.id,
            limits,
            &format!("reply-{size}"),
            &(size as u32),
            &value,
        );
        output.limits = DEFAULT_MUTATE.request;
        if size == 65_523 {
            output.expected = Expected::Trap;
        } else {
            output.mutation = 1;
        }
        cases.push(output);
    }
    cases.push(case(
        "default_unit",
        "query",
        native::method_id("default_unit"),
        DEFAULT_BLOB.request,
        "unit",
        &(),
        &(),
    ));
    for allowed in [true, false] {
        let mut row = case(
            "composed",
            "update",
            16,
            LIMITS,
            if allowed { "allow" } else { "deny" },
            &allowed,
            &0u64,
        );
        row.expected = if allowed {
            Expected::Count(1)
        } else {
            Expected::Reject
        };
        row.mutation = u64::from(allowed);
        cases.push(row);
    }
    // Each actual export has a valid control plus identity/full-input/pre-copy
    // refusals. Future exports must be added here or the artifact coverage fails.
    let mut methods = BTreeSet::new();
    for base in cases.clone() {
        if !methods.insert(base.method) {
            continue;
        }
        for label in [
            "wrong-id",
            "wrong-magic",
            "wrong-direction",
            "short-header",
            "wrong-id-at-limit",
            "trailing",
            "oversized",
            "truncated",
        ] {
            let mut row = base.clone();
            row.label = label.into();
            row.expected = Expected::Trap;
            row.mutation = 0;
            match label {
                "wrong-id" => row.frame[2] ^= 1,
                "wrong-magic" => row.frame[0] ^= 1,
                "wrong-direction" => row.frame[10] = 1,
                "short-header" => row.frame.truncate(10),
                "wrong-id-at-limit" => {
                    row.frame.resize(row.limits.bytes, 0);
                    row.frame[2] ^= 1;
                }
                "trailing" => row.frame.push(0),
                "oversized" => row.frame = vec![0; row.limits.bytes + 1],
                _ => {
                    row.frame.pop();
                }
            }
            cases.push(row);
        }
    }
    cases
}
fn http(url: &str, post: bool) -> Value {
    let mut command = Command::new("curl");
    command.args(["--fail", "--silent", "--show-error", "--max-time", "60"]);
    if post {
        command.args(["-H", "Content-Type: application/json", "-d", "{}"]);
    }
    let result = command.arg(url).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn integer(value: &Value) -> u64 {
    value.as_str().unwrap().parse().unwrap()
}

#[test]
#[ignore = "explicit instrumented PocketIC, frozen Wasm and ENDPOINT_MATRIX_DIR"]
fn all_endpoint_costs() {
    let out = PathBuf::from(std::env::var_os("ENDPOINT_MATRIX_DIR").unwrap());
    assert!(std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_none());
    let expected_exports: BTreeSet<String> =
        serde_json::from_slice(&fs::read(out.join("exports.json")).unwrap()).unwrap();
    let cases = cases();
    let covered: BTreeSet<_> = cases
        .iter()
        .map(|c| format!("canister_{} {}", c.mode, c.method))
        .collect();
    assert_eq!(
        covered, expected_exports,
        "every exported endpoint needs a cost case"
    );
    let session = PocketIcStartupConfig::from_env(Duration::from_secs(90))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pic = session.instance();
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    let wasm = fs::read(out.join("endpoint.wasm")).unwrap();
    let module_hash = digest(&wasm);
    pic.install_canister(canister, wasm, vec![], None);
    let caller = Principal::anonymous().into();
    let count = || {
        native::decode_response::<u64>(
            8,
            &pic.query_call(
                canister,
                caller,
                "count",
                native::encode_request(8, &(), LIMITS).unwrap(),
            )
            .unwrap(),
            LIMITS,
        )
        .unwrap()
    };
    let viewer = http(
        &format!("{}memory_access_viewer", pic.get_server_url()),
        true,
    );
    let path = format!(
        "http://{}:{}/api/instances/{}/canisters/{}",
        viewer["ip_addr"].as_str().unwrap(),
        viewer["port"].as_u64().unwrap(),
        pic.instance_id(),
        canister.to_text()
    );
    let mut samples = fs::File::create(out.join("samples.jsonl")).unwrap();
    let mut server = None;
    for (case_index, case) in cases.into_iter().enumerate() {
        for repetition in 0..9 {
            let state = count();
            // Clear only this owned private recorder after earlier receipts have
            // been persisted. Observation queries run outside the traced interval.
            http(&format!("{path}/clear"), true);
            http(&format!("{path}/start"), true);
            // Publish a round before sampling. Each call uses a distinct caller
            // so query-result caching cannot replace actual Wasm execution.
            pic.tick();
            let balance = pic.cycle_balance(canister);
            let measured_caller =
                Principal::from_slice(&((case_index * 9 + repetition) as u64).to_le_bytes()).into();
            let result = if case.mode == "update" {
                pic.update_call(canister, measured_caller, case.method, case.frame.clone())
            } else {
                pic.query_call(canister, measured_caller, case.method, case.frame.clone())
            };
            let cycles = balance.checked_sub(pic.cycle_balance(canister)).unwrap();
            http(&format!("{path}/stop"), true);
            let trace = http(&format!("{path}/executions?limit=1000"), false);
            // Preserve raw failures as well as passing receipts before assertions.
            fs::write(
                out.join("latest-trace.json"),
                serde_json::to_vec(&trace).unwrap(),
            )
            .unwrap();
            assert_eq!(trace["more"], false);
            assert_eq!(trace["truncated"], false);
            assert_eq!(integer(&trace["dropped_executions"]), 0);
            assert_eq!(integer(&trace["dropped_instructions"]), 0);
            assert_eq!(trace["dropped_instructions_incomplete"], false);
            if let Some(hash) = &server {
                assert_eq!(hash, &trace["server_binary_hash"]);
            } else {
                server = Some(trace["server_binary_hash"].clone());
                fs::write(
                    out.join("trace-header.json"),
                    serde_json::to_vec_pretty(&trace).unwrap(),
                )
                .unwrap();
            }
            let executions = trace["executions"].as_array().unwrap();
            assert!(
                !executions.is_empty(),
                "missing physical trace for {} / {}: {result:?}",
                case.method,
                case.label
            );
            let mut instructions = 0;
            let mut page_instructions = 0;
            for execution in executions {
                assert_eq!(execution["module_hash"], module_hash);
                assert_eq!(execution["attempt"]["instructions_available"], true);
                assert_eq!(execution["detail_truncated"], false);
                instructions += integer(&execution["instructions"]);
                page_instructions += integer(&execution["page_access_instructions"]);
            }
            assert!(
                executions
                    .iter()
                    .any(|e| e["function"] == format!("canister_{} {}", case.mode, case.method))
            );
            let (outcome, reply_bytes, reply_hash) = match (&case.expected, result) {
                (Expected::Reply(expected), Ok(reply)) => {
                    assert_eq!(&reply, expected, "{} / {}", case.method, case.label);
                    ("reply", Some(reply.len()), Some(digest(&reply)))
                }
                (Expected::Count(delta), Ok(reply)) => {
                    assert_eq!(
                        native::decode_response::<u64>(case.id, &reply, case.limits).unwrap(),
                        state + delta
                    );
                    ("reply", Some(reply.len()), Some(digest(&reply)))
                }
                (Expected::Trap, Err(error)) => {
                    assert_eq!(
                        error.reject_code,
                        RejectCode::CanisterError,
                        "{} / {}: {error:?}",
                        case.method,
                        case.label
                    );
                    ("trap", None, None)
                }
                (Expected::Reject, Err(error)) => {
                    assert_eq!(error.reject_code, RejectCode::CanisterReject);
                    ("reject", None, None)
                }
                (_, unexpected) => panic!("{} / {}: {unexpected:?}", case.method, case.label),
            };
            assert_eq!(
                count(),
                state + case.mutation,
                "{} / {} transaction state",
                case.method,
                case.label
            );
            writeln!(samples,"{}",json!({"method":case.method,"mode":case.mode,"case":case.label,"repetition":repetition,"warmup":repetition==0,"caller":measured_caller.to_text(),"request_bytes":case.frame.len(),"request_sha256":digest(&case.frame),"reply_bytes":reply_bytes,"reply_sha256":reply_hash,"outcome":outcome,"cycles_debited":cycles.to_string(),"ic_instructions":instructions,"heap_page_instructions":page_instructions,"physical_executions":executions})).unwrap();
        }
    }
    pic.stop_canister(canister, None).unwrap();
    pic.delete_canister(canister, None).unwrap();
}
