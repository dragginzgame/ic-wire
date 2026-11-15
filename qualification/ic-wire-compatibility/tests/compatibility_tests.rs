//! Independent literal bytes, primitive rules and destination custody.
use serde::{Deserialize, Serialize};
use wire::{Direction, Encoding, Error, Limits};

fn limits() -> Limits {
    Limits {
        bytes: 128,
        allocation: 8192,
        nodes: 256,
        depth: 32,
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Choice {
    Ready,
    Value(u16),
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Record {
    number: u8,
    choice: Choice,
    optional: Option<u8>,
    bytes: wire::Bytes<3>,
    packed: wire::Packed<i16, 2>,
    ordinary: wire::List<i16, 2>,
}
fn value() -> Record {
    Record {
        number: 7,
        choice: Choice::Value(128),
        optional: Some(9),
        bytes: wire::Bytes::new(vec![0xaa, 0xbb]).unwrap(),
        packed: wire::Packed::new(vec![1, -1]).unwrap(),
        ordinary: wire::List::new(vec![1, -1]).unwrap(),
    }
}
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
fn representative_record_matches_independent_literals() {
    for encoding in encodings() {
        let expected = literal(encoding);
        assert_eq!(
            wire::encode(&value(), encoding, limits()).unwrap(),
            expected
        );
        assert_eq!(
            wire::decode::<Record>(&expected, encoding, limits()).unwrap(),
            value()
        );
        for length in 0..expected.len() {
            assert!(wire::decode::<Record>(&expected[..length], encoding, limits()).is_err());
        }
    }
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
            let expected = if limit.nodes == 0 || limit.bytes == 10 {
                Err(Error::Limit)
            } else {
                Ok(literal(encoding))
            };
            let mut output = vec![42; 64];
            let result = wire::encode_into(&value(), encoding, limit, &mut output)
                .map(|bytes| bytes.to_vec());
            assert_eq!(result, expected);
            assert_eq!(
                wire::encoded_len(&value(), encoding, limit),
                expected.as_ref().map(Vec::len).map_err(|e| *e)
            );
            if expected.is_err() {
                assert!(output.is_empty());
            }
            for capacity in 0..=32 {
                let mut storage = vec![0xa5; capacity];
                let result = wire::encode_to_slice(&value(), encoding, limit, &mut storage)
                    .map(|bytes| bytes.to_vec());
                if expected.is_err() || capacity < literal(encoding).len() {
                    assert_eq!(result, Err(Error::Limit));
                    assert!(storage.iter().all(|byte| *byte == 0xa5));
                } else {
                    assert_eq!(result, expected);
                }
            }
        }
    }
}
#[test]
fn all_u16_and_i16_encodings_follow_varint_and_zigzag_rules() {
    for bits in 0..=u16::MAX {
        let literal = if bits < 128 {
            vec![bits as u8]
        } else if bits < 16384 {
            vec![(bits as u8 & 127) | 128, (bits >> 7) as u8]
        } else {
            vec![
                (bits as u8 & 127) | 128,
                ((bits >> 7) as u8 & 127) | 128,
                (bits >> 14) as u8,
            ]
        };
        assert_eq!(
            wire::encode(&bits, Encoding::Payload, limits()).unwrap(),
            literal
        );
        assert_eq!(
            wire::decode::<u16>(&literal, Encoding::Payload, limits()).unwrap(),
            bits
        );
        let signed = ((bits >> 1) as i16) ^ -((bits & 1) as i16);
        assert_eq!(
            wire::encode(&signed, Encoding::Payload, limits()).unwrap(),
            literal
        );
        assert_eq!(
            wire::decode::<i16>(&literal, Encoding::Payload, limits()).unwrap(),
            signed
        );
    }
}
#[test]
fn short_primitives_follow_literal_acceptance_rules() {
    assert_eq!(
        wire::decode::<u16>(&[], Encoding::Payload, limits()),
        Err(Error::Malformed)
    );
    for first in 0..=u8::MAX {
        let boolean = match first {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Malformed),
        };
        assert_eq!(
            wire::decode::<bool>(&[first], Encoding::Payload, limits()),
            boolean
        );
        let single = if first < 128 {
            Ok(first as u16)
        } else {
            Err(Error::Malformed)
        };
        assert_eq!(
            wire::decode::<u16>(&[first], Encoding::Payload, limits()),
            single
        );
        for second in 0..=u8::MAX {
            let expected = if first < 128 {
                Err(Error::Trailing)
            } else if second >= 128 {
                Err(Error::Malformed)
            } else {
                Ok((first & 127) as u16 | ((second as u16) << 7))
            };
            assert_eq!(
                wire::decode::<u16>(&[first, second], Encoding::Payload, limits()),
                expected
            );
        }
    }
}
#[test]
fn borrowed_bytes_retain_the_input_span() {
    let input = [3, b'a', b'b', b'c'];
    let decoded =
        wire::decode::<wire::BytesRef<'_, 3>>(&input, Encoding::Payload, limits()).unwrap();
    assert_eq!(decoded.as_slice().as_ptr(), input[1..].as_ptr());
}
