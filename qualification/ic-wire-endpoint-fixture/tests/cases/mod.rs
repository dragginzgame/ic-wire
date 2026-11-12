//! One endpoint scenario catalogue for installed and instrumented qualification.
use ic_testkit::pocket_ic::{RejectCode, RejectResponse};
use ic_wire::{ByteBuf, Limits, native};
use ic_wire_endpoint_fixture::{
    DEFAULT_BLOB, DEFAULT_MUTATE, LIMITS, NAMED, RESPONSE_ONLY, Request,
};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone)]
enum Expected {
    Reply(Vec<u8>),
    Count(u64),
    Trap,
    Reject,
}
#[derive(Clone)]
pub(super) struct Case {
    pub(super) method: &'static str,
    pub(super) mode: &'static str,
    pub(super) label: String,
    pub(super) id: u64,
    pub(super) limits: Limits,
    pub(super) frame: Vec<u8>,
    expected: Expected,
    pub(super) mutation: u64,
}
fn case<T: Serialize, R: Serialize>(
    method: &'static str,
    mode: &'static str,
    id: u64,
    limits: Limits,
    label: &str,
    request: &T,
    reply: &R,
) -> Case {
    Case {
        method,
        mode,
        id,
        limits,
        label: label.into(),
        frame: native::encode_request(id, request, limits).unwrap(),
        expected: Expected::Reply(native::encode_response(id, reply, limits).unwrap()),
        mutation: 0,
    }
}
pub(super) fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (method, id) in [("echo", 7), ("borrowed_echo", 17)] {
        for size in [0, 32] {
            let value = Request {
                value: 300,
                bytes: ic_wire::Bytes::new(vec![42; size]).unwrap(),
            };
            cases.push(case(
                method,
                "query",
                id,
                LIMITS,
                &format!("blob-{size}"),
                &value,
                &value,
            ));
        }
    }
    let mut count = case("count", "query", 8, LIMITS, "unit", &(), &0u64);
    count.expected = Expected::Count(0);
    cases.push(count);
    cases.push(case("unit-call", "update", 10, LIMITS, "unit", &(), &()));
    for value in [0, u64::MAX] {
        cases.push(case(
            "async_query",
            "query",
            11,
            LIMITS,
            &format!("value-{value}"),
            &value,
            &(value, true),
        ));
        cases.push(case(
            "stable-name",
            "query",
            NAMED.id,
            NAMED.request,
            &format!("value-{value}"),
            &value,
            &value,
        ));
    }
    cases.push(case(
        "response_only",
        "query",
        RESPONSE_ONLY.id,
        RESPONSE_ONLY.request,
        "scalar",
        &42u64,
        &42u64,
    ));
    let mut composite = case(
        "composite",
        "composite_query",
        12,
        LIMITS,
        "self-call",
        &(),
        &0u64,
    );
    composite.expected = Expected::Count(0);
    cases.push(composite);
    for (method, id) in [
        ("mutate", 9),
        ("async_mutate", 13),
        ("async_ready", 14),
        ("borrowed_mutate", 18),
    ] {
        for size in [0u32, 32, 116, 117] {
            let mut row = case(
                method,
                "update",
                id,
                LIMITS,
                &format!("reply-{size}"),
                &size,
                &ByteBuf::new(),
            );
            if id == 18 {
                row.frame = native::encode_request(
                    id,
                    &Request {
                        value: u64::from(size),
                        bytes: ic_wire::Bytes::new(vec![42; 32]).unwrap(),
                    },
                    LIMITS,
                )
                .unwrap();
            }
            if size == 117 {
                row.expected = Expected::Trap;
                row.mutation = if id == 13 { 10 } else { 0 };
            } else {
                row.expected = Expected::Reply(
                    native::encode_response(id, &ByteBuf::from(vec![42; size as usize]), LIMITS)
                        .unwrap(),
                );
                row.mutation = if id == 13 { 110 } else { 1 };
            }
            cases.push(row);
        }
    }
    for length in [0, 1, 2] {
        let mut row = case(
            "nodes",
            "query",
            15,
            LIMITS,
            &format!("nodes-{length}"),
            &vec![1u8; length],
            &(length as u64),
        );
        if length == 2 {
            row.expected = Expected::Trap;
        }
        cases.push(row);
    }
    for size in [0usize, 32, 65_522, 65_523] {
        let limits = Limits {
            bytes: 65_537,
            ..DEFAULT_BLOB.request
        };
        let value = ByteBuf::from(vec![42; size]);
        let mut blob = case(
            "default_blob",
            "query",
            DEFAULT_BLOB.id,
            limits,
            &format!("blob-{size}"),
            &value,
            &value,
        );
        blob.limits = DEFAULT_BLOB.request;
        if size == 65_523 {
            blob.expected = Expected::Trap;
        }
        cases.push(blob);
        let mut output = case(
            "default_mutate",
            "update",
            DEFAULT_MUTATE.id,
            limits,
            &format!("reply-{size}"),
            &(size as u32),
            &value,
        );
        output.limits = DEFAULT_MUTATE.request;
        if size == 65_523 {
            output.expected = Expected::Trap;
        } else {
            output.mutation = 1;
        }
        cases.push(output);
    }
    cases.push(case(
        "default_unit",
        "query",
        native::method_id("default_unit"),
        DEFAULT_BLOB.request,
        "unit",
        &(),
        &(),
    ));
    for allowed in [true, false] {
        let mut row = case(
            "composed",
            "update",
            16,
            LIMITS,
            if allowed { "allow" } else { "deny" },
            &allowed,
            &0u64,
        );
        row.expected = if allowed {
            Expected::Count(1)
        } else {
            Expected::Reject
        };
        row.mutation = u64::from(allowed);
        cases.push(row);
    }
    // Each actual export has a valid control plus identity/full-input/pre-copy
    // refusals. Future exports must be added here or the artifact coverage fails.
    let mut methods = BTreeSet::new();
    for base in cases.clone() {
        if !methods.insert(base.method) {
            continue;
        }
        for label in [
            "wrong-id",
            "wrong-magic",
            "wrong-direction",
            "short-header",
            "wrong-id-at-limit",
            "trailing",
            "oversized",
            "truncated",
        ] {
            let mut row = base.clone();
            row.label = label.into();
            row.expected = Expected::Trap;
            row.mutation = 0;
            match label {
                "wrong-id" => row.frame[2] ^= 1,
                "wrong-magic" => row.frame[0] ^= 1,
                "wrong-direction" => row.frame[10] = 1,
                "short-header" => row.frame.truncate(10),
                "wrong-id-at-limit" => {
                    row.frame.resize(row.limits.bytes, 0);
                    row.frame[2] ^= 1;
                }
                "trailing" => row.frame.push(0),
                "oversized" => row.frame = vec![0; row.limits.bytes + 1],
                _ => {
                    row.frame.pop();
                }
            }
            cases.push(row);
        }
    }
    cases
}

impl Case {
    // Transport/setup errors must never count as observed canister refusals.
    pub(super) fn verify(
        &self,
        result: Result<Vec<u8>, RejectResponse>,
        state: u64,
    ) -> (&'static str, Option<Vec<u8>>) {
        match (&self.expected, result) {
            (Expected::Reply(expected), Ok(reply)) => {
                assert_eq!(&reply, expected, "{} / {}", self.method, self.label);
                ("reply", Some(reply))
            }
            (Expected::Count(delta), Ok(reply)) => {
                assert_eq!(
                    native::decode_response::<u64>(self.id, &reply, self.limits).unwrap(),
                    state + delta
                );
                ("reply", Some(reply))
            }
            (Expected::Trap, Err(error)) => {
                assert_eq!(
                    error.reject_code,
                    RejectCode::CanisterError,
                    "{} / {}: {error:?}",
                    self.method,
                    self.label
                );
                ("trap", None)
            }
            (Expected::Reject, Err(error)) => {
                assert_eq!(error.reject_code, RejectCode::CanisterReject);
                ("reject", None)
            }
            (_, unexpected) => panic!("{} / {}: {unexpected:?}", self.method, self.label),
        }
    }
}
