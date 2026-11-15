//! Independent comparison with the published, source-verified reference.

use candidate::{Direction, Encoding, Error, Limits};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

fn limits() -> Limits {
    Limits {
        bytes: 128,
        allocation: 8192,
        nodes: 256,
        depth: 32,
    }
}

fn old_limits(limits: Limits) -> reference::Limits {
    reference::Limits {
        bytes: limits.bytes,
        allocation: limits.allocation,
        nodes: limits.nodes,
        depth: limits.depth,
    }
}

fn old_error(error: reference::Error) -> Error {
    match error {
        reference::Error::Limit => Error::Limit,
        reference::Error::Allocation => Error::Allocation,
        reference::Error::Malformed => Error::Malformed,
        reference::Error::Trailing => Error::Trailing,
        reference::Error::Identity => Error::Identity,
    }
}

fn old_encode<T: Serialize + ?Sized>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    match encoding {
        Encoding::Payload => reference::native::encode(value, old_limits(limits)),
        Encoding::Frame {
            method,
            direction: Direction::Request,
        } => reference::native::encode_request(method, value, old_limits(limits)),
        Encoding::Frame {
            method,
            direction: Direction::Response,
        } => reference::native::encode_response(method, value, old_limits(limits)),
    }
    .map_err(old_error)
}

fn old_decode<'de, T: Deserialize<'de>>(
    input: &'de [u8],
    encoding: Encoding,
    limits: Limits,
) -> Result<T, Error> {
    match encoding {
        Encoding::Payload => reference::native::decode(input, old_limits(limits)),
        Encoding::Frame {
            method,
            direction: Direction::Request,
        } => reference::native::decode_request(method, input, old_limits(limits)),
        Encoding::Frame {
            method,
            direction: Direction::Response,
        } => reference::native::decode_response(method, input, old_limits(limits)),
    }
    .map_err(old_error)
}

fn compare_decode<T: for<'de> Deserialize<'de> + Debug + PartialEq>(
    input: &[u8],
    encoding: Encoding,
    limits: Limits,
) {
    assert_eq!(
        candidate::decode::<T>(input, encoding, limits),
        old_decode::<T>(input, encoding, limits),
        "input={input:?}, encoding={encoding:?}, limits={limits:?}"
    );
}

#[test]
fn exhaustive_short_primitive_inputs_keep_acceptance_and_errors() {
    compare_decode::<u16>(&[], Encoding::Payload, limits());
    for first in 0..=u8::MAX {
        compare_decode::<bool>(&[first], Encoding::Payload, limits());
        compare_decode::<u16>(&[first], Encoding::Payload, limits());
        compare_decode::<Option<u8>>(&[first], Encoding::Payload, limits());
        for second in 0..=u8::MAX {
            compare_decode::<u16>(&[first, second], Encoding::Payload, limits());
            compare_decode::<bool>(&[first, second], Encoding::Payload, limits());
        }
    }
}

#[test]
fn supplier_delegation_preserves_all_i16_and_u16_encodings() {
    for bits in 0..=u16::MAX {
        assert_eq!(
            candidate::encode(&bits, Encoding::Payload, limits()),
            old_encode(&bits, Encoding::Payload, limits())
        );
        let signed = bits as i16;
        assert_eq!(
            candidate::encode(&signed, Encoding::Payload, limits()),
            old_encode(&signed, Encoding::Payload, limits())
        );
    }
}

// Same consumer declarations, independently instantiated with each package's
// adapters. No reference implementation is copied into the candidate graph.
macro_rules! record {
    ($module:ident, $wire:ident) => {
        mod $module {
            use super::*;
            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            pub(super) enum Choice {
                Ready,
                Value(u16),
            }
            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            pub(super) struct Record {
                number: u8,
                choice: Choice,
                optional: Option<u8>,
                bytes: $wire::Bytes<3>,
                packed: $wire::PackedI16<2>,
                ordinary: $wire::List<i16, 2>,
            }
            pub(super) fn value() -> Record {
                Record {
                    number: 7,
                    choice: Choice::Value(128),
                    optional: Some(9),
                    bytes: $wire::Bytes::new(vec![0xaa, 0xbb]).unwrap(),
                    packed: $wire::PackedI16::new(vec![1, -1]).unwrap(),
                    ordinary: $wire::List::new(vec![1, -1]).unwrap(),
                }
            }
        }
    };
}
record!(new_record, candidate);
record!(old_record, reference);

