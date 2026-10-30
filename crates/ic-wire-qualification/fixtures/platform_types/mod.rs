//! Standalone consumer proof transferred from Platform commit 421edbc.
use ic_platform::Principal;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(not(target_family = "wasm"), derive(ic_wire::Wire))]
pub struct Request {
    pub caller: Principal,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(not(target_family = "wasm"), derive(ic_wire::Wire))]
pub struct AccountRequest {
    pub account: ic_platform::Account,
}

#[cfg(feature = "endpoint")]
#[ic_wire::endpoint::query(contract = ACCOUNT_CONTRACT)]
pub fn echo_account(value: AccountRequest) -> AccountRequest {
    value
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
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
        accepts::<ic_platform::Principal>();
        accepts::<ic_platform::Account>();
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
            let method = serde_json::json!({
                "name": "principal", "mode": "query", "id": "17", "request": schema,
                "response": schema, "requestBytes": 512, "responseBytes": 512,
                "requestNodes": 1024, "responseNodes": 1024, "requestDepth": 16, "responseDepth": 16
            });
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
        use ic_platform::Account;
        assert_eq!(
            typescript::native_schema::<Account>().unwrap(),
            serde_json::json!({
                "k": "record", "fields": [
                    {"name": "owner", "value": {"k": "bytes", "max": 29}},
                    {"name": "subaccount", "value": {"k": "option", "item": {
                        "k": "array", "max": 32, "item": {"k": "u", "bits": 8}
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
                // Compare against the pre-existing field encoding, not Account equality.
                assert_eq!(
                    request,
                    native::encode_request(
                        18,
                        &(serde_bytes::Bytes::new(&raw), subaccount),
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
            let method = serde_json::json!({
                "name": "account", "mode": "query", "id": "18", "request": schema,
                "response": schema, "requestBytes": 512, "responseBytes": 512,
                "requestNodes": 1024, "responseNodes": 1024, "requestDepth": 16, "responseDepth": 16
            });
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
        // Build malformed field layouts with the canonical encoder. The fixed-array
        // subaccount has no length prefix; missing/extra elements must still refuse.
        let owner = serde_bytes::Bytes::new(&[4]);
        let valid = ic_platform::Account::from(Principal::anonymous());
        let mut cases = vec![
            (
                "short subaccount",
                account_frames(&(owner, 1_u8, [0_u8; 31])),
            ),
            (
                "long subaccount",
                account_frames(&(owner, 1_u8, [0_u8; 32], 0_u8)),
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

    #[cfg(feature = "endpoint")]
    #[test]
    fn combined_endpoint_uses_the_same_schema_trait() {
        assert_eq!(
            ACCOUNT_CONTRACT.request_schema().unwrap(),
            typescript::native_schema::<AccountRequest>().unwrap()
        );
        assert_eq!(
            ACCOUNT_CONTRACT.response_schema().unwrap(),
            typescript::native_schema::<AccountRequest>().unwrap()
        );
    }
}
