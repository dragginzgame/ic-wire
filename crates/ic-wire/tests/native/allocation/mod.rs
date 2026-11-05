//! Public admission boundaries, independent of the budget's internal units.
use ic_wire::{Error, Limits, native};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Response {
    counts: [u64; 2],
}

#[test]
fn tiny_response_can_exceed_its_frame_size_in_allocation_charges() {
    let value = Response {
        counts: [u64::MAX, 1],
    };
    let limits = Limits {
        bytes: 64,
        allocation: 1024,
        nodes: 64,
        depth: 16,
    };
    let frame = native::encode_response(7, &value, limits).unwrap();
    assert_eq!(frame.len(), 22);
    assert_eq!(
        &frame[11..],
        &[255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 1]
    );

    // Reproduce the reported 512/1024 symptom with both traversal directions.
    let short = Limits {
        allocation: 512,
        ..limits
    };
    assert_eq!(native::encode_response(7, &value, short), Err(Error::Limit));
    assert_eq!(
        native::decode_response::<Response>(7, &frame, short),
        Err(Error::Limit)
    );
    assert_eq!(native::decode_response(7, &frame, limits), Ok(value));
}

#[test]
fn response_allocation_reserves_the_configured_byte_limit_and_weights_visits() {
    let value = Response {
        counts: [u64::MAX, 1],
    };
    // Encoding charges the 16-byte record, 16-byte array and two 8-byte fields
    // twice each: 96 source bytes x8 = 768 allocation bytes per pass.
    // Decoding charges record/array visitors and field seeds/visitors:
    // 16 + (16 + 16) + 2*(8 + 8) = 80 source bytes x8 = 640 allocation bytes.
    // These fixed-width value sizes are the same on native and Wasm targets.
    for bytes in [22, 23, 64] {
        let encode = Limits {
            bytes,
            allocation: bytes + 768,
            nodes: 64,
            depth: 16,
        };
        let frame = native::encode_response(7, &value, encode).unwrap();
        assert_eq!(frame.len(), 22);
        assert_eq!(native::encoded_frame_len(&value, encode), Ok(22));
        let short = Limits {
            allocation: encode.allocation - 1,
            ..encode
        };
        assert_eq!(native::encode_response(7, &value, short), Err(Error::Limit));
        assert_eq!(native::encoded_frame_len(&value, short), Err(Error::Limit));

        let decode = Limits {
            allocation: bytes + 640,
            ..encode
        };
        assert_eq!(
            native::decode_response(7, &frame, decode),
            Ok(Response {
                counts: value.counts,
            })
        );
        assert_eq!(
            native::decode_response::<Response>(
                7,
                &frame,
                Limits {
                    allocation: decode.allocation - 1,
                    ..decode
                }
            ),
            Err(Error::Limit)
        );
    }
}
