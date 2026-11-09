#[test]
fn packed_domain_transport_keeps_validation_and_rebuilds_untrusted_heap_order() {
    use super::*;
    use crate::fixtures::LIMITS;
    // Explicit layout change, not an automatic replacement of existing Serde:
    // [-1_i64] changes 01 01 -> 08 ff ff ff ff ff ff ff ff.
    let old = vec![NonZeroI64::new(-1).unwrap()];
    assert_eq!(ic_wire::native::encode(&old, LIMITS).unwrap(), [1, 1]);
    let packed = nonzero_to_wire(&old);
    assert_eq!(
        ic_wire::native::encode(&packed, LIMITS).unwrap(),
        [8, 255, 255, 255, 255, 255, 255, 255, 255]
    );
    assert_eq!(nonzero_from_wire(packed).unwrap(), old);
    for values in [vec![0], vec![1, 0], vec![i64::MIN, i64::MAX, 0]] {
        let bytes = ic_wire::native::encode(&Packed::new(values), LIMITS).unwrap();
        let packed = ic_wire::native::decode(&bytes, LIMITS).unwrap();
        assert!(nonzero_from_wire(packed).is_none());
    }
    assert_eq!(nonzero_from_wire(Packed::new(vec![])), Some(vec![]));
    let bytes = ic_wire::native::encode(&Packed::new(vec![1_u64, 7, 2, 7, 0]), LIMITS).unwrap();
    let heap = heap_from_wire(ic_wire::native::decode(&bytes, LIMITS).unwrap());
    assert_eq!(heap.peek(), Some(&7));
    assert_eq!(heap.clone().into_sorted_vec(), [0, 1, 2, 7, 7]);
    assert_eq!(
        heap_from_wire(heap_to_wire(&heap)).into_sorted_vec(),
        [0, 1, 2, 7, 7]
    );
    for end in 0..bytes.len() {
        assert!(ic_wire::native::decode::<Packed<u64>>(&bytes[..end], LIMITS).is_err());
    }
    assert_eq!(
        ic_wire::native::decode::<Packed<u64>>(
            &bytes,
            ic_wire::Limits {
                allocation: 0,
                ..LIMITS
            }
        ),
        Err(ic_wire::Error::Limit)
    );
}
