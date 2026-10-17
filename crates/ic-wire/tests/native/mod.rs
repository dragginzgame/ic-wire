//! Native Serde admission, wire compatibility and failure boundaries.
use ic_wire::{Error, Limits, Method, native};
use serde::{Deserialize, Serialize};

const LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 64 * 1024,
    nodes: 4096,
    depth: 64,
};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Tree {
    Leaf(String, u128),
    Branch(Vec<Self>),
}
#[derive(Serialize)]
struct Borrowed<'a> {
    label: &'a str,
    values: &'a [u64],
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Owned {
    label: String,
    values: Vec<u64>,
}
struct Scalar;
impl Method for Scalar {
    type Request = u64;
    type Response = u64;
    const ID: u64 = 7;
    const NAME: &'static str = "scalar";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

#[test]
fn borrowed_frames_match_owned_postcard_and_finite_wire() {
    let value = Borrowed {
        label: "é🦀",
        values: &[0, 128, u64::MAX],
    };
    let raw = postcard::to_allocvec(&value).unwrap();
    assert_eq!(native::encode(&value, LIMITS).unwrap(), raw);
    let expected = Owned {
        label: value.label.into(),
        values: value.values.to_vec(),
    };
    assert_eq!(native::decode::<Owned>(&raw, LIMITS).unwrap(), expected);
    let frame = native::encode_request(7, &value, LIMITS).unwrap();
    assert_eq!(&frame[11..], raw);
    assert_eq!(
        native::encoded_frame_len(&value, LIMITS).unwrap(),
        frame.len()
    );
    assert_eq!(native::encoded_len(&value, LIMITS).unwrap(), raw.len());
    assert_eq!(
        native::decode_request::<Owned>(7, &frame, LIMITS).unwrap(),
        expected
    );
    assert_eq!(
        native::decode_response::<Owned>(7, &frame, LIMITS),
        Err(Error::Identity)
    );
    assert_eq!(
        native::decode_request::<Owned>(8, &frame, LIMITS),
        Err(Error::Identity)
    );
    let response = native::encode_response(7, &value, LIMITS).unwrap();
    assert_eq!(
        native::decode_response::<Owned>(7, &response, LIMITS).unwrap(),
        expected
    );
    for n in [0, 127, 128, u64::MAX] {
        assert_eq!(
            native::encode_request(7, &n, LIMITS).unwrap(),
            ic_wire::encode_request::<Scalar>(&n).unwrap().as_bytes()
        );
        assert_eq!(
            native::encode_response(7, &n, LIMITS).unwrap(),
            ic_wire::encode_response::<Scalar>(&n).unwrap().as_bytes()
        );
    }
    for end in 0..frame.len() {
        assert!(native::decode_request::<Owned>(7, &frame[..end], LIMITS).is_err());
    }
    for index in 0..frame.len() {
        for mask in [1, 0x80, 0xff] {
            let mut changed = frame.clone();
            changed[index] ^= mask;
            if let Ok(value) = native::decode_request::<Owned>(7, &changed, LIMITS) {
                assert_eq!(native::encode_request(7, &value, LIMITS).unwrap(), changed);
            }
        }
    }
    let mut trailing = frame;
    trailing.push(0);
    assert_eq!(
        native::decode_request::<Owned>(7, &trailing, LIMITS),
        Err(Error::Trailing)
    );
    assert_eq!(
        native::decode::<u64>(&[0x80, 0], LIMITS),
        Err(Error::Noncanonical)
    );
    assert_eq!(
        native::decode::<String>(&[1, 0xff], LIMITS),
        Err(Error::Malformed)
    );
}

#[test]
fn recursive_and_zero_byte_values_cannot_bypass_budgets() {
    let mut tree = Tree::Leaf("leaf".into(), u128::MAX);
    let mut admitted = 0;
    for _ in 0..40 {
        let raw = postcard::to_allocvec(&tree).unwrap();
        match native::encode(&tree, LIMITS) {
            Ok(bytes) => {
                admitted += 1;
                assert_eq!(bytes, raw);
                assert_eq!(native::decode::<Tree>(&bytes, LIMITS).unwrap(), tree);
            }
            Err(error) => assert_eq!(error, Error::Limit),
        }
        tree = Tree::Branch(vec![tree]);
    }
    assert!(admitted > 0 && admitted < 40);
    assert_eq!(
        native::decode::<Tree>(&postcard::to_allocvec(&tree).unwrap(), LIMITS),
        Err(Error::Limit)
    );
    let limits = Limits {
        nodes: 16,
        ..LIMITS
    };
    let units = vec![(); 100];
    assert_eq!(native::encode(&units, limits), Err(Error::Limit));
    assert_eq!(
        native::decode::<Vec<()>>(&postcard::to_allocvec(&units).unwrap(), limits),
        Err(Error::Limit)
    );
    let forged = postcard::to_allocvec(&u64::MAX).unwrap();
    assert!(native::decode::<Vec<()>>(&forged, limits).is_err());
    assert!(native::decode::<Vec<u128>>(&forged, limits).is_err());
    assert!(native::decode::<String>(&forged, limits).is_err());
}

#[test]
fn every_allowance_is_enforced_and_empty_values_remain_valid() {
    let map = std::collections::BTreeMap::from([
        (
            "ok".to_owned(),
            Some(Ok::<_, (u8, String)>(vec![i64::MIN, 0, i64::MAX])),
        ),
        ("err".to_owned(), Some(Err((7, "reason".to_owned())))),
        ("none".to_owned(), None),
    ]);
    let bytes = native::encode(&map, LIMITS).unwrap();
    assert_eq!(bytes, postcard::to_allocvec(&map).unwrap());
    assert_eq!(native::decode::<std::collections::BTreeMap<String, Option<Result<Vec<i64>, (u8, String)>>>>(&bytes, LIMITS).unwrap(), map);
    // Deserialization alone normalizes unordered and duplicate map keys; native
    // admission must reject both instead of accepting alternate hash/retry bytes.
    for bytes in [[2, 2, 20, 1, 10], [2, 1, 10, 1, 20]] {
        assert_eq!(
            native::decode::<std::collections::BTreeMap<u8, u8>>(&bytes, LIMITS),
            Err(Error::Noncanonical)
        );
    }
    let text = "x".repeat(128);
    let raw = postcard::to_allocvec(&text).unwrap();
    for limits in [
        Limits { bytes: 1, ..LIMITS },
        Limits {
            allocation: 8,
            ..LIMITS
        },
        Limits { nodes: 0, ..LIMITS },
        Limits { depth: 0, ..LIMITS },
    ] {
        assert_eq!(native::encode(&text, limits), Err(Error::Limit));
        assert_eq!(native::decode::<String>(&raw, limits), Err(Error::Limit));
    }
    assert_eq!(native::encode(&(), LIMITS).unwrap(), Vec::<u8>::new());
    native::decode::<()>(&[], LIMITS).unwrap();
    for bytes in 0..11 {
        let limits = Limits { bytes, ..LIMITS };
        assert_eq!(native::encode_request(7, &(), limits), Err(Error::Limit));
        assert_eq!(native::encoded_frame_len(&(), limits), Err(Error::Limit));
    }
    let exact = Limits {
        bytes: raw.len() + 11,
        ..LIMITS
    };
    assert_eq!(
        native::encode_request(7, &text, exact).unwrap().len(),
        exact.bytes
    );
    assert_eq!(
        native::encode_request(
            7,
            &text,
            Limits {
                bytes: exact.bytes - 1,
                ..exact
            }
        ),
        Err(Error::Limit)
    );
    let floats = [-0.0f64, f64::from_bits(1), f64::MAX];
    let bytes = native::encode(&floats, LIMITS).unwrap();
    let decoded: [f64; 3] = native::decode(&bytes, LIMITS).unwrap();
    assert_eq!(decoded.map(f64::to_bits), floats.map(f64::to_bits));
}

#[test]
fn frame_budgets_agree_at_header_boundary_and_preserve_error_precedence() {
    let exact = Limits {
        bytes: 11,
        allocation: 11,
        ..LIMITS
    };
    let request = native::encode_request(7, &(), exact).unwrap();
    let response = native::encode_response(7, &(), exact).unwrap();
    assert_eq!(native::encoded_frame_len(&(), exact), Ok(11));
    assert_eq!(native::decode_request::<()>(7, &request, exact), Ok(()));
    assert_eq!(native::decode_response::<()>(7, &response, exact), Ok(()));
    for short in 0..11 {
        let bytes = Limits {
            bytes: short,
            ..exact
        };
        assert_eq!(native::encoded_frame_len(&(), bytes), Err(Error::Limit));
        assert_eq!(native::encode_request(7, &(), bytes), Err(Error::Limit));
        assert_eq!(
            native::decode_request::<()>(7, &request, bytes),
            Err(Error::Limit)
        );
        let allocation = Limits {
            allocation: short,
            ..exact
        };
        assert_eq!(
            native::encoded_frame_len(&(), allocation),
            Err(Error::Limit)
        );
        assert_eq!(
            native::encode_response(7, &(), allocation),
            Err(Error::Limit)
        );
        assert_eq!(
            native::decode_request::<()>(7, &request, allocation),
            Err(Error::Limit)
        );
        // Identity and malformed-header checks still precede allocation admission.
        assert_eq!(
            native::decode_request::<()>(8, &request, allocation),
            Err(Error::Identity)
        );
        assert_eq!(
            native::decode_response::<()>(7, &request, allocation),
            Err(Error::Identity)
        );
        assert_eq!(
            native::decode_request::<()>(7, &request[..short], allocation),
            Err(Error::Malformed)
        );
    }
}