const PAYLOAD: &[u8] = &[
    7, 1, 0x80, 1, 1, 9, 2, 0xaa, 0xbb, 4, 1, 0, 255, 255, 2, 2, 1,
];

fn encodings() -> [Encoding; 3] {
    [
        Encoding::Payload,
        Encoding::Frame {
            method: 7,
            direction: Direction::Request,
        },
        Encoding::Frame {
            method: 7,
            direction: Direction::Response,
        },
    ]
}

#[test]
fn reusable_and_slice_destinations_preserve_custody_and_bytes() {
    for encoding in encodings() {
        for limit in [
            limits(),
            Limits {
                nodes: 0,
                ..limits()
            },
            Limits {
                bytes: 10,
                ..limits()
            },
        ] {
            let expected = old_encode(&old_record::value(), encoding, limit);
            let mut new_output = vec![42; 64];
            let mut old_output = vec![42; 64];
            let new =
                candidate::encode_into(&new_record::value(), encoding, limit, &mut new_output)
                    .map(|bytes| bytes.to_vec());
            let old = match encoding {
                Encoding::Payload => reference::native::encode_into(
                    &old_record::value(),
                    old_limits(limit),
                    &mut old_output,
                ),
                Encoding::Frame {
                    method,
                    direction: Direction::Request,
                } => reference::native::encode_request_into(
                    method,
                    &old_record::value(),
                    old_limits(limit),
                    &mut old_output,
                ),
                Encoding::Frame {
                    method,
                    direction: Direction::Response,
                } => reference::native::encode_response_into(
                    method,
                    &old_record::value(),
                    old_limits(limit),
                    &mut old_output,
                ),
            }
            .map(|()| old_output.clone())
            .map_err(old_error);
            assert_eq!(new, expected);
            assert_eq!(old, expected);
            assert_eq!(new_output, old_output);
            if expected.is_err() {
                assert!(new_output.is_empty());
            }
            let old_length = match encoding {
                Encoding::Payload => {
                    reference::native::encoded_len(&old_record::value(), old_limits(limit))
                }
                Encoding::Frame { .. } => {
                    reference::native::encoded_frame_len(&old_record::value(), old_limits(limit))
                }
            }
            .map_err(old_error);
            assert_eq!(
                candidate::encoded_len(&new_record::value(), encoding, limit),
                old_length
            );
            for capacity in 0..=32 {
                let mut new_storage = vec![0xa5; capacity];
                let mut old_storage = vec![0xa5; capacity];
                let new = candidate::encode_to_slice(
                    &new_record::value(),
                    encoding,
                    limit,
                    &mut new_storage,
                )
                .map(|bytes| bytes.to_vec());
                let old = match encoding {
                    Encoding::Payload => reference::native::encode_to_slice(
                        &old_record::value(),
                        old_limits(limit),
                        &mut old_storage,
                    ),
                    Encoding::Frame {
                        method,
                        direction: Direction::Request,
                    } => reference::native::encode_request_to_slice(
                        method,
                        &old_record::value(),
                        old_limits(limit),
                        &mut old_storage,
                    ),
                    Encoding::Frame {
                        method,
                        direction: Direction::Response,
                    } => reference::native::encode_response_to_slice(
                        method,
                        &old_record::value(),
                        old_limits(limit),
                        &mut old_storage,
                    ),
                }
                .map(|bytes| bytes.to_vec())
                .map_err(old_error);
                assert_eq!(
                    new, old,
                    "capacity={capacity} encoding={encoding:?} limits={limit:?}"
                );
                if new.is_err() {
                    // These inputs are deterministic: errors are preflight or
                    // short-output refusals, whose storage must stay untouched.
                    assert!(new_storage.iter().all(|byte| *byte == 0xa5));
                    assert!(old_storage.iter().all(|byte| *byte == 0xa5));
                }
            }
        }
    }
}

#[test]
fn wider_integer_specialization_boundaries_match_supplier_delegation() {
    for value in [
        i64::MIN,
        i32::MIN as i64,
        -8193,
        -8192,
        -65,
        -64,
        -1,
        0,
        1,
        63,
        64,
        127,
        128,
        8191,
        8192,
        i32::MAX as i64,
        i64::MAX,
    ] {
        assert_eq!(
            candidate::encode(&value, Encoding::Payload, limits()),
            old_encode(&value, Encoding::Payload, limits())
        );
        if let Ok(value) = i32::try_from(value) {
            assert_eq!(
                candidate::encode(&value, Encoding::Payload, limits()),
                old_encode(&value, Encoding::Payload, limits())
            );
        }
    }
    for value in [0_u64, 1, 127, 128, 16383, 16384, u32::MAX as u64, u64::MAX] {
        assert_eq!(
            candidate::encode(&value, Encoding::Payload, limits()),
            old_encode(&value, Encoding::Payload, limits())
        );
        if let Ok(value) = u32::try_from(value) {
            assert_eq!(
                candidate::encode(&value, Encoding::Payload, limits()),
                old_encode(&value, Encoding::Payload, limits())
            );
        }
    }
}

