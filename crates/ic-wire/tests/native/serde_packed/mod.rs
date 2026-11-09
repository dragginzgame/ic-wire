use super::LIMITS;
use ic_wire::Packed;
use serde::{Deserialize, Serialize};
use std::{collections::BinaryHeap, num::NonZeroI64};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct NonzeroValue(#[serde(with = "ic_wire::serde_packed::nonzero_i64")] Vec<NonZeroI64>);
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct HeapValue(#[serde(with = "ic_wire::serde_packed::heap_u64")] BinaryHeap<u64>);

#[test]
fn bool_adapter_handles_nested_fields_and_rejects_invalid_bytes() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Flags {
        tag: u8,
        #[serde(with = "ic_wire::serde_packed::bools")]
        flags: Vec<bool>,
        tail: bool,
    }
    let value = Flags {
        tag: 7,
        flags: vec![false, true],
        tail: true,
    };
    let bytes = ic_wire::native::encode(&value, LIMITS).unwrap();
    assert_eq!(bytes, [7, 2, 0, 1, 1]);
    assert_eq!(
        ic_wire::native::decode::<Flags>(&bytes, LIMITS).unwrap(),
        value
    );
    for invalid in 2..=255 {
        assert_eq!(
            ic_wire::native::decode::<Flags>(&[7, 2, 0, invalid, 1], LIMITS),
            Err(ic_wire::Error::Malformed)
        );
    }
}

#[test]
fn adapters_preserve_packed_bytes_and_admission_boundaries() {
    fn compare<
        T: Serialize + serde::de::DeserializeOwned,
        U: Serialize + serde::de::DeserializeOwned,
    >(
        a: &T,
        b: &U,
    ) {
        let bytes = ic_wire::native::encode(a, LIMITS).unwrap();
        assert_eq!(bytes, ic_wire::native::encode(b, LIMITS).unwrap());
        let mut limits = Vec::new();
        for bytes in 0..=bytes.len() + 1 {
            limits.push(ic_wire::Limits { bytes, ..LIMITS });
        }
        // Find the actual reference transition. A tiny allowance with the
        // corpus's 2 MiB byte reservation would only test unconditional refusal.
        for encode in [false, true] {
            let base = ic_wire::Limits {
                bytes: bytes.len(),
                ..LIMITS
            };
            let admitted = |allocation| {
                let limit = ic_wire::Limits { allocation, ..base };
                if encode {
                    ic_wire::native::encode(b, limit).is_ok()
                } else {
                    ic_wire::native::decode::<U>(&bytes, limit).is_ok()
                }
            };
            let (mut low, mut high) = (0, base.allocation);
            assert!(!admitted(low) && admitted(high));
            while low + 1 < high {
                let middle = low + (high - low) / 2;
                if admitted(middle) {
                    high = middle;
                } else {
                    low = middle;
                }
            }
            for allocation in [0, low, high, high + 1] {
                limits.push(ic_wire::Limits { allocation, ..base });
            }
        }
        for nodes in 0..16 {
            limits.push(ic_wire::Limits { nodes, ..LIMITS });
        }
        for depth in 0..8 {
            limits.push(ic_wire::Limits { depth, ..LIMITS });
        }
        for limit in limits {
            assert_eq!(
                ic_wire::native::encode(a, limit),
                ic_wire::native::encode(b, limit),
                "encode: {limit:?}"
            );
            assert_eq!(
                ic_wire::native::decode::<T>(&bytes, limit).map(|_| ()),
                ic_wire::native::decode::<U>(&bytes, limit).map(|_| ()),
                "decode: {limit:?}"
            );
        }
        for end in 0..bytes.len() {
            assert!(ic_wire::native::decode::<T>(&bytes[..end], LIMITS).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert_eq!(
            ic_wire::native::decode::<T>(&trailing, LIMITS).err(),
            Some(ic_wire::Error::Trailing)
        );
    }
    for n in [0, 1, 129] {
        let values = (0..n)
            .map(|i| NonZeroI64::new([i64::MIN, -1, i64::MAX][i % 3]).unwrap())
            .collect::<Vec<_>>();
        let packed = Packed::new(values.iter().map(|value| value.get()).collect());
        compare(&NonzeroValue(values), &packed);
        let heap = (0..n as u64).collect::<BinaryHeap<_>>();
        let packed = Packed::new(heap.as_slice().to_vec());
        compare(&HeapValue(heap), &packed);
    }
}

#[test]
fn adapter_decode_checks_zero_and_reconstructs_untrusted_heap_order() {
    for values in [vec![0_i64], vec![1, 0], vec![i64::MIN, i64::MAX, 0]] {
        let bytes = ic_wire::native::encode(&Packed::new(values), LIMITS).unwrap();
        assert_eq!(
            ic_wire::native::decode::<NonzeroValue>(&bytes, LIMITS),
            Err(ic_wire::Error::Malformed)
        );
    }
    let bytes = ic_wire::native::encode(&Packed::new(vec![1_u64, 7, 2, 7, 0]), LIMITS).unwrap();
    let heap: HeapValue = ic_wire::native::decode(&bytes, LIMITS).unwrap();
    assert_eq!(heap.0.into_sorted_vec(), [0, 1, 2, 7, 7]);
    let valid = ic_wire::native::encode(&Packed::new(vec![-1_i64]), LIMITS).unwrap();
    let decoded: NonzeroValue = ic_wire::native::decode(&valid, LIMITS).unwrap();
    assert_eq!(decoded.0[0].get(), -1);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn field_adapter_bytes_match_but_encode_admission_needs_consumer_review() {
    #[derive(Serialize, Deserialize)]
    struct Adapted {
        tag: u64,
        #[serde(with = "ic_wire::serde_packed::nonzero_i64")]
        words: Vec<NonZeroI64>,
    }
    #[derive(Serialize, Deserialize)]
    struct Dto {
        tag: u64,
        words: Packed<i64>,
    }
    let a = Adapted {
        tag: 1,
        words: vec![NonZeroI64::new(-1).unwrap()],
    };
    let b = Dto {
        tag: 1,
        words: Packed::new(vec![-1]),
    };
    let bytes = ic_wire::native::encode(&a, LIMITS).unwrap();
    assert_eq!(bytes, ic_wire::native::encode(&b, LIMITS).unwrap());
    let minimum = |admit: &dyn Fn(ic_wire::Limits) -> bool| {
        (0..4096)
            .find(|&allocation| {
                admit(ic_wire::Limits {
                    bytes: bytes.len(),
                    allocation,
                    ..LIMITS
                })
            })
            .unwrap()
    };
    let encode_a = minimum(&|l| ic_wire::native::encode(&a, l).is_ok());
    let encode_b = minimum(&|l| ic_wire::native::encode(&b, l).is_ok());
    let decode_a = minimum(&|l| ic_wire::native::decode::<Adapted>(&bytes, l).is_ok());
    let decode_b = minimum(&|l| ic_wire::native::decode::<Dto>(&bytes, l).is_ok());
    // On this 64-bit host, Serde's serialize-with reference wrapper is smaller
    // than the owned Packed field. Same bytes do not promise the same admission.
    assert_eq!(
        (encode_a, encode_b, decode_a, decode_b),
        (842, 1098, 842, 842)
    );
}

#[cfg(feature = "typescript")]
#[test]
fn metadata_constraints_are_validated_and_do_not_change_plain_packed() {
    use ic_wire::typescript::{self, GenerationError};
    use serde_json::json;
    let types = Default::default();
    for schema in [
        json!({"k":"packed","word":"i64","nonzero":true}),
        json!({"k":"packed","word":"i64"}),
    ] {
        typescript::validate_payload_schema(&types, &schema).unwrap();
    }
    for schema in [
        json!({"k":"packed","word":"u64","nonzero":true}),
        json!({"k":"packed","word":"i64","nonzero":false}),
        json!({"k":"packed","word":"i64","nonzero":"true"}),
        json!({"k":"list","item":{"k":"i","bits":64},"nonzero":true}),
    ] {
        assert_eq!(
            typescript::validate_payload_schema(&types, &schema),
            Err(GenerationError::Schema)
        );
    }
    let zero = ic_wire::native::encode(&Packed::new(vec![0_i64]), LIMITS).unwrap();
    assert_eq!(
        ic_wire::native::decode::<Packed<i64>>(&zero, LIMITS)
            .unwrap()
            .as_slice(),
        [0]
    );
}

#[test]
fn finite_adapters_preserve_bits_and_reject_nonfinite_partial_and_truncated_words() {
    macro_rules! check {
        ($ty:ty, $bits:ty, $adapter:literal, $width:expr, $bad:expr) => {{
            #[derive(Debug, Serialize, Deserialize)]
            #[serde(transparent)]
            struct Values(#[serde(with = $adapter)] Vec<$ty>);
            for count in [0, 1, 7, 8, 9, 15, 16, 17, 31, 32, 127, 128] {
                let values: Vec<$ty> = (0..count)
                    .map(|i| {
                        [
                            0.0,
                            -0.0,
                            <$ty>::from_bits(1),
                            -<$ty>::from_bits(1),
                            <$ty>::MAX,
                            -<$ty>::MAX,
                            <$ty>::MIN_POSITIVE,
                            1.0,
                        ][i % 8]
                    })
                    .collect();
                let raw: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
                let encoded = ic_wire::native::encode(&Values(values.clone()), LIMITS).unwrap();
                assert_eq!(
                    encoded,
                    ic_wire::native::encode(&ic_wire::ByteBuf::from(raw), LIMITS).unwrap()
                );
                for offset in 0..16 {
                    let mut padded = vec![0; offset];
                    padded.extend_from_slice(&encoded);
                    let decoded: Values =
                        ic_wire::native::decode(&padded[offset..], LIMITS).unwrap();
                    assert_eq!(
                        decoded.0.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                        values.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
                    );
                }
                for end in 0..encoded.len() {
                    assert!(ic_wire::native::decode::<Values>(&encoded[..end], LIMITS).is_err());
                }
                for bad in $bad {
                    assert!(
                        ic_wire::native::encode(&Values(vec![<$ty>::from_bits(bad)]), LIMITS)
                            .is_err()
                    );
                    for position in [0, count / 2, count.saturating_sub(1)] {
                        if count == 0 {
                            continue;
                        }
                        let mut words = values.clone();
                        words[position] = <$ty>::from_bits(bad);
                        let bytes: Vec<u8> = words.iter().flat_map(|v| v.to_le_bytes()).collect();
                        let bytes = ic_wire::native::encode(&ic_wire::ByteBuf::from(bytes), LIMITS)
                            .unwrap();
                        assert_eq!(
                            ic_wire::native::decode::<Values>(&bytes, LIMITS).err(),
                            Some(ic_wire::Error::Malformed)
                        );
                    }
                }
            }
            for length in 1..$width {
                let bytes =
                    ic_wire::native::encode(&ic_wire::ByteBuf::from(vec![0; length]), LIMITS)
                        .unwrap();
                assert_eq!(
                    ic_wire::native::decode::<Values>(&bytes, LIMITS).err(),
                    Some(ic_wire::Error::Malformed)
                );
            }
            let value = Values(vec![1.0, -0.0]);
            let bytes = ic_wire::native::encode(&value, LIMITS).unwrap();
            for which in 0..4 {
                let limit = |n| {
                    let mut limits = ic_wire::Limits {
                        bytes: bytes.len(),
                        ..LIMITS
                    };
                    match which {
                        0 => limits.bytes = n,
                        1 => limits.allocation = n,
                        2 => limits.nodes = n,
                        _ => limits.depth = n,
                    }
                    limits
                };
                let exact = (0..4096)
                    .find(|&n| ic_wire::native::decode::<Values>(&bytes, limit(n)).is_ok())
                    .unwrap();
                assert!(exact > 0);
                assert_eq!(
                    ic_wire::native::decode::<Values>(&bytes, limit(exact - 1)).err(),
                    Some(ic_wire::Error::Limit)
                );
                let exact = (0..4096)
                    .find(|&n| ic_wire::native::encode(&value, limit(n)).is_ok())
                    .unwrap();
                assert!(exact > 0);
                assert_eq!(
                    ic_wire::native::encode(&value, limit(exact - 1)),
                    Err(ic_wire::Error::Limit)
                );
            }
        }};
    }
    check!(
        f32,
        u32,
        "ic_wire::serde_packed::finite_f32",
        4,
        [0x7f80_0000_u32, 0xff80_0000, 0x7fc0_0001, 0x7f80_0001]
    );
    check!(
        f64,
        u64,
        "ic_wire::serde_packed::finite_f64",
        8,
        [
            0x7ff0_0000_0000_0000_u64,
            0xfff0_0000_0000_0000,
            0x7ff8_0000_0000_0001,
            0x7ff0_0000_0000_0001
        ]
    );
}
