#[test]
fn external_nat_evidence_preserves_overflow_and_rejects_nonminimal_magnitudes() {
    use super::{BigUint, Error, Magnitude, Nat, from_nat, to_nat};
    use ic_wire::native;
    let values = [
        None,
        Some(Nat::from(0_u8)),
        Some(Nat::from(u128::MAX)),
        Some(Nat(BigUint::from(1_u8) << 128)),
        Some(Nat(BigUint::from(1_u8) << 256)),
        Some(Nat(BigUint::from_bytes_be(&[255; 64]))),
    ];
    let magnitudes = [
        None,
        Some(vec![]),
        Some(vec![255; 16]),
        Some([vec![1], vec![0; 16]].concat()),
        Some([vec![1], vec![0; 32]].concat()),
        Some(vec![255; 64]),
    ];
    for (original, expected) in values.into_iter().zip(magnitudes) {
        let wire = original.as_ref().map(from_nat).transpose().unwrap();
        assert_eq!(
            wire.as_ref().map(|value| value.as_slice()),
            expected.as_deref()
        );
        let bytes = native::encode_request(26, &wire, crate::fixtures::LIMITS).unwrap();
        let decoded: Option<Magnitude> =
            native::decode_request(26, &bytes, crate::fixtures::LIMITS).unwrap();
        assert_eq!(decoded.as_ref().map(to_nat).transpose().unwrap(), original);
        for limits in [
            ic_wire::Limits {
                bytes: bytes.len() - 1,
                ..crate::fixtures::LIMITS
            },
            ic_wire::Limits {
                allocation: 0,
                ..crate::fixtures::LIMITS
            },
            ic_wire::Limits {
                nodes: 0,
                ..crate::fixtures::LIMITS
            },
        ] {
            assert_eq!(
                native::decode_request::<Option<Magnitude>>(26, &bytes, limits),
                Err(Error::Limit)
            );
        }
    }
    assert_eq!(
        from_nat(&Nat(BigUint::from(1_u8) << 512)),
        Err(Error::Limit)
    );
    assert_eq!(
        to_nat(&Magnitude::new(vec![0]).unwrap()),
        Err(Error::Malformed)
    );
    assert_eq!(
        to_nat(&Magnitude::new(vec![0, 1]).unwrap()),
        Err(Error::Malformed)
    );
    // Generic byte admission does not silently strip or reject leading zeros.
    let raw = native::encode(
        &Magnitude::new(vec![0, 1]).unwrap(),
        crate::fixtures::LIMITS,
    )
    .unwrap();
    let admitted: Magnitude = native::decode(&raw, crate::fixtures::LIMITS).unwrap();
    assert_eq!(to_nat(&admitted), Err(Error::Malformed));
    assert_eq!(
        native::encode(&None::<Magnitude>, crate::fixtures::LIMITS).unwrap(),
        [0]
    );
    assert_eq!(
        native::encode(
            &Some(Magnitude::new(vec![]).unwrap()),
            crate::fixtures::LIMITS
        )
        .unwrap(),
        [1, 0]
    );
    assert_eq!(
        native::decode::<Magnitude>(&[0x80, 0], crate::fixtures::LIMITS)
            .unwrap()
            .as_slice(),
        &[] as &[u8]
    );
    for raw in [vec![1], [vec![65], vec![1; 65]].concat()] {
        assert_eq!(
            native::decode::<Magnitude>(&raw, crate::fixtures::LIMITS),
            Err(Error::Malformed)
        );
    }
}
