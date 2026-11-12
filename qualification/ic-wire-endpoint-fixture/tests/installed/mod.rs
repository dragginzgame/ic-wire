//! Owning installed endpoint/transaction checks; explicit prepared PocketIC only.
use ic_testkit::pic::PocketIcStartupConfig;
use ic_testkit::pocket_ic::CanisterId;
use ic_testkit::pocket_ic::{PocketIcBuilder, RejectCode, RejectResponse};
use ic_wire::{ByteBuf, native};
use ic_wire_endpoint_fixture::{LIMITS, Request};
use std::time::Duration;

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires ENDPOINT_WASM and an explicitly prepared POCKET_IC_BIN"]
async fn typed_calls_limits_and_transactions() {
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))
        .unwrap()
        .build_instance(
            PocketIcBuilder::new()
                .with_max_request_time_ms(Some(300_000))
                .with_application_subnet(),
        )
        .await
        .unwrap();
    let pic = owned.instance();
    let canister = pic.create_canister().await;
    pic.add_cycles(canister, 100_000_000_000_000).await;
    pic.install_canister(
        canister,
        std::fs::read(std::env::var_os("ENDPOINT_WASM").unwrap()).unwrap(),
        vec![],
        None,
    )
    .await;
    let caller = CanisterId::anonymous();
    #[cfg(feature = "admission-contract")]
    {
        let id = native::method_id("admission_contract");
        let limits = ic_wire_endpoint::DEFAULT_REQUEST_LIMITS;
        let frame = native::encode_request(id, &(), limits).unwrap();
        let reply = pic
            .query_call(canister, caller, "admission_contract", frame)
            .await
            .unwrap();
        native::decode_response::<()>(id, &reply, limits).unwrap();
    }
    let unit = native::encode_request(8, &(), LIMITS).unwrap();
    let count = || async {
        native::decode_response::<u64>(
            8,
            &pic.query_call(canister, caller, "count", unit.clone())
                .await
                .unwrap(),
            LIMITS,
        )
        .unwrap()
    };
    let request = Request {
        value: 300,
        bytes: ic_wire::Bytes::new(vec![0, 128, 255]).unwrap(),
    };
    let frame = native::encode_request(7, &request, LIMITS).unwrap();
    let reply = pic
        .query_call(canister, caller, "echo", frame.clone())
        .await
        .unwrap();
    assert_eq!(reply, native::encode_response(7, &request, LIMITS).unwrap());
    assert_eq!(
        native::decode_response::<Request>(7, &reply, LIMITS).unwrap(),
        request
    );
    assert_eq!(count().await, 0);
    // Unit request identity/direction and complete-input checks must never be skipped.
    for malformed in [vec![], vec![0; 11]] {
        expect_trap(pic.query_call(canister, caller, "count", malformed).await);
    }
    expect_trap(pic.update_call(canister, caller, "mutate", vec![]).await);
    assert_eq!(count().await, 0);
    for size in [0u32, 115, 116] {
        let frame = native::encode_request(9, &size, LIMITS).unwrap();
        let reply = pic
            .update_call(canister, caller, "mutate", frame)
            .await
            .unwrap();
        assert_eq!(reply.len(), 12 + size as usize);
        assert_eq!(
            native::decode_response::<ByteBuf>(9, &reply, LIMITS)
                .unwrap()
                .len(),
            size as usize
        );
    }
    assert_eq!(count().await, 3);
    for size in [117u32, 200] {
        expect_trap(
            pic.update_call(
                canister,
                caller,
                "mutate",
                native::encode_request(9, &size, LIMITS).unwrap(),
            )
            .await,
        );
        assert_eq!(
            count().await,
            3,
            "synchronous encoding trap must roll back mutation"
        );
    }
    let reply = pic
        .query_call(
            canister,
            caller,
            "async_query",
            native::encode_request(11, &55u64, LIMITS).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        native::decode_response::<(u64, bool)>(11, &reply, LIMITS).unwrap(),
        (55, true)
    );
    let reply = pic
        .query_call(
            canister,
            caller,
            "composite",
            native::encode_request(12, &(), LIMITS).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        native::decode_response::<u64>(12, &reply, LIMITS).unwrap(),
        3
    );
    expect_trap(
        pic.update_call(
            canister,
            caller,
            "async_ready",
            native::encode_request(14, &117u32, LIMITS).unwrap(),
        )
        .await,
    );
    assert_eq!(
        count().await,
        3,
        "a ready future has not crossed a transaction boundary"
    );
    let reply = pic
        .update_call(
            canister,
            caller,
            "async_mutate",
            native::encode_request(13, &0u32, LIMITS).unwrap(),
        )
        .await
        .unwrap();
    assert!(
        native::decode_response::<ByteBuf>(13, &reply, LIMITS)
            .unwrap()
            .is_empty()
    );
    assert_eq!(count().await, 113);
    expect_trap(
        pic.update_call(
            canister,
            caller,
            "async_mutate",
            native::encode_request(13, &117u32, LIMITS).unwrap(),
        )
        .await,
    );
    assert_eq!(
        count().await,
        123,
        "pre-await +10 commits; callback +100 rolls back"
    );
    let reply = pic
        .update_call(
            canister,
            caller,
            "composed",
            native::encode_request(16, &true, LIMITS).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        native::decode_response::<u64>(16, &reply, LIMITS).unwrap(),
        124
    );
    let refusal = pic
        .update_call(
            canister,
            caller,
            "composed",
            native::encode_request(16, &false, LIMITS).unwrap(),
        )
        .await
        .unwrap_err();
    assert_eq!(
        refusal.reject_code,
        ic_testkit::pocket_ic::RejectCode::CanisterReject
    );
    assert_eq!(
        count().await,
        124,
        "guarded dispatch executes once and refusals retain their transport semantics"
    );
    use ic_wire_endpoint_fixture::DEFAULT_BLOB;
    let wide = ic_wire::Limits {
        bytes: 65_537,
        ..DEFAULT_BLOB.request
    };
    // A ByteBuf at this scale has a three-byte length prefix plus the frame.
    for length in [65_521usize, 65_522, 65_523] {
        let value = ByteBuf::from(vec![42; length]);
        let frame = native::encode_request(DEFAULT_BLOB.id, &value, wide).unwrap();
        assert_eq!(frame.len(), length + 14);
        let result = pic
            .query_call(canister, caller, "default_blob", frame)
            .await;
        if length <= 65_522 {
            assert_eq!(
                result.unwrap(),
                native::encode_response(DEFAULT_BLOB.id, &value, DEFAULT_BLOB.response).unwrap()
            );
        } else {
            expect_trap(result);
        }
    }
    let request = ic_wire_endpoint_fixture::Request {
        value: 2,
        bytes: ic_wire::Bytes::new(vec![1, 2, 3]).unwrap(),
    };
    let reply = pic
        .query_call(
            canister,
            caller,
            "borrowed_echo",
            native::encode_request(17, &request, LIMITS).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        reply,
        native::encode_response(17, &request, LIMITS).unwrap()
    );
    let valid = native::encode_request(18, &request, LIMITS).unwrap();
    let before = count().await;
    let reply = pic
        .update_call(canister, caller, "borrowed_mutate", valid)
        .await
        .unwrap();
    assert_eq!(
        native::decode_response::<ByteBuf>(18, &reply, LIMITS)
            .unwrap()
            .as_ref(),
        &[42, 42]
    );
    assert_eq!(count().await, before + 1);
    let too_large = ic_wire_endpoint_fixture::Request {
        value: 117,
        ..request
    };
    expect_trap(
        pic.update_call(
            canister,
            caller,
            "borrowed_mutate",
            native::encode_request(18, &too_large, LIMITS).unwrap(),
        )
        .await,
    );
    assert_eq!(
        count().await,
        before + 1,
        "borrowed reply failure rolls back mutation"
    );
    // Run the exact cost catalogue on the standard server too. Relative state
    // checks preserve each case's rollback proof after the ordered sequence.
    for (index, case) in super::cases::cases().into_iter().enumerate() {
        let state = count().await;
        let scenario_caller = CanisterId::from_slice(&(index as u64).to_le_bytes());
        let result = if case.mode == "update" {
            pic.update_call(canister, scenario_caller, case.method, case.frame.clone())
                .await
        } else {
            pic.query_call(canister, scenario_caller, case.method, case.frame.clone())
                .await
        };
        case.verify(result, state);
        assert_eq!(
            count().await,
            state + case.mutation,
            "{} / {} transaction state",
            case.method,
            case.label
        );
    }
    owned.close().await;
}

// Upstream PocketIC returns canister rejections and panics on transport/setup
// failures. Only the expected reject code proves refusal or rollback. No retry.
fn expect_trap(result: Result<Vec<u8>, RejectResponse>) {
    match result {
        Err(reject) => {
            assert_eq!(reject.reject_code, RejectCode::CanisterError)
        }
        other => panic!("expected a canister trap, got {other:?}"),
    }
}
