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
#[test]
fn nested_record_allowance_counts_value_visits_not_encoded_bytes() {
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Child {
        hash: [u8; 32],
        label: String,
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Directory {
        children: Vec<Child>,
    }
    for count in [0, 1, 32] {
        let value = Directory {
            children: (0..count)
                .map(|_| Child {
                    hash: [0; 32],
                    label: "child".into(),
                })
                .collect(),
        };
        // Each child is visited as a struct, an array, 32 scalars and a String.
        // String contents add a separate buffer charge. Root decode has no seed;
        // every nested field/element has both seed and visitor admission.
        let nested = size_of::<Vec<Child>>()
            + count * (size_of::<Child>() + size_of::<[u8; 32]>() + 32 + size_of::<String>());
        // Vec serialization uses collect_seq over &Child; decode owns Child.
        let encode_nested = nested - count * size_of::<Child>() + count * size_of::<&Child>();
        let encode_units = 2 * (size_of::<Directory>() + encode_nested) + 5 * count;
        let decode_units = size_of::<Directory>() + 2 * nested + 5 * count;
        let limits = ic_wire::Limits {
            bytes: 4096,
            allocation: 4096 + 8 * encode_units,
            nodes: 10000,
            depth: 32,
        };
        let bytes = ic_wire::native::encode(&value, limits).unwrap();
        assert_eq!(
            ic_wire::native::encode(
                &value,
                ic_wire::Limits {
                    allocation: limits.allocation - 1,
                    ..limits
                }
            ),
            Err(ic_wire::Error::Limit)
        );
        let decode_limits = ic_wire::Limits {
            allocation: 4096 + 8 * decode_units,
            ..limits
        };
        assert_eq!(
            ic_wire::native::decode::<Directory>(&bytes, decode_limits).unwrap(),
            value
        );
        assert_eq!(
            ic_wire::native::decode::<Directory>(
                &bytes,
                ic_wire::Limits {
                    allocation: decode_limits.allocation - 1,
                    ..decode_limits
                }
            ),
            Err(ic_wire::Error::Limit)
        );
    }
}
