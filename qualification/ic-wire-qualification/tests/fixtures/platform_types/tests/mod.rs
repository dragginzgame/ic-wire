use super::*;
use ic_wire::{Limits, native, typescript};

const LIMITS: Limits = Limits {
    bytes: 512,
    allocation: 65536,
    nodes: 1024,
    depth: 16,
};

#[test]
fn owned_identities_implement_the_consumers_wire_trait() {
    fn accepts<T: ic_wire::typescript::Wire>() {}
    accepts::<ic_platform_types::Principal>();
    accepts::<ic_platform_types::Account>();
}

#[test]
fn principal_schema_and_frames() {
    assert_eq!(
        typescript::native_schema::<Principal>().unwrap(),
        serde_json::json!({"k": "bytes", "max": 29})
    );
    let schema = typescript::native_schema::<Request>().unwrap();
    let mut vectors = Vec::new();
    for raw in [vec![], vec![4], vec![0; 29], vec![255; 29]] {
        let value = Request {
            caller: Principal::from_slice(&raw),
        };
        let request = native::encode_request(17, &value, LIMITS).unwrap();
        let response = native::encode_response(17, &value, LIMITS).unwrap();
        // A one-field Serde struct has the same payload as its byte buffer.
        assert_eq!(
            request,
            native::encode_request(17, serde_bytes::Bytes::new(&raw), LIMITS).unwrap()
        );
        assert_eq!(
            native::decode_request::<Request>(17, &request, LIMITS).unwrap(),
            value
        );
        assert_eq!(
            native::decode_response::<Request>(17, &response, LIMITS).unwrap(),
            value
        );
        vectors.push(serde_json::json!({"raw": raw, "request": request, "response": response}));
    }
    let mut invalid = Vec::new();
    for length in [30, 31, 255] {
        let raw = vec![0; length];
        assert!(Principal::try_from_slice(&raw).is_err());
        let request =
            native::encode_request(17, serde_bytes::Bytes::new(&raw), LIMITS).unwrap();
        let response =
            native::encode_response(17, serde_bytes::Bytes::new(&raw), LIMITS).unwrap();
        assert!(native::decode_request::<Request>(17, &request, LIMITS).is_err());
        assert!(native::decode_response::<Request>(17, &response, LIMITS).is_err());
        invalid.push(serde_json::json!({"raw": raw, "request": request, "response": response}));
    }
    if let Some(directory) = std::env::var_os("WIRE_PLATFORM_TYPES_OUTPUT") {
        let directory = std::path::PathBuf::from(directory);
        let mut method = serde_json::json!({
            "name": "principal", "mode": "query", "id": "17", "request": schema,
            "response": schema
        });
        method.as_object_mut().unwrap().append(
            typescript::method_limits(LIMITS, LIMITS)
                .unwrap()
                .as_object_mut()
                .unwrap(),
        );
        std::fs::write(
            directory.join("principal.ts"),
            typescript::generate_registry(
                &Default::default(),
                &[method],
                "./wire-runtime.js",
                &Default::default(),
                |report| eprint!("{report}"),
            )
            .unwrap(),
        )
        .unwrap();
        std::fs::write(directory.join("wire-runtime.ts"), typescript::RUNTIME).unwrap();
        std::fs::write(
            directory.join("vectors.json"),
            serde_json::to_vec(&serde_json::json!({
                "valid": vectors, "invalid": invalid
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn account_schema_and_frames_preserve_representation() {
    use ic_platform_types::Account;
    assert_eq!(
        typescript::native_schema::<Account>().unwrap(),
        serde_json::json!({
            "k": "record", "fields": [
                {"name": "owner", "value": {"k": "bytes", "max": 29}},
                {"name": "subaccount", "value": {"k": "option", "item": {
                    "k": "bytes", "min": 32, "max": 32
                }}}
            ]
        })
    );
    let schema = typescript::native_schema::<AccountRequest>().unwrap();
    let absent = Account::from(Principal::anonymous());
    let zero = Account {
        subaccount: Some([0; 32]),
        ..absent
    };
    assert_eq!(absent, zero);
    assert_ne!(
        native::encode_request(18, &absent, LIMITS).unwrap(),
        native::encode_request(18, &zero, LIMITS).unwrap()
    );
    let mut vectors = Vec::new();
    for raw in [vec![], vec![4], vec![255; 29]] {
        for subaccount in [None, Some([0; 32]), Some([255; 32])] {
            let value = AccountRequest {
                account: Account {
                    owner: Principal::from_slice(&raw),
                    subaccount,
                },
            };
            let request = native::encode_request(18, &value, LIMITS).unwrap();
            let response = native::encode_response(18, &value, LIMITS).unwrap();
            // Match the new byte field, not normalized Account equality.
            assert_eq!(
                request,
                native::encode_request(
                    18,
                    &(
                        serde_bytes::Bytes::new(&raw),
                        subaccount.map(serde_bytes::ByteArray::new)
                    ),
                    LIMITS
                )
                .unwrap()
            );
            let decoded =
                native::decode_request::<AccountRequest>(18, &request, LIMITS).unwrap();
            assert_eq!(decoded.account.owner, value.account.owner);
            assert_eq!(decoded.account.subaccount, subaccount);
            let decoded =
                native::decode_response::<AccountRequest>(18, &response, LIMITS).unwrap();
            assert_eq!(decoded.account.subaccount, subaccount);
            vectors.push(serde_json::json!({"owner": raw, "subaccount": subaccount,
            "request": request, "response": response}));
        }
    }
    if let Some(directory) = std::env::var_os("WIRE_PLATFORM_TYPES_OUTPUT") {
        let directory = std::path::PathBuf::from(directory);
        let mut method = serde_json::json!({
            "name": "account", "mode": "query", "id": "18", "request": schema,
            "response": schema
        });
        method.as_object_mut().unwrap().append(
            typescript::method_limits(LIMITS, LIMITS)
                .unwrap()
                .as_object_mut()
                .unwrap(),
        );
        std::fs::write(
            directory.join("account.ts"),
            typescript::generate_registry(
                &Default::default(),
                &[method],
                "./wire-runtime.js",
                &Default::default(),
                |report| eprint!("{report}"),
            )
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("accounts.json"),
            serde_json::to_vec(&vectors).unwrap(),
        )
        .unwrap();
    }
}

fn account_frames<T: Serialize>(value: &T) -> (Vec<u8>, Vec<u8>) {
    (
        native::encode_request(18, value, LIMITS).unwrap(),
        native::encode_response(18, value, LIMITS).unwrap(),
    )
}

#[test]
fn malformed_account_frames_are_refused() {
    // Byte-prefixed subaccounts must contain exactly 32 bytes. The retired
    // unprefixed present-zero layout is a refusal control, not a fallback.
    let owner = serde_bytes::Bytes::new(&[4]);
    let valid = ic_platform_types::Account::from(Principal::anonymous());
    let mut cases = vec![
        (
            "short subaccount",
            account_frames(&(owner, Some(serde_bytes::ByteArray::new([0_u8; 31])))),
        ),
        (
            "long subaccount",
            account_frames(&(owner, Some(serde_bytes::ByteArray::new([0_u8; 33])))),
        ),
        (
            "retired unprefixed present-zero field",
            account_frames(&(owner, 1_u8, [0_u8; 32])),
        ),
        ("invalid option tag", account_frames(&(owner, 2_u8))),
        (
            "oversized owner",
            account_frames(&(serde_bytes::Bytes::new(&[0; 30]), None::<[u8; 32]>)),
        ),
        ("trailing bytes", account_frames(&(valid, 7_u8))),
        (
            "wrong method",
            (
                native::encode_request(19, &valid, LIMITS).unwrap(),
                native::encode_response(19, &valid, LIMITS).unwrap(),
            ),
        ),
    ];
    let (request, response) = account_frames(&valid);
    cases.push(("wrong direction", (response, request)));
    let mut vectors = Vec::new();
    for (name, (request, response)) in cases {
        assert!(
            native::decode_request::<AccountRequest>(18, &request, LIMITS).is_err(),
            "{name}"
        );
        assert!(
            native::decode_response::<AccountRequest>(18, &response, LIMITS).is_err(),
            "{name}"
        );
        vectors
            .push(serde_json::json!({"name": name, "request": request, "response": response}));
    }
    if let Some(directory) = std::env::var_os("WIRE_PLATFORM_TYPES_OUTPUT") {
        std::fs::write(
            std::path::PathBuf::from(directory).join("accounts-invalid.json"),
            serde_json::to_vec(&vectors).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn account_redundant_length_matches_upstream_and_preserves_zero_subaccount() {
    let owner = serde_bytes::Bytes::new(&[4]);
    let (request, response) = account_frames(&(owner, 1_u8, 0xa0_u8, 0_u8, [0_u8; 32]));
    let expected = account_frames(&(owner, Some(serde_bytes::ByteArray::new([0_u8; 32]))));
    let request_value = native::decode_request::<AccountRequest>(18, &request, LIMITS).unwrap();
    let response_value = native::decode_response::<AccountRequest>(18, &response, LIMITS).unwrap();
    assert_eq!(native::encode_request(18, &request_value, LIMITS).unwrap(), expected.0);
    assert_eq!(native::encode_response(18, &response_value, LIMITS).unwrap(), expected.1);
    if let Some(directory) = std::env::var_os("WIRE_PLATFORM_TYPES_OUTPUT") {
        std::fs::write(
            std::path::PathBuf::from(directory).join("accounts-normalized.json"),
            serde_json::to_vec(&serde_json::json!({
                "request": request, "response": response,
                "normalizedRequest": expected.0, "normalizedResponse": expected.1
            })).unwrap(),
        ).unwrap();
    }
}

#[cfg(feature = "endpoint")]
#[test]
fn combined_endpoint_uses_the_same_schema_trait() {
    assert_eq!(ACCOUNT_CONTRACT.id, native::method_id("echo_account_bytes"));
    assert_ne!(ACCOUNT_CONTRACT.id, native::method_id("echo_account"));
    assert_eq!(
        ACCOUNT_CONTRACT.request_schema().unwrap(),
        typescript::native_schema::<AccountRequest>().unwrap()
    );
    assert_eq!(
        ACCOUNT_CONTRACT.response_schema().unwrap(),
        typescript::native_schema::<AccountRequest>().unwrap()
    );
}
#[test]
fn account_byte_contract_and_admission_boundaries() {
    // An unprefixed tuple is a qualification control, never a fallback reader.
    type ArrayControl = (Principal, Option<[u8; 32]>);
    assert_eq!(
        std::mem::size_of::<ArrayControl>(),
        std::mem::size_of::<ic_platform_types::Account>()
    );
    fn minimum(mut accepts: impl FnMut(usize) -> bool, high: usize) -> usize {
        assert!(accepts(high));
        let (mut low, mut high) = (0, high);
        while low < high {
            let middle = low + (high - low) / 2;
            if accepts(middle) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        low
    }
    let mut rows = Vec::new();
    for (state, subaccount) in [
        ("absent", None),
        ("zero", Some([0; 32])),
        ("nonzero", Some([255; 32])),
    ] {
        let value = ic_platform_types::Account {
            owner: Principal::anonymous(),
            subaccount,
        };
        let control = (value.owner, subaccount);
        let old = native::encode_request(18, &control, LIMITS).unwrap();
        let current = native::encode_request(18, &value, LIMITS).unwrap();
        if subaccount.is_none() {
            assert_eq!(old, current);
        } else {
            let offset = native::REQUEST_HEADER_BYTES + 2 + 1; // owner len/data, Some tag
            assert_eq!(&old[..offset], &current[..offset]);
            assert_eq!(current[offset], 32);
            assert_eq!(&old[offset..], &current[offset + 1..]);
            assert_eq!(
                native::decode_request::<ic_platform_types::Account>(18, &old, LIMITS),
                Err(ic_wire::Error::Malformed)
            );
        }
        for encode in [true, false] {
            for (metric, upper) in [
                ("bytes", LIMITS.bytes),
                ("allocation", LIMITS.allocation),
                ("nodes", LIMITS.nodes),
                ("depth", LIMITS.depth),
            ] {
                let result = |array: bool, amount: usize| {
                    let mut limits = LIMITS;
                    match metric {
                        "bytes" => limits.bytes = amount,
                        "allocation" => limits.allocation = amount,
                        "nodes" => limits.nodes = amount,
                        _ => limits.depth = amount,
                    }
                    if encode {
                        if array {
                            native::encode_request(18, &control, limits).map(|_| ())
                        } else {
                            native::encode_request(18, &value, limits).map(|_| ())
                        }
                    } else if array {
                        native::decode_request::<ArrayControl>(18, &old, limits).map(|_| ())
                    } else {
                        native::decode_request::<ic_platform_types::Account>(
                            18, &current, limits,
                        )
                        .map(|_| ())
                    }
                };
                let before = minimum(|amount| result(true, amount).is_ok(), upper);
                let after = minimum(|amount| result(false, amount).is_ok(), upper);
                assert_eq!(result(true, before - 1), Err(ic_wire::Error::Limit));
                assert_eq!(result(false, after - 1), Err(ic_wire::Error::Limit));
                rows.push(serde_json::json!({"state": state, "operation": if encode {"encode"} else {"decode"},
                    "metric": metric, "array_control_min": before, "byte_field_min": after}));
            }
        }
    }
    if let Some(directory) = std::env::var_os("WIRE_PLATFORM_TYPES_OUTPUT") {
        std::fs::write(std::path::PathBuf::from(directory).join("account-budgets.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"arch": std::env::consts::ARCH, "pointer_bits": usize::BITS, "rows": rows})).unwrap()).unwrap();
    }
}
#[test]
fn account_cut_requires_a_fresh_frame_key_even_when_old_bytes_decode() {
    #[derive(Serialize)]
    struct OldAccount {
        owner: Principal,
        subaccount: Option<[u8; 32]>,
    }
    #[derive(Serialize)]
    struct OldRequest {
        account: OldAccount,
        blob: ic_wire::ByteBuf,
    }
    #[derive(Debug, Deserialize)]
    struct CurrentRequest {
        account: ic_platform_types::Account,
        blob: ic_wire::ByteBuf,
    }
    let mut old_subaccount = [0; 32];
    old_subaccount[0] = 0x20;
    let old = OldRequest {
        account: OldAccount {
            owner: Principal::anonymous(),
            subaccount: Some(old_subaccount),
        },
        blob: ic_wire::ByteBuf::from(vec![0x02, 0xaa, 0xbb]),
    };
    let old_key = native::method_id("echo_account");
    let new_key = native::method_id("echo_account_bytes");
    let request = native::encode_request(old_key, &old, LIMITS).unwrap();
    let response = native::encode_response(old_key, &old, LIMITS).unwrap();
    // An old 0x20 becomes the new byte length. The old blob's length 03
    // becomes the final subaccount byte; 02 becomes the new blob length.
    // A same-key decode succeeds with different values: malformed checks
    // cannot fence this cut. This is a test control, never a legacy reader.
    let changed = native::decode_request::<CurrentRequest>(old_key, &request, LIMITS).unwrap();
    let mut new_subaccount = [0; 32];
    new_subaccount[31] = 3;
    assert_eq!(changed.account.owner, old.account.owner);
    assert_eq!(changed.account.subaccount, Some(new_subaccount));
    assert_eq!(changed.blob.as_ref(), &[0xaa, 0xbb]);
    let no_payload_work = Limits { nodes: 0, ..LIMITS };
    assert_eq!(
        native::decode_request::<CurrentRequest>(new_key, &request, no_payload_work)
            .unwrap_err(),
        ic_wire::Error::Identity
    );
    assert_eq!(
        native::decode_response::<CurrentRequest>(new_key, &response, no_payload_work)
            .unwrap_err(),
        ic_wire::Error::Identity
    );
}
