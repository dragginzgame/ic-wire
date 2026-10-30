use ic_platform::{candid::contract::Principal, pocket_ic::PocketIcBuilder};
use ic_testkit::pic::PocketIcStartupConfig;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf, process::Command, time::Duration};
use wire_tripwire_probe::{MAX, packet};
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
fn main() {
    let out = PathBuf::from(std::env::var_os("TRIPWIRE_EVIDENCE").unwrap());
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
        "candid",
        "tripwire_owned",
        "tripwire_selective",
    ] {
        let id = pic.create_canister();
        pic.add_cycles(id, 100_000_000_000_000);
        pic.install_canister(
            id,
            fs::read(out.join(format!("{lane}.wasm"))).unwrap(),
            vec![],
            None,
        );
        pic.update_call(id, Principal::anonymous(), "consume", packet(lane, 0, 0, 0))
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
                        packet(lane, size, 0, nonce)
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
                        packet(lane, size, mode, nonce)
                    };
                    let method = if mode == 3 { "encode" } else { "consume" };
                    let before = pic.cycle_balance(*id);
                    let reply = pic
                        .update_call(*id, Principal::anonymous(), method, bytes.clone())
                        .unwrap();
                    let after = pic.cycle_balance(*id);
                    assert!(reply == expected, "reply mismatch: {lane}/{size}/{mode}/{repetition}");
                    writeln!(rows,"{}",json!({"lane":lane,"size":size,"mode":mode,"method":method,"repetition":repetition,"nonce":nonce.to_string(),"request_bytes":bytes.len(),"request_sha256":hash(&bytes),"reply_bytes":reply.len(),"reply_sha256":hash(&reply),"cycles":(before-after).to_string()})).unwrap();
                }
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
