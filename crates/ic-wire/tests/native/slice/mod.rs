use ic_wire::{Error, Limits, native};
use serde::Serialize;
use std::cell::Cell;

const LIMITS: Limits = Limits {
    bytes: 65536,
    allocation: 4 * 1024 * 1024,
    nodes: 65536,
    depth: 64,
};

fn compare<T: Serialize + ?Sized>(value: &T) {
    for direction in 0..3 {
        let expected = match direction {
            0 => native::encode(value, LIMITS),
            1 => native::encode_request(7, value, LIMITS),
            _ => native::encode_response(7, value, LIMITS),
        }
        .unwrap();
        for extra in [0, 1, 64] {
            let mut storage = vec![0xa5; expected.len() + extra];
            let encoded = match direction {
                0 => native::encode_to_slice(value, LIMITS, &mut storage),
                1 => native::encode_request_to_slice(7, value, LIMITS, &mut storage),
                _ => native::encode_response_to_slice(7, value, LIMITS, &mut storage),
            }
            .unwrap();
            assert_eq!(encoded, expected);
            assert!(storage[expected.len()..].iter().all(|b| *b == 0xa5));
        }
        if !expected.is_empty() {
            let mut storage = vec![0xa5; expected.len() - 1];
            let result = match direction {
                0 => native::encode_to_slice(value, LIMITS, &mut storage),
                1 => native::encode_request_to_slice(7, value, LIMITS, &mut storage),
                _ => native::encode_response_to_slice(7, value, LIMITS, &mut storage),
            };
            assert_eq!(result, Err(Error::Limit));
            assert!(storage.iter().all(|b| *b == 0xa5));
        }
    }
}

#[test]
fn caller_slice_matches_owned_payload_and_both_frame_directions() {
    compare(&());
    compare(&7_u64);
    for length in [0, 1, 128, 4096] {
        compare(&vec![u32::MAX; length]);
        compare(&ic_wire::ByteBuf::from(vec![0xa5; length]));
    }
    #[derive(Serialize)]
    struct Record<'a> {
        name: &'a str,
        values: Option<Result<Vec<i16>, ()>>,
    }
    compare(&Record {
        name: "é🦀",
        values: Some(Ok(vec![i16::MIN, 0, i16::MAX])),
    });
}

#[test]
fn caller_slice_refuses_limits_before_writing() {
    for limits in [
        Limits { bytes: 0, ..LIMITS },
        Limits {
            allocation: 0,
            ..LIMITS
        },
        Limits { nodes: 0, ..LIMITS },
        Limits { depth: 0, ..LIMITS },
    ] {
        let mut storage = [0xa5; 32];
        assert_eq!(
            native::encode_to_slice(&7_u64, limits, &mut storage),
            Err(Error::Limit)
        );
        assert_eq!(storage, [0xa5; 32]);
        assert_eq!(
            native::encode_response_to_slice(7, &7_u64, limits, &mut storage),
            Err(Error::Limit)
        );
        assert_eq!(storage, [0xa5; 32]);
    }
}

#[test]
fn caller_slice_never_returns_partial_output_after_a_second_pass_failure() {
    struct Changed(Cell<bool>);
    impl Serialize for Changed {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            if self.0.replace(true) {
                serializer.serialize_bytes(&[1, 2, 3])
            } else {
                serializer.serialize_u8(7)
            }
        }
    }
    let mut storage = [0xa5; 32];
    assert_eq!(
        native::encode_to_slice(&Changed(Cell::new(false)), LIMITS, &mut storage),
        Err(Error::Malformed)
    );
    assert_eq!(
        native::encode_response_to_slice(7, &Changed(Cell::new(false)), LIMITS, &mut storage),
        Err(Error::Malformed)
    );
}
