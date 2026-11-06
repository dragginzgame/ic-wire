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
