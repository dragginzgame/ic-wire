//! Standard byte arrays retain fixed width while using bulk Serde bytes.
use super::LIMITS;
use ic_wire::{ByteArray, Error, native};

fn roundtrip<const N: usize>(value: [u8; N]) {
    let value = ByteArray::new(value);
    let payload = postcard::to_allocvec(&value).unwrap();
    assert_eq!(native::encode(&value, LIMITS).unwrap(), payload);
    assert_eq!(native::decode::<ByteArray<N>>(&payload, LIMITS), Ok(value));
    for response in [false, true] {
        let encode = |limits| {
            if response {
                native::encode_response(7, &value, limits)
            } else {
                native::encode_request(7, &value, limits)
            }
        };
        let decode = |bytes: &[u8], limits| {
            if response {
                native::decode_response::<ByteArray<N>>(7, bytes, limits)
            } else {
                native::decode_request::<ByteArray<N>>(7, bytes, limits)
            }
        };
        let frame = encode(LIMITS).unwrap();
        assert_eq!(&frame[11..], payload);
        assert_eq!(decode(&frame, LIMITS), Ok(value));
        for end in 0..frame.len() {
            assert!(decode(&frame[..end], LIMITS).is_err());
        }
        let mut trailing = frame.clone();
        trailing.push(0);
        assert_eq!(decode(&trailing, LIMITS), Err(Error::Trailing));
        for short in [
            ic_wire::Limits {
                bytes: frame.len() - 1,
                ..LIMITS
            },
            ic_wire::Limits {
                allocation: 0,
                ..LIMITS
            },
            ic_wire::Limits { nodes: 0, ..LIMITS },
            ic_wire::Limits { depth: 0, ..LIMITS },
        ] {
            assert_eq!(encode(short), Err(Error::Limit));
            assert_eq!(decode(&frame, short), Err(Error::Limit));
        }
        // Failure must not poison a subsequent valid decode.
        assert_eq!(decode(&frame, LIMITS), Ok(value));
    }
}

#[test]
fn byte_array_preserves_standard_bytes_frames_and_admission() {
    roundtrip([]);
    roundtrip([255]);
    roundtrip([0, 128, 255]);
    roundtrip([0x95; 32]);
    roundtrip([0x95; 127]);
    roundtrip([0x95; 128]);
    // Bulk bytes do not consume a node for every key byte. This intentionally
    // differs from ordinary-array admission when a consumer adopts ByteArray.
    let bulk = ic_wire::Limits { nodes: 8, ..LIMITS };
    let key = ByteArray::new([0x95; 32]);
    let bytes = native::encode(&key, bulk).unwrap();
    assert_eq!(native::decode::<ByteArray<32>>(&bytes, bulk), Ok(key));
    assert_eq!(native::encode(&[0x95_u8; 32], bulk), Err(Error::Limit));
    assert_eq!(
        native::decode::<[u8; 32]>(&[0x95; 32], bulk),
        Err(Error::Limit)
    );
    assert_eq!(native::encode(&[0_u8, 255], LIMITS).unwrap(), [0, 255]);
    assert_eq!(
        native::encode(&ByteArray::new([0, 255]), LIMITS).unwrap(),
        [2, 0, 255]
    );
    assert_eq!(native::encode(&None::<ByteArray<2>>, LIMITS).unwrap(), [0]);
    assert_eq!(
        native::encode(&Some(ByteArray::new([0, 255])), LIMITS).unwrap(),
        [1, 2, 0, 255]
    );
}

#[test]
fn byte_array_rejects_wrong_lengths_and_invalid_prefixes() {
    for bytes in [
        vec![],
        vec![1, 0],
        vec![3, 0, 128, 255],
        vec![2, 0],
        vec![0x82, 0, 0, 255],
        vec![0x80; 10],
        vec![0xff; 10],
    ] {
        assert_eq!(
            native::decode::<ByteArray<2>>(&bytes, LIMITS),
            Err(Error::Malformed),
            "{bytes:?}"
        );
    }
    assert_eq!(
        native::decode::<ByteArray<0>>(&[1, 0], LIMITS),
        Err(Error::Malformed)
    );
    assert_eq!(
        native::decode::<ByteArray<2>>(&[2, 0, 255], LIMITS),
        Ok(ByteArray::new([0, 255]))
    );
}

#[cfg(feature = "typescript")]
#[test]
fn byte_array_schema_and_adapter_keep_exact_width() {
    use ic_wire::typescript::native_schema;
    use serde_json::json;
    #[derive(serde::Serialize, serde::Deserialize, ic_wire::Wire)]
    struct Adapted {
        #[serde(with = "serde_bytes")]
        key: [u8; 2],
        #[serde(with = "serde_bytes")]
        optional: Option<ByteArray<2>>,
    }
    assert_eq!(
        native_schema::<ByteArray<0>>().unwrap(),
        json!({"k":"bytes","min":0,"max":0})
    );
    assert_eq!(
        native_schema::<ByteArray<128>>().unwrap(),
        json!({"k":"bytes","min":128,"max":128})
    );
    assert_eq!(
        native_schema::<[u8; 2]>().unwrap(),
        json!({"k":"array","max":2,"item":{"k":"u","bits":8}})
    );
    assert_eq!(
        native_schema::<Adapted>().unwrap(),
        json!({"k":"record","fields":[
            {"name":"key","value":{"k":"bytes","min":2,"max":2}},
            {"name":"optional","value":{"k":"option","item":{"k":"bytes","min":2,"max":2}}}
        ]})
    );
    for optional in [None, Some(ByteArray::new([0, 255]))] {
        let value = Adapted {
            key: [0, 255],
            optional,
        };
        let expected = (ByteArray::new(value.key), value.optional);
        let frame = native::encode_request(7, &value, LIMITS).unwrap();
        assert_eq!(frame, native::encode_request(7, &expected, LIMITS).unwrap());
        assert_eq!(
            native::decode_request::<Adapted>(7, &frame, LIMITS)
                .unwrap()
                .optional,
            optional
        );
    }
}
