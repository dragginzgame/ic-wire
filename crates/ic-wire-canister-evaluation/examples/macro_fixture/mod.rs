//! Installed qualification of the evaluation-only query/update attributes.
use ic_host_artifacts::wasm::{InspectionLimits, inspect};
use ic_platform::Principal;
use ic_testkit::pic::{CanisterInstallExt, InstallSpec, PocketIcBuilder, PocketIcStartupConfig};
use ic_wire::{Method, decode_response, encode_request};
use ic_wire_canister_evaluation::macro_fixture::{Add, BadReply, Read, Reset};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs};

fn main() {
    let wasm_path = env::args_os().nth(1).expect("Wasm path");
    let wasm = fs::read(&wasm_path).unwrap();
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
    let exports: Vec<_> = facts
        .exports
        .keys()
        .map(|name| (*name).to_owned())
        .collect();
    for expected in [
        format!("canister_query {}", Read::NAME),
        format!("canister_update {}", Add::NAME),
        format!("canister_update {}", Reset::NAME),
    ] {
        assert_eq!(exports.iter().filter(|name| **name == expected).count(), 1);
    }
    assert!(!exports.contains(&format!("canister_update {}", Read::NAME)));
    assert!(!exports.contains(&format!("canister_query {}", Add::NAME)));
    let server = env::var_os("POCKET_IC_BIN").expect("prepared PocketIC server");
    let owned = PocketIcStartupConfig::Spawn {
        server_binary: server.clone().into(),
        timeout: std::time::Duration::from_secs(60),
        server_hard_ttl: None,
        server_idle_ttl: None,
        server_output_files: None,
    }
    .build_instance(PocketIcBuilder::new().with_application_subnet())
    .unwrap();
    let pic = &owned.instance;
    let id = pic
        .try_create_and_install(InstallSpec {
            wasm: wasm.clone(),
            init_bytes: vec![],
            cycles: 10_000_000_000_000,
            install_sender: None,
            label: None,
        })
        .unwrap();
    let caller = Principal::anonymous();
    let query = |value| {
        let bytes = encode_request::<Read>(&value).unwrap();
        let reply = pic
            .query_call(id, caller, Read::NAME, bytes.into_bytes())
            .unwrap();
        decode_response::<Read>(&reply).unwrap()
    };
    let request = encode_request::<Add>(&7).unwrap().into_bytes();
    assert_eq!(query(0), 0);
    let reply = pic
        .update_call(id, caller, Add::NAME, request.clone())
        .unwrap();
    assert_eq!(decode_response::<Add>(&reply).unwrap(), Ok(7));
    assert_eq!(query(2), 9);
    let committed = pic.get_stable_memory(id);
    let mut rejected = 0;
    for bytes in [
        vec![0; 65],
        encode_request::<Add>(&13).unwrap().into_bytes(),
        encode_request::<Read>(&7).unwrap().into_bytes(),
    ] {
        assert!(pic.update_call(id, caller, Add::NAME, bytes).is_err());
        assert_eq!(pic.get_stable_memory(id), committed);
        rejected += 1;
    }
    for end in 0..request.len() {
        assert!(
            pic.update_call(id, caller, Add::NAME, request[..end].to_vec())
                .is_err()
        );
        assert_eq!(pic.get_stable_memory(id), committed);
        rejected += 1;
    }
    assert!(
        pic.update_call(id, Principal::from_slice(&[42]), Add::NAME, request.clone())
            .is_err()
    );
    assert_eq!(pic.get_stable_memory(id), committed);
    rejected += 1;
    let failure = pic
        .update_call(
            id,
            caller,
            Add::NAME,
            encode_request::<Add>(&u64::MAX).unwrap().into_bytes(),
        )
        .unwrap();
    assert_eq!(decode_response::<Add>(&failure).unwrap(), Err(7));
    assert_eq!(pic.get_stable_memory(id), committed);
    assert!(
        pic.query_call(id, caller, Add::NAME, request.clone())
            .is_err()
    );
    // The IC permits replicated execution of a query through an update call.
    let replicated_query = pic
        .update_call(
            id,
            caller,
            Read::NAME,
            encode_request::<Read>(&0).unwrap().into_bytes(),
        )
        .unwrap();
    assert_eq!(decode_response::<Read>(&replicated_query).unwrap(), 7);
    assert_eq!(pic.get_stable_memory(id), committed);
    assert_eq!(query(0), 7);
    // The macro deliberately adds no automatic replay policy.
    let second = pic
        .update_call(id, caller, Add::NAME, request.clone())
        .unwrap();
    assert_eq!(decode_response::<Add>(&second).unwrap(), Ok(14));
    assert_eq!(query(0), 14);
    let before_reply_failure = pic.get_stable_memory(id);
    let error = pic
        .update_call(
            id,
            caller,
            BadReply::NAME,
            encode_request::<BadReply>(&()).unwrap().into_bytes(),
        )
        .unwrap_err();
    assert!(format!("{error:?}").contains("response codec"));
    assert_eq!(pic.get_stable_memory(id), before_reply_failure);
    assert_eq!(query(0), 14);
    let unit = pic
        .update_call(
            id,
            caller,
            Reset::NAME,
            encode_request::<Reset>(&()).unwrap().into_bytes(),
        )
        .unwrap();
    decode_response::<Reset>(&unit).unwrap();
    assert_eq!(unit.len(), 11);
    assert_eq!(query(0), 0);
    let report = json!({"scope":"evaluation-only macro expansion; no Canic source changes","target":"wasm32-unknown-unknown","profile":"canister-evaluation","features":["macro-fixture"],"exports":exports,"request":request,"success":reply,"applicationFailure":failure,"rejectionsWithoutMutation":rejected,"updateViaQueryRejected":true,"replicatedQueryVerified":true,"repeatIsNewEffect":true,"replyFailureRolledBack":true,"finalBalance":0,"syncUnitReply":unit,"wasmSha256":format!("{:x}",Sha256::digest(&wasm))});
    fs::write(
        env::args_os().nth(2).expect("report path"),
        serde_json::to_string_pretty(&report).unwrap() + "\n",
    )
    .unwrap();
    println!(
        "Installed macro query/sync/async update, exports, typed failure and {rejected} rejection cases passed."
    );
}
