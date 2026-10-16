//! Installed direct-caller authorization, atomic effect and retained-byte trial.
use candid::Principal;
use ic_wire::{Bytes, List, Method, Text, decode_response, encode_request, encode_response};
use ic_wire_canister_evaluation::commands::{
    Apply, Command as Request, Failure, Inspect, Inspection, Receipt,
};
use pocket_ic::PocketIcBuilder;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf, process::Command};

fn encode(subject: Principal, audience: Principal, expiry: u64, id: u64, amount: u64) -> Vec<u8> {
    encode_request::<Apply>(&Request {
        subject: Bytes::new(subject.as_slice().to_vec()).unwrap(),
        audience: Bytes::new(audience.as_slice().to_vec()).unwrap(),
        expiry,
        id,
        amount,
    })
    .unwrap()
    .into_bytes()
}

fn main() {
    let path = PathBuf::from(env::args_os().nth(1).expect("Wasm path"));
    let server = PathBuf::from(env::var_os("POCKET_IC_BIN").expect("prepared PocketIC server"));
    let wasm = fs::read(path).unwrap();
    let server_version = Command::new(&server).arg("--version").output().unwrap();
    assert!(server_version.status.success());
    let rustc = Command::new("rustc").arg("-vV").output().unwrap();
    assert!(rustc.status.success());
    let pic = PocketIcBuilder::new()
        .with_server_binary(server.clone())
        .with_application_subnet()
        .build();
    let canister = pic.create_canister();
    pic.add_cycles(canister, 10_000_000_000_000);
    pic.install_canister(canister, wasm.clone(), vec![], None);
    let caller = Principal::anonymous(); // PocketIC's default fixture controller.
    let other = Principal::from_slice(&[42]);
    let expiry = u64::MAX;
    let bytes = encode(caller, canister, expiry, 1, 7);
    let call = |method: &str, bytes: Vec<u8>| pic.update_call(canister, caller, method, bytes);
    let expected = encode_response::<Apply>(&Ok(Receipt { id: 1, balance: 7 }))
        .unwrap()
        .into_bytes();
    // Discard a successful response to model client uncertainty after commit.
    drop(call("replay_command", bytes.clone()).unwrap());
    assert_eq!(call("replay_command", bytes.clone()).unwrap(), expected);
    let committed = pic.get_stable_memory(canister);
    assert_eq!(u64::from_le_bytes(committed[..8].try_into().unwrap()), 7);
    assert_eq!(u64::from_le_bytes(committed[8..16].try_into().unwrap()), 1);
    let mut rejected = 0;
    for invalid in [
        encode(other, canister, expiry, 2, 1),
        encode(caller, other, expiry, 2, 1),
        encode(caller, canister, 0, 2, 1),
        encode(caller, canister, expiry, 1, 8),
        encode(caller, canister, expiry - 1, 1, 7),
        vec![0; 129],
    ] {
        assert!(call("replay_command", invalid).is_err());
        assert_eq!(pic.get_stable_memory(canister), committed);
        rejected += 1;
    }
    assert!(
        pic.update_call(canister, other, "replay_command", bytes.clone())
            .is_err()
    );
    assert_eq!(pic.get_stable_memory(canister), committed);
    rejected += 1;
    for end in 0..bytes.len() {
        assert!(call("replay_command", bytes[..end].to_vec()).is_err());
        assert_eq!(pic.get_stable_memory(canister), committed);
        rejected += 1;
    }
    let next = encode(caller, canister, expiry, 2, 5);
    assert!(call("replay_trap", next.clone()).is_err());
    assert_eq!(
        pic.get_stable_memory(canister),
        committed,
        "trap must roll back effect and receipt"
    );
    let second = call("replay_command", next.clone()).unwrap();
    assert_eq!(
        second,
        encode_response::<Apply>(&Ok(Receipt { id: 2, balance: 12 }))
            .unwrap()
            .into_bytes()
    );
    let before_upgrade = pic.get_stable_memory(canister);
    pic.upgrade_canister(canister, wasm.clone(), vec![], None)
        .unwrap();
    assert_eq!(call("replay_command", bytes.clone()).unwrap(), expected);
    assert_eq!(call("replay_command", next).unwrap(), second);
    assert_eq!(pic.get_stable_memory(canister), before_upgrade);
    let overflow = encode(caller, canister, expiry, 3, u64::MAX);
    let failure = call("replay_command", overflow.clone()).unwrap();
    assert_eq!(
        failure,
        encode_response::<Apply>(&Err(Failure::Overflow(Receipt { id: 3, balance: 12 })))
            .unwrap()
            .into_bytes()
    );
    assert_eq!(call("replay_command", overflow).unwrap(), failure);
    for id in 4..=8 {
        call("replay_command", encode(caller, canister, expiry, id, 1)).unwrap();
    }
    let full = pic.get_stable_memory(canister);
    assert!(call("replay_command", encode(caller, canister, expiry, 9, 1)).is_err());
    assert_eq!(call("replay_command", bytes.clone()).unwrap(), expected);
    assert_eq!(pic.get_stable_memory(canister), full);
    assert_eq!(u64::from_le_bytes(full[..8].try_into().unwrap()), 17);
    assert_eq!(u64::from_le_bytes(full[8..16].try_into().unwrap()), 8);
    let inspection = Inspection {
        labels: List::new(vec![Text::new("é".repeat(16)).unwrap(); 4]).unwrap(),
        correlation: Some(u64::MAX),
    };
    let query_bytes = encode_request::<Inspect>(&inspection).unwrap();
    let query_reply = pic
        .query_call(
            canister,
            caller,
            Inspect::NAME,
            query_bytes.as_bytes().to_vec(),
        )
        .unwrap();
    let snapshot = decode_response::<Inspect>(&query_reply).unwrap();
    assert_eq!(snapshot.balance, 17);
    assert_eq!(snapshot.inspection, inspection);
    assert_eq!(pic.get_stable_memory(canister), full);
    assert!(
        pic.query_call(canister, caller, Inspect::NAME, bytes.clone())
            .is_err()
    );
    for end in 0..query_bytes.as_bytes().len() {
        assert!(
            pic.query_call(
                canister,
                caller,
                Inspect::NAME,
                query_bytes.as_bytes()[..end].to_vec()
            )
            .is_err()
        );
    }
    let receiver = pic.create_canister();
    let forwarder = pic.create_canister();
    for id in [receiver, forwarder] {
        pic.add_cycles(id, 10_000_000_000_000);
        pic.install_canister(id, wasm.clone(), vec![], None);
    }
    pic.set_controllers(receiver, None, vec![caller, forwarder])
        .unwrap();
    let forwarded = encode(forwarder, receiver, expiry, 1, 11);
    let forwarded_reply = pic
        .update_call(forwarder, caller, "runtime_forward", forwarded.clone())
        .unwrap();
    assert_eq!(
        decode_response::<Apply>(&forwarded_reply).unwrap(),
        Ok(Receipt { id: 1, balance: 11 })
    );
    let peer_state = pic.get_stable_memory(receiver);
    assert_eq!(u64::from_le_bytes(peer_state[..8].try_into().unwrap()), 11);
    assert_eq!(u64::from_le_bytes(peer_state[8..16].try_into().unwrap()), 1);
    assert!(
        pic.update_call(forwarder, other, "runtime_forward", forwarded)
            .is_err()
    );
    assert_eq!(pic.get_stable_memory(receiver), peer_state);
    let source_hashes = [
        "Cargo.toml",
        "docs/runtime.md",
        "crates/ic-wire/Cargo.toml",
        "crates/ic-wire-derive/Cargo.toml",
        "crates/ic-wire/src/lib.rs",
        "crates/ic-wire/src/contract/mod.rs",
        "crates/ic-wire/src/bounded/mod.rs",
        "crates/ic-wire/src/shape/mod.rs",
        "crates/ic-wire-derive/src/lib.rs",
        "crates/ic-wire-canister-evaluation/src/commands/mod.rs",
        "Cargo.lock",
        "crates/ic-wire-canister-evaluation/Cargo.toml",
        "crates/ic-wire-canister-evaluation/src/lib.rs",
        "crates/ic-wire-canister-evaluation/src/authority/mod.rs",
        "crates/ic-wire-canister-evaluation/src/replay/mod.rs",
        "crates/ic-wire-canister-evaluation/examples/replay_fixture/mod.rs",
        "scripts/ci/evaluate-replay.sh",
    ]
    .into_iter()
    .map(|path| {
        (
            path,
            format!("{:x}", Sha256::digest(fs::read(path).unwrap())),
        )
    })
    .collect::<std::collections::BTreeMap<_, _>>();
    let report = serde_json::to_string_pretty(&json!({
        "scenario": "direct-controller atomic local command; eight retained receipts",
        "wasm_sha256": format!("{:x}", Sha256::digest(&wasm)),
        "server_sha256": format!("{:x}", Sha256::digest(fs::read(server).unwrap())),
        "server_version": String::from_utf8(server_version.stdout).unwrap(),
        "rustc": String::from_utf8(rustc.stdout).unwrap(),
        "profile": "canister-evaluation; wasm32-unknown-unknown; replay feature; ic-wire runtime Postcard",
        "rustflags": env::var("RUSTFLAGS").unwrap_or_default(),
        "cargo_encoded_rustflags": env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default(),
        "source_sha256": source_hashes,
        "request_hex": bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "response_hex": expected.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "rejections_without_mutation": rejected,
        "intercanister_calls": 2, "peer_effect_count": 1, "peer_balance": 11,
        "nested_query_roundtrip": true, "query_truncations_rejected": query_bytes.as_bytes().len(),
        "trap_rollback": true, "upgrade_replay": true,
        "overflow_response_retained": true, "capacity_refusal": true,
        "final_balance": 17, "retained_receipts": 8,
        "limits": "No delegated proofs, external effects, browser command or format migration; lost reply modeled by discarding a successful response."
    })).unwrap();
    fs::write(env::args_os().nth(2).expect("report path"), report + "\n").unwrap();
}
