use super::{MAX, packet};
use ic_platform::{pocket_ic::PocketIcBuilder, principal::Principal};
use ic_testkit::pic::PocketIcStartupConfig;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf, process::Command, time::Duration};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn http(path: &str, post: bool) -> Vec<u8> {
    let mut cmd = Command::new("curl");
    cmd.args(["--fail", "--silent", "--show-error", "--max-time", "60"]);
    if post {
        cmd.args(["-H", "Content-Type: application/json", "-d", "{}"]);
    }
    let r = cmd.arg(path).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    r.stdout
}
#[test]
#[ignore = "explicit instrumented PocketIC and frozen Wasm"]
fn physical_updates() {
    let out = PathBuf::from(std::env::var_os("SELECTIVE_EVIDENCE").unwrap());
    assert!(std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_none());
    let session = PocketIcStartupConfig::from_env(Duration::from_secs(90))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pic = session.instance();
    let viewer: Value = serde_json::from_slice(&http(
        &format!("{}memory_access_viewer", pic.get_server_url()),
        true,
    ))
    .unwrap();
    fs::write(
        out.join("viewer.json"),
        serde_json::to_vec_pretty(&viewer).unwrap(),
    )
    .unwrap();
    let viewer = format!(
        "http://{}:{}",
        viewer["ip_addr"].as_str().unwrap(),
        viewer["port"].as_u64().unwrap()
    );
    let mut rows = fs::File::create_new(out.join("calls.jsonl")).unwrap();
    let mut canisters = Vec::new();
    for lane in [
        "wire_owned",
        "wire_borrowed",
        "wire_direct",
        "wire_selective",
    ] {
        let id = pic.create_canister();
        pic.add_cycles(id, 100_000_000_000_000);
        pic.install_canister(
            id,
            fs::read(out.join(format!("{lane}.wasm"))).unwrap(),
            vec![],
            None,
        );
        pic.update_call(id, Principal::anonymous().into(), "consume", packet(0, 0, 0))
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
        for mode in 0..4 {
            for repetition in 0..8_u64 {
                let nonce = [
                    0,
                    127,
                    128,
                    16383,
                    16384,
                    u32::MAX as u64,
                    u64::MAX - 1,
                    u64::MAX,
                ][repetition as usize];
                let n = match mode {
                    0 => size,
                    1 => size.min(256),
                    _ => 0,
                };
                let sum = (0..n).fold(nonce, |s, i| {
                    s.wrapping_add(u64::from(
                        (i as u8).wrapping_mul(17).wrapping_add(nonce as u8),
                    ))
                });
                for offset in 0..canisters.len() {
                    let (lane, id, _) =
                        &canisters[(repetition as usize + offset) % canisters.len()];
                    let expected = if mode == 3 {
                        packet(size, 0, nonce)
                    } else {
                        [nonce, n as u64, sum]
                            .into_iter()
                            .flat_map(u64::to_le_bytes)
                            .collect()
                    };
                    let bytes = if mode == 3 {
                        (size as u32)
                            .to_le_bytes()
                            .into_iter()
                            .chain(nonce.to_le_bytes())
                            .chain([0])
                            .collect()
                    } else {
                        packet(size, mode, nonce)
                    };
                    let method = if mode == 3 { "encode" } else { "consume" };
                    let before = pic.cycle_balance(*id);
                    let reply = pic
                        .update_call(*id, Principal::anonymous().into(), method, bytes.clone())
                        .unwrap();
                    let after = pic.cycle_balance(*id);
                    assert!(
                        reply == expected,
                        "reply mismatch: {lane}/{size}/{mode}/{repetition}"
                    );
                    writeln!(rows,"{}",json!({"outcome":"reply","case":format!("size-{size}-mode-{mode}"),"lane":lane,"size":size,"mode":mode,"method":method,"repetition":repetition,"nonce":nonce.to_string(),"request_bytes":bytes.len(),"request_sha256":hash(&bytes),"reply_bytes":reply.len(),"reply_sha256":hash(&reply),"cycles":(before-after).to_string()})).unwrap();
                }
            }
        }
    }
    let mut bad_id = packet(MAX, 2, 0);
    bad_id[2] ^= 1;
    let mut bad_direction = packet(32, 2, 0);
    bad_direction[10] = 1;
    let mut truncated = packet(MAX, 2, 0);
    truncated.pop();
    let mut trailing = packet(MAX, 2, 0);
    trailing.push(0);
    let mut overlong_nonce = packet(0, 2, 0);
    overlong_nonce[12] = 0x80;
    overlong_nonce.insert(13, 0);
    let mut overlong_length = packet(0, 2, 0);
    overlong_length[13] = 0x80;
    overlong_length.push(0);
    let field_limit = ic_wire::native::encode_request(
        super::super::ID,
        &(2u8, 0u64, ic_wire::ByteBuf::from(vec![0; MAX + 1])),
        super::super::LIMITS,
    )
    .unwrap();
    for (label, bytes) in [
        ("wrong-id-large", bad_id),
        ("wrong-direction", bad_direction),
        ("short-header", vec![0; 10]),
        ("truncated-large", truncated),
        ("trailing-large", trailing),
        ("overlong-nonce", overlong_nonce),
        ("overlong-length", overlong_length),
        ("field-limit", field_limit),
    ] {
        for repetition in 0..8 {
            for offset in 0..canisters.len() {
                let (lane, id, _) = &canisters[(repetition + offset) % canisters.len()];
                let before = pic.cycle_balance(*id);
                let failure = pic
                    .update_call(*id, Principal::anonymous().into(), "consume", bytes.clone())
                    .unwrap_err();
                assert_eq!(
                    failure.reject_code,
                    ic_platform::pocket_ic::RejectCode::CanisterError
                );
                let cycles = before - pic.cycle_balance(*id);
                writeln!(rows, "{}", json!({"case":label,"lane":lane,"size":bytes.len(),"mode":2,"method":"consume","repetition":repetition,"nonce":"0","request_bytes":bytes.len(),"request_sha256":hash(&bytes),"reply_bytes":0,"reply_sha256":null,"cycles":cycles.to_string(),"outcome":"trap"})).unwrap();
            }
        }
    }
    for (lane, id, path) in canisters {
        for (suffix, post) in [
            ("stop", true),
            ("executions?limit=1000", false),
            ("summary", false),
        ] {
            let name = if suffix.starts_with("executions") {
                "executions"
            } else {
                suffix
            };
            fs::write(
                out.join(format!("{lane}-{name}.json")),
                http(&format!("{path}/{suffix}"), post),
            )
            .unwrap();
        }
        pic.stop_canister(id, None).unwrap();
        pic.delete_canister(id, None).unwrap();
    }
}
