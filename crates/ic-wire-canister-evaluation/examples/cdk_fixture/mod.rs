//! Execute only against an explicit prepared PocketIC server and fixture Wasm.
use candid::Principal;
use ic_testkit::{
    pic::PocketIcStartupConfig,
    pocket_ic::{ErrorCode, PocketIcBuilder},
};
use ic_wire::{Method, decode_response, encode_request};
use ic_wire_canister_evaluation::cdk_fixture::{BadReply, Read};
use std::{env, fs, time::Duration};
fn main() {
    let wasm = fs::read(env::args_os().nth(1).expect("fixture Wasm path")).unwrap();
    let managed = PocketIcStartupConfig::from_env(Duration::from_secs(30))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pic = &managed.instance;
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(canister, wasm, vec![], None);
    let caller = Principal::anonymous();
    let read = encode_request::<Read>(&()).unwrap().into_bytes();
    let before = pic.get_stable_memory(canister);
    let state = || {
        decode_response::<Read>(
            &pic.query_call(canister, caller, Read::NAME, read.clone())
                .unwrap(),
        )
        .unwrap()
    };
    assert_eq!(state(), 0);
    let error = pic
        .update_call(
            canister,
            caller,
            BadReply::NAME,
            encode_request::<BadReply>(&999).unwrap().into_bytes(),
        )
        .unwrap_err();
    assert!(matches!(
        error.error_code,
        ErrorCode::CanisterTrapped | ErrorCode::CanisterCalledTrap
    ));
    assert!(error.reject_message.contains("response codec"));
    assert_eq!(state(), 0);
    assert!(pic.get_stable_memory(canister) == before);
    for bytes in [
        vec![0; Read::REQUEST_LIMITS.bytes + 1],
        b"DIDL\0\0".to_vec(),
        encode_request::<BadReply>(&0).unwrap().into_bytes(),
    ] {
        assert!(pic.query_call(canister, caller, Read::NAME, bytes).is_err());
        assert!(pic.get_stable_memory(canister) == before);
    }
    let stranger = Principal::self_authenticating(b"unauthorized");
    let error = pic
        .query_call(canister, stranger, Read::NAME, vec![0; 4096])
        .unwrap_err();
    assert!(error.reject_message.contains("controller required"));
    println!(
        "PASS: raw CDK query, authorization before size/copy, method limits and malformed input; reply encoding failure rolled back stable growth and writes."
    );
    fs::write(
        env::args_os().nth(2).expect("report path"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "query": true,
            "authorization_before_admission": true,
            "malformed_and_oversized_rejected": true,
            "stable_growth_and_write_rollback": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
