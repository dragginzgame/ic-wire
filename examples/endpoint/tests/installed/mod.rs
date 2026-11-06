//! Owning installed endpoint/transaction checks; explicit prepared PocketIC only.
use ic_platform_types::principal::Principal;
use ic_testkit::pic::PocketIcStartupConfig;
use ic_testkit::pocket_ic::{ClientError, PocketIcBuilder, RejectCode};
use ic_wire::{ByteBuf, native};
use ic_wire_endpoint_fixture::{LIMITS, Request};
use std::time::Duration;

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires ENDPOINT_WASM and an explicitly prepared POCKET_IC_BIN"]
async fn typed_calls_limits_and_transactions() {
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .await
        .unwrap();
    let pic = owned.instance();
    let canister = pic.create_canister().await.unwrap();
    pic.add_cycles(canister, 100_000_000_000_000).await.unwrap();
    pic.install_canister(
        canister,
        std::fs::read(std::env::var_os("ENDPOINT_WASM").unwrap()).unwrap(),
        vec![],
        None,
    )
    .await
    .unwrap();
    let caller = Principal::anonymous().into();
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
    for malformed in [
        vec![],
        vec![0; 11],
        native::encode_request(99, &(), LIMITS).unwrap(),
        native::encode_response(8, &(), LIMITS).unwrap(),
        [unit.as_slice(), &[0]].concat(),
        vec![0; LIMITS.bytes + 1],
    ] {
        expect_trap(pic.query_call(canister, caller, "count", malformed).await);
    }
    for malformed in [
        frame[..frame.len() - 1].to_vec(),
        [frame.as_slice(), &[0]].concat(),
    ] {
        expect_trap(pic.query_call(canister, caller, "echo", malformed).await);
    }
    // Invalid and over-budget updates cannot dispatch or mutate state.
    for malformed in [
        vec![],
        native::encode_request(8, &0u32, LIMITS).unwrap(),
        vec![0; 129],
    ] {
        expect_trap(pic.update_call(canister, caller, "mutate", malformed).await);
        assert_eq!(count().await, 0);
    }
    for (length, accepted) in [(0usize, true), (1, true), (2, false)] {
        let frame = native::encode_request(15, &vec![1u8; length], LIMITS).unwrap();
        let outcome = pic.query_call(canister, caller, "nodes", frame).await;
        if accepted {
            outcome.unwrap();
        } else {
            expect_trap(outcome);
        }
    }
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
    let unit_reply = pic
        .update_call(
            canister,
            caller,
            "unit-call",
            native::encode_request(10, &(), LIMITS).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        unit_reply,
        native::encode_response(10, &(), LIMITS).unwrap()
    );
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
    let ClientError::Canister(refusal) = refusal else {
        panic!("expected canister guard rejection, got {refusal:?}")
    };
    assert_eq!(
        refusal.reject_code,
        ic_testkit::pocket_ic::RejectCode::CanisterReject
    );
    assert_eq!(
        count().await,
        124,
        "guarded dispatch executes once and refusals retain their transport semantics"
    );
    use ic_wire_endpoint_fixture::{DEFAULT_BLOB, DEFAULT_MUTATE, NAMED};
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
    expect_trap(
        pic.query_call(
            canister,
            caller,
            "default_blob",
            native::encode_request(7, &ByteBuf::new(), wide).unwrap(),
        )
        .await,
    );
    let unit_id = native::method_id("default_unit");
    let reply = pic
        .query_call(
            canister,
            caller,
            "default_unit",
            native::encode_request(unit_id, &(), wide).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reply, native::encode_response(unit_id, &(), wide).unwrap());
    let reply = pic
        .query_call(
            canister,
            caller,
            "stable-name",
            native::encode_request(NAMED.id, &5u64, NAMED.request).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        native::decode_response::<u64>(NAMED.id, &reply, NAMED.response).unwrap(),
        5
    );
    for (size, succeeds) in [(65_522u32, true), (65_523, false)] {
        let result = pic
            .update_call(
                canister,
                caller,
                "default_mutate",
                native::encode_request(DEFAULT_MUTATE.id, &size, DEFAULT_MUTATE.request).unwrap(),
            )
            .await;
        if succeeds {
            result.unwrap();
        } else {
            expect_trap(result);
        }
        assert_eq!(
            count().await,
            125,
            "default response limits retain rollback"
        );
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
    let mut trailing = valid.clone();
    trailing.push(0);
    let before = count().await;
    for frame in [
        trailing,
        valid[..valid.len() - 1].to_vec(),
        native::encode_request(17, &request, LIMITS).unwrap(),
        vec![0; 129],
    ] {
        expect_trap(
            pic.update_call(canister, caller, "borrowed_mutate", frame)
                .await,
        );
        assert_eq!(
            count().await,
            before,
            "borrowed request refusal must precede mutation"
        );
    }
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
    owned.close().await.unwrap();
}

// The client reports transport/protocol failures as Err too. Only an observed
// canister error proves malformed-input refusal or rollback; fail setup/I/O
// errors instead of counting them as successful negative cases. No retry.
fn expect_trap(result: Result<Vec<u8>, ClientError>) {
    match result {
        Err(ClientError::Canister(reject)) => {
            assert_eq!(reject.reject_code, RejectCode::CanisterError)
        }
        other => panic!("expected a canister trap, got {other:?}"),
    }
}
