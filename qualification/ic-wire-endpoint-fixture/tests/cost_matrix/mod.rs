//! Complete exported-endpoint costs; no instrumentation in the measured canister.
use super::cases::cases;
use ic_testkit::pic::PocketIcStartupConfig;
use ic_testkit::pocket_ic::CanisterId;
use ic_testkit::pocket_ic::PocketIcBuilder;
use ic_wire::native;
use ic_wire_endpoint_fixture::LIMITS;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, io::Write, path::PathBuf, process::Command, time::Duration};

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
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
fn integer(value: &Value) -> u64 {
    value.as_str().unwrap().parse().unwrap()
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "explicit instrumented PocketIC, frozen Wasm and ENDPOINT_MATRIX_DIR"]
async fn all_endpoint_costs() {
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
        .build_instance(
            PocketIcBuilder::new()
                .with_max_request_time_ms(Some(300_000))
                .with_application_subnet(),
        )
        .await
        .unwrap();
    let pic = session.instance();
    let canister = pic.create_canister().await;
    pic.add_cycles(canister, 100_000_000_000_000).await;
    let wasm = fs::read(out.join("endpoint.wasm")).unwrap();
    let module_hash = digest(&wasm);
    pic.install_canister(canister, wasm, vec![], None).await;
    let caller = CanisterId::anonymous();
    let count = || async {
        native::decode_response::<u64>(
            8,
            &pic.query_call(
                canister,
                caller,
                "count",
                native::encode_request(8, &(), LIMITS).unwrap(),
            )
            .await
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
        pic.instance_id,
        canister.to_text()
    );
    let mut samples = fs::File::create(out.join("samples.jsonl")).unwrap();
    let mut server = None;
    for (case_index, case) in cases.into_iter().enumerate() {
        for repetition in 0..9 {
            let state = count().await;
            // Clear only this owned private recorder after earlier receipts have
            // been persisted. Observation queries run outside the traced interval.
            http(&format!("{path}/clear"), true);
            http(&format!("{path}/start"), true);
            // Publish a round before sampling. Each call uses a distinct caller
            // so query-result caching cannot replace actual Wasm execution.
            pic.tick().await;
            let balance = pic.cycle_balance(canister).await;
            let measured_caller =
                CanisterId::from_slice(&((case_index * 9 + repetition) as u64).to_le_bytes());
            let result = if case.mode == "update" {
                pic.update_call(canister, measured_caller, case.method, case.frame.clone())
                    .await
            } else {
                pic.query_call(canister, measured_caller, case.method, case.frame.clone())
                    .await
            };
            let cycles = balance
                .checked_sub(pic.cycle_balance(canister).await)
                .unwrap();
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
            let (outcome, reply) = case.verify(result, state);
            let reply_bytes = reply.as_ref().map(Vec::len);
            let reply_hash = reply.as_ref().map(|bytes| digest(bytes));
            assert_eq!(
                count().await,
                state + case.mutation,
                "{} / {} transaction state",
                case.method,
                case.label
            );
            writeln!(samples,"{}",json!({"method":case.method,"mode":case.mode,"case":case.label,"repetition":repetition,"warmup":repetition==0,"caller":measured_caller.to_text(),"request_bytes":case.frame.len(),"request_sha256":digest(&case.frame),"reply_bytes":reply_bytes,"reply_sha256":reply_hash,"outcome":outcome,"cycles_debited":cycles.to_string(),"ic_instructions":instructions,"heap_page_instructions":page_instructions,"physical_executions":executions})).unwrap();
        }
    }
    pic.stop_canister(canister, None).await.unwrap();
    pic.delete_canister(canister, None).await.unwrap();
    session.close().await;
}