fn literal(encoding: Encoding) -> Vec<u8> {
    let mut bytes = match encoding {
        Encoding::Payload => vec![],
        Encoding::Frame {
            direction: Direction::Request,
            ..
        } => vec![b'I', b'W', 7, 0, 0, 0, 0, 0, 0, 0, 0],
        Encoding::Frame {
            direction: Direction::Response,
            ..
        } => vec![b'I', b'W', 7, 0, 0, 0, 0, 0, 0, 0, 1],
    };
    bytes.extend_from_slice(PAYLOAD);
    bytes
}

#[test]
fn representative_record_matches_independent_literal_and_reference() {
    assert_eq!(
        size_of::<new_record::Record>(),
        size_of::<old_record::Record>()
    );
    for encoding in encodings() {
        let expected = literal(encoding);
        assert_eq!(
            candidate::encode(&new_record::value(), encoding, limits()).unwrap(),
            expected
        );
        assert_eq!(
            old_encode(&old_record::value(), encoding, limits()).unwrap(),
            expected
        );
        assert_eq!(
            candidate::decode::<new_record::Record>(&expected, encoding, limits()).unwrap(),
            new_record::value()
        );
        assert_eq!(
            old_decode::<old_record::Record>(&expected, encoding, limits()).unwrap(),
            old_record::value()
        );
    }
}

#[test]
fn representative_budget_sweeps_preserve_refusal_boundaries() {
    let cases = (0..=64)
        .map(|bytes| Limits { bytes, ..limits() })
        .chain((0..=1536).map(|allocation| Limits {
            allocation,
            ..limits()
        }))
        .chain((0..=128).map(|nodes| Limits { nodes, ..limits() }))
        .chain((0..=16).map(|depth| Limits { depth, ..limits() }));
    for limits in cases {
        for encoding in encodings() {
            assert_eq!(
                candidate::encode(&new_record::value(), encoding, limits),
                old_encode(&old_record::value(), encoding, limits),
                "encode {encoding:?} {limits:?}"
            );
            let input = literal(encoding);
            let new = candidate::decode::<new_record::Record>(&input, encoding, limits).map(|_| ());
            let old = old_decode::<old_record::Record>(&input, encoding, limits).map(|_| ());
            assert_eq!(new, old, "decode {encoding:?} {limits:?}");
        }
    }
}

#[test]
fn mutated_frames_preserve_failure_precedence() {
    let encoding = Encoding::Frame {
        method: 7,
        direction: Direction::Request,
    };
    let original = literal(encoding);
    for offset in 0..original.len() {
        for byte in 0..=u8::MAX {
            let mut input = original.clone();
            input[offset] = byte;
            for limit in [
                limits(),
                Limits {
                    nodes: 0,
                    ..limits()
                },
                Limits {
                    bytes: 10,
                    ..limits()
                },
            ] {
                let new =
                    candidate::decode::<new_record::Record>(&input, encoding, limit).map(|_| ());
                let old = old_decode::<old_record::Record>(&input, encoding, limit).map(|_| ());
                assert_eq!(new, old, "offset={offset} byte={byte} limits={limit:?}");
            }
        }
    }
    for length in 0..original.len() {
        let input = &original[..length];
        assert_eq!(
            candidate::decode::<new_record::Record>(input, encoding, limits()).map(|_| ()),
            old_decode::<old_record::Record>(input, encoding, limits()).map(|_| ())
        );
    }
}

#[test]
fn borrowed_spans_and_noncanonical_chars_match_reference() {
    let input = [3, b'a', b'b', b'c'];
    let new = candidate::decode::<candidate::BytesRef<'_, 3>>(&input, Encoding::Payload, limits())
        .unwrap();
    let old =
        old_decode::<reference::BytesRef<'_, 3>>(&input, Encoding::Payload, limits()).unwrap();
    assert_eq!(new.as_slice().as_ptr(), input[1..].as_ptr());
    assert_eq!(old.as_slice().as_ptr(), input[1..].as_ptr());
    compare_decode::<char>(&[2, b'a', b'b'], Encoding::Payload, limits());
}
