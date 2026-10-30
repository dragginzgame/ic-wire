//! Matched physical update probe for owned versus borrowed request decoding.
#![cfg(any(target_family = "wasm", test))]
#[cfg(any(test, not(feature = "endpoint-input"), not(feature = "borrowed-input")))]
use ic_wire::Bytes;
#[cfg(any(test, not(feature = "endpoint-input"), feature = "borrowed-input"))]
use ic_wire::BytesRef;
use ic_wire::Limits;
#[cfg(any(test, not(feature = "endpoint-input")))]
use ic_wire::native;
use serde::{Deserialize, Serialize};
const ID: u64 = 77;
const MAX: usize = 1048576;
const LIMITS: Limits = Limits {
    bytes: MAX + 64,
    allocation: 16 * MAX,
    nodes: 64,
    depth: 16,
};
#[derive(Serialize, Deserialize)]
#[cfg(any(test, not(feature = "endpoint-input"), not(feature = "borrowed-input")))]
struct Owned {
    mode: u8,
    nonce: u64,
    body: Bytes<MAX>,
}
#[derive(Serialize, Deserialize)]
#[cfg(any(test, not(feature = "endpoint-input"), feature = "borrowed-input"))]
struct Borrowed<'a> {
    mode: u8,
    nonce: u64,
    #[serde(borrow)]
    body: BytesRef<'a, MAX>,
}
fn consume(mode: u8, nonce: u64, bytes: &[u8]) -> [u64; 3] {
    let selected = match mode {
        0 => bytes,
        1 => &bytes[..bytes.len().min(256)],
        2 => &[],
        _ => panic!("invalid fixture mode"),
    };
    let sum = selected
        .iter()
        .fold(nonce, |sum, byte| sum.wrapping_add(u64::from(*byte)));
    [nonce, selected.len() as u64, sum]
}
#[cfg(any(test, not(feature = "endpoint-input")))]
fn owned(input: Vec<u8>) -> Result<[u64; 3], ic_wire::Error> {
    let value: Owned = native::decode_request(ID, &input, LIMITS)?;
    drop(input); // Fair owned control releases encoded storage before the handler.
    Ok(consume(value.mode, value.nonce, value.body.as_slice()))
}
#[cfg(any(test, not(feature = "endpoint-input")))]
fn borrowed(input: &[u8]) -> Result<[u64; 3], ic_wire::Error> {
    let value: Borrowed<'_> = native::decode_request(ID, input, LIMITS)?;
    Ok(consume(value.mode, value.nonce, value.body.as_slice()))
}
#[cfg(all(
    target_family = "wasm",
    feature = "endpoint-input",
    not(feature = "borrowed-input")
))]
#[ic_wire::endpoint::update(name = "consume", id = ID, request = LIMITS, response = LIMITS)]
fn owned_endpoint(value: Owned) -> [u64; 3] {
    consume(value.mode, value.nonce, value.body.as_slice())
}
#[cfg(all(
    target_family = "wasm",
    feature = "endpoint-input",
    feature = "borrowed-input"
))]
#[ic_wire::endpoint::update(borrow, name = "consume", id = ID, request = LIMITS, response = LIMITS)]
fn borrowed_endpoint(value: Borrowed<'_>) -> [u64; 3] {
    consume(value.mode, value.nonce, value.body.as_slice())
}
#[cfg(all(target_family = "wasm", not(feature = "endpoint-input")))]
#[expect(
    unsafe_code,
    reason = "This fixture owns the IC export; system calls use safe wrappers."
)]
#[unsafe(export_name = "canister_update consume")]
/// Same endpoint lifecycle and bytes; only request ownership differs.
pub extern "C" fn entry() {
    let length = ic_platform::ic0::msg_arg_data_size();
    assert!(length <= LIMITS.bytes);
    let mut input = vec![0; length];
    ic_platform::ic0::msg_arg_data_copy(&mut input, 0);
    let reply = if cfg!(feature = "borrowed-input") {
        let reply = borrowed(&input).unwrap();
        drop(input);
        reply
    } else {
        owned(input).unwrap()
    };
    let mut output = [0; 64];
    let bytes = native::encode_response_to_slice(ID, &reply, LIMITS, &mut output).unwrap();
    ic_platform::ic0::msg_reply_data_append(bytes);
    ic_platform::ic0::msg_reply();
}
#[cfg(test)]
mod tests {
    use super::*;
    fn packet(size: usize, mode: u8, nonce: u64) -> Vec<u8> {
        let body = (0..size)
            .map(|i| (i as u8).wrapping_mul(17).wrapping_add(nonce as u8))
            .collect();
        native::encode_request(
            ID,
            &Owned {
                mode,
                nonce,
                body: Bytes::new(body).unwrap(),
            },
            LIMITS,
        )
        .unwrap()
    }
    #[test]
    fn identical_bytes_borrowed_custody_and_rejection() {
        for size in [0, 32, 32768, MAX] {
            for mode in 0..3 {
                let bytes = packet(size, mode, 19);
                let view: Borrowed<'_> = native::decode_request(ID, &bytes, LIMITS).unwrap();
                assert_eq!(native::encode_request(ID, &view, LIMITS).unwrap(), bytes);
                if size > 0 {
                    assert_eq!(
                        view.body.as_slice().as_ptr(),
                        bytes[bytes.len() - size..].as_ptr()
                    );
                }
                assert_eq!(owned(bytes.clone()), borrowed(&bytes));
                let mut damaged = bytes.clone();
                damaged.push(0);
                assert_eq!(owned(damaged.clone()), Err(ic_wire::Error::Trailing));
                assert_eq!(borrowed(&damaged), Err(ic_wire::Error::Trailing));
                damaged = bytes[..bytes.len() - 1].to_vec();
                assert!(owned(damaged.clone()).is_err());
                assert_eq!(owned(damaged.clone()), borrowed(&damaged));
            }
        }
    }
    #[cfg(feature = "ic")]
    #[test]
    #[ignore = "explicit prepared instrumented PocketIC and frozen guest artifacts"]
    fn physical_updates() {
        use ic_platform::{candid::contract::Principal, pocket_ic::PocketIcBuilder};
        use ic_testkit::pic::PocketIcStartupConfig;
        use serde_json::json;
        use sha2::{Digest, Sha256};
        use std::{fs, io::Write, path::PathBuf, process::Command, time::Duration};
        let out = PathBuf::from(std::env::var_os("IC_WIRE_INPUT_EVIDENCE").unwrap());
        assert!(std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_none());
        let session = PocketIcStartupConfig::from_env(Duration::from_secs(90))
            .unwrap()
            .build_instance(PocketIcBuilder::new().with_application_subnet())
            .unwrap();
        let pic = session.instance();
        let http = |path: &str, post: bool| {
            let mut command = Command::new("curl");
            command.args(["--fail", "--silent", "--show-error", "--max-time", "60"]);
            if post {
                command.args(["-H", "Content-Type: application/json", "-d", "{}"]);
            }
            let output = command.arg(path).output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        let viewer: serde_json::Value = serde_json::from_slice(&http(
            &format!("{}memory_access_viewer", pic.get_server_url()),
            true,
        ))
        .unwrap();
        let viewer = format!(
            "http://{}:{}",
            viewer["ip_addr"].as_str().unwrap(),
            viewer["port"].as_u64().unwrap()
        );
        let mut rows = fs::File::create(out.join("calls.jsonl")).unwrap();
        let mut canisters = Vec::new();
        for lane in ["owned", "borrowed"] {
            let id = pic.create_canister();
            pic.add_cycles(id, 100_000_000_000_000);
            pic.install_canister(
                id,
                fs::read(out.join(format!("{lane}.wasm"))).unwrap(),
                vec![],
                None,
            );
            pic.update_call(id, Principal::anonymous(), "consume", packet(0, 0, 0))
                .unwrap();
            let path = format!(
                "{viewer}/api/instances/{}/canisters/{}",
                pic.instance_id(),
                id.to_text()
            );
            fs::write(
                out.join(format!("{lane}-start.json")),
                http(&format!("{path}/start"), true),
            )
            .unwrap();
            canisters.push((lane, id, path));
        }
        for size in [0, 32, 32768, MAX] {
            for mode in 0..3 {
                for repetition in 0..8 {
                    let nonce = 128 + repetition;
                    let bytes = packet(size, mode, nonce);
                    // Independent logical oracle, rather than the guest's consume function.
                    let n = match mode {
                        0 => size,
                        1 => size.min(256),
                        _ => 0,
                    };
                    let sum = (0..n).fold(nonce, |sum, i| {
                        sum.wrapping_add(u64::from(
                            (i as u8).wrapping_mul(17).wrapping_add(nonce as u8),
                        ))
                    });
                    for offset in 0..2 {
                        let (lane, id, _) = &canisters[((repetition as usize) + offset) % 2];
                        let before = pic.cycle_balance(*id);
                        let reply = pic
                            .update_call(*id, Principal::anonymous(), "consume", bytes.clone())
                            .unwrap();
                        let after = pic.cycle_balance(*id);
                        let value: [u64; 3] = native::decode_response(ID, &reply, LIMITS).unwrap();
                        assert_eq!(value, [nonce, n as u64, sum]);
                        writeln!(rows,"{}",json!({"lane":lane,"size":size,"mode":mode,"repetition":repetition,"nonce":nonce,"request_bytes":bytes.len(),"request_sha256":format!("{:x}",Sha256::digest(&bytes)),"reply_sha256":format!("{:x}",Sha256::digest(&reply)),"cycles":(before-after).to_string()})).unwrap();
                    }
                }
            }
        }
        for (lane, id, path) in canisters {
            fs::write(
                out.join(format!("{lane}-stop.json")),
                http(&format!("{path}/stop"), true),
            )
            .unwrap();
            fs::write(
                out.join(format!("{lane}-executions.json")),
                http(&format!("{path}/executions?limit=1000"), false),
            )
            .unwrap();
            fs::write(
                out.join(format!("{lane}-summary.json")),
                http(&format!("{path}/summary"), false),
            )
            .unwrap();
            pic.stop_canister(id, None).unwrap();
            pic.delete_canister(id, None).unwrap();
        }
    }
}
