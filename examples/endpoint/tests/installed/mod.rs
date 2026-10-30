//! Owning installed endpoint/transaction checks; explicit prepared PocketIC only.
use ic_platform::{pocket_ic::PocketIcBuilder, principal::Principal};
use ic_testkit::pic::PocketIcStartupConfig;
use ic_wire::{ByteBuf, native};
use ic_wire_endpoint_fixture::{LIMITS, Request};
use std::time::Duration;

#[test]
#[ignore = "requires ENDPOINT_WASM and an explicitly prepared POCKET_IC_BIN"]
fn typed_calls_limits_and_transactions() {
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pic = owned.instance();
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(
        canister,
        std::fs::read(std::env::var_os("ENDPOINT_WASM").unwrap()).unwrap(),
        vec![],
        None,
    );
    let caller = Principal::anonymous().into();
    let unit = native::encode_request(8, &(), LIMITS).unwrap();
    let count = || -> u64 {
        native::decode_response(
            8,
            &pic.query_call(canister, caller, "count", unit.clone())
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
        .unwrap();
    assert_eq!(reply, native::encode_response(7, &request, LIMITS).unwrap());
    assert_eq!(
        native::decode_response::<Request>(7, &reply, LIMITS).unwrap(),
        request
    );
    assert_eq!(count(), 0);
    // Unit request identity/direction and complete-input checks must never be skipped.
    for malformed in [
        vec![],
        vec![0; 11],
        native::encode_request(99, &(), LIMITS).unwrap(),
        native::encode_response(8, &(), LIMITS).unwrap(),
        [unit.as_slice(), &[0]].concat(),
        vec![0; LIMITS.bytes + 1],
    ] {
        assert!(
            pic.query_call(canister, caller, "count", malformed)
                .is_err()
        );
    }
    for malformed in [
        frame[..frame.len() - 1].to_vec(),
        [frame.as_slice(), &[0]].concat(),
    ] {
        assert!(pic.query_call(canister, caller, "echo", malformed).is_err());
    }
    // Invalid and over-budget updates cannot dispatch or mutate state.
    for malformed in [
        vec![],
        native::encode_request(8, &0u32, LIMITS).unwrap(),
        vec![0; 129],
    ] {
        assert!(
            pic.update_call(canister, caller, "mutate", malformed)
                .is_err()
        );
        assert_eq!(count(), 0);
    }
    for (length, accepted) in [(0usize, true), (1, true), (2, false)] {
        let frame = native::encode_request(15, &vec![1u8; length], LIMITS).unwrap();
        let outcome = pic.query_call(canister, caller, "nodes", frame);
        assert_eq!(
            outcome.is_ok(),
            accepted,
            "nodes length {length}: {outcome:?}"
        );
    }
    for size in [0u32, 115, 116] {
        let frame = native::encode_request(9, &size, LIMITS).unwrap();
        let reply = pic.update_call(canister, caller, "mutate", frame).unwrap();
        assert_eq!(reply.len(), 12 + size as usize);
        assert_eq!(
            native::decode_response::<ByteBuf>(9, &reply, LIMITS)
                .unwrap()
                .len(),
            size as usize
        );
    }
    assert_eq!(count(), 3);
    for size in [117u32, 200] {
        assert!(
            pic.update_call(
                canister,
                caller,
                "mutate",
                native::encode_request(9, &size, LIMITS).unwrap()
            )
            .is_err()
        );
        assert_eq!(
            count(),
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
        .unwrap();
    assert_eq!(
        native::decode_response::<u64>(12, &reply, LIMITS).unwrap(),
        3
    );
    assert!(
        pic.update_call(
            canister,
            caller,
            "async_ready",
            native::encode_request(14, &117u32, LIMITS).unwrap()
        )
        .is_err()
    );
    assert_eq!(
        count(),
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
        .unwrap();
    assert!(
        native::decode_response::<ByteBuf>(13, &reply, LIMITS)
            .unwrap()
            .is_empty()
    );
    assert_eq!(count(), 113);
    assert!(
        pic.update_call(
            canister,
            caller,
            "async_mutate",
            native::encode_request(13, &117u32, LIMITS).unwrap()
        )
        .is_err()
    );
    assert_eq!(
        count(),
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
        .unwrap_err();
    assert_eq!(
        refusal.reject_code,
        ic_platform::pocket_ic::RejectCode::CanisterReject
    );
    assert_eq!(
        count(),
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
        let result = pic.query_call(canister, caller, "default_blob", frame);
        if length <= 65_522 {
            assert_eq!(
                result.unwrap(),
                native::encode_response(DEFAULT_BLOB.id, &value, DEFAULT_BLOB.response).unwrap()
            );
        } else {
            assert!(result.is_err());
        }
    }
    assert!(
        pic.query_call(
            canister,
            caller,
            "default_blob",
            native::encode_request(7, &ByteBuf::new(), wide).unwrap()
        )
        .is_err()
    );
    let unit_id = native::method_id("default_unit");
    let reply = pic
        .query_call(
            canister,
            caller,
            "default_unit",
            native::encode_request(unit_id, &(), wide).unwrap(),
        )
        .unwrap();
    assert_eq!(reply, native::encode_response(unit_id, &(), wide).unwrap());
    let reply = pic
        .query_call(
            canister,
            caller,
            "stable-name",
            native::encode_request(NAMED.id, &5u64, NAMED.request).unwrap(),
        )
        .unwrap();
    assert_eq!(
        native::decode_response::<u64>(NAMED.id, &reply, NAMED.response).unwrap(),
        5
    );
    for (size, succeeds) in [(65_522u32, true), (65_523, false)] {
        let result = pic.update_call(
            canister,
            caller,
            "default_mutate",
            native::encode_request(DEFAULT_MUTATE.id, &size, DEFAULT_MUTATE.request).unwrap(),
        );
        assert_eq!(result.is_ok(), succeeds);
        assert_eq!(count(), 125, "default response limits retain rollback");
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
        .unwrap();
    assert_eq!(
        reply,
        native::encode_response(17, &request, LIMITS).unwrap()
    );
    let valid = native::encode_request(18, &request, LIMITS).unwrap();
    let mut trailing = valid.clone();
    trailing.push(0);
    let before = count();
    for frame in [
        trailing,
        valid[..valid.len() - 1].to_vec(),
        native::encode_request(17, &request, LIMITS).unwrap(),
        vec![0; 129],
    ] {
        assert!(
            pic.update_call(canister, caller, "borrowed_mutate", frame)
                .is_err()
        );
        assert_eq!(
            count(),
            before,
            "borrowed request refusal must precede mutation"
        );
    }
    let reply = pic
        .update_call(canister, caller, "borrowed_mutate", valid)
        .unwrap();
    assert_eq!(
        native::decode_response::<ByteBuf>(18, &reply, LIMITS)
            .unwrap()
            .as_ref(),
        &[42, 42]
    );
    assert_eq!(count(), before + 1);
    let too_large = ic_wire_endpoint_fixture::Request {
        value: 117,
        ..request
    };
    assert!(
        pic.update_call(
            canister,
            caller,
            "borrowed_mutate",
            native::encode_request(18, &too_large, LIMITS).unwrap()
        )
        .is_err()
    );
    assert_eq!(
        count(),
        before + 1,
        "borrowed reply failure rolls back mutation"
    );
}
