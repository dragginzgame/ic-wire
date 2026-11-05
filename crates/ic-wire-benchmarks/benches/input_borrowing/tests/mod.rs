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
    use ic_platform::candid::contract::Principal;
    use ic_testkit::pic::PocketIcStartupConfig;
    use ic_testkit::pocket_ic::PocketIcBuilder;
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
        let id = pic.create_canister().unwrap();
        pic.add_cycles(id, 100_000_000_000_000).unwrap();
        pic.install_canister(
            id,
            fs::read(out.join(format!("{lane}.wasm"))).unwrap(),
            vec![],
            None,
        )
        .unwrap();
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
                    let before = pic.cycle_balance(*id).unwrap();
                    let reply = pic
                        .update_call(*id, Principal::anonymous(), "consume", bytes.clone())
                        .unwrap();
                    let after = pic.cycle_balance(*id).unwrap();
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
