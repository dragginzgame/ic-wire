//! External Nat conversion stays in this unpublished consumer qualification.
use ic_platform::candid::contract::Nat;
use ic_wire::{Bytes, Error, native};
use num_bigint::BigUint;

// Consumer-selected maximum: 64 magnitude bytes, not a Wire-wide integer limit.
type Magnitude = Bytes<64>;

fn from_nat(value: &Nat) -> Result<Magnitude, Error> {
    // Check the external integer before allocating its magnitude buffer.
    if value.0.bits() > 64 * 8 {
        return Err(Error::Limit);
    }
    let bytes = if value.0.bits() == 0 {
        Vec::new()
    } else {
        value.0.to_bytes_be()
    };
    Magnitude::new(bytes)
}
fn to_nat(value: &Magnitude) -> Result<Nat, Error> {
    // Bytes intentionally accepts opaque bytes; numeric canonicality is checked
    // at the consumer boundary, before constructing its arithmetic type.
    if value.as_slice().first() == Some(&0) {
        return Err(Error::Malformed);
    }
    Ok(Nat(BigUint::from_bytes_be(value.as_slice())))
}

#[test]
fn external_nat_evidence_preserves_overflow_and_rejects_nonminimal_magnitudes() {
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
        let bytes = native::encode_request(26, &wire, super::fixtures::LIMITS).unwrap();
        let decoded: Option<Magnitude> =
            native::decode_request(26, &bytes, super::fixtures::LIMITS).unwrap();
        assert_eq!(decoded.as_ref().map(to_nat).transpose().unwrap(), original);
        for limits in [
            ic_wire::Limits {
                bytes: bytes.len() - 1,
                ..super::fixtures::LIMITS
            },
            ic_wire::Limits {
                allocation: 0,
                ..super::fixtures::LIMITS
            },
            ic_wire::Limits {
                nodes: 0,
                ..super::fixtures::LIMITS
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
        super::fixtures::LIMITS,
    )
    .unwrap();
    let admitted: Magnitude = native::decode(&raw, super::fixtures::LIMITS).unwrap();
    assert_eq!(to_nat(&admitted), Err(Error::Malformed));
    assert_eq!(
        native::encode(&None::<Magnitude>, super::fixtures::LIMITS).unwrap(),
        [0]
    );
    assert_eq!(
        native::encode(
            &Some(Magnitude::new(vec![]).unwrap()),
            super::fixtures::LIMITS
        )
        .unwrap(),
        [1, 0]
    );
    for raw in [vec![0x80, 0], vec![1], [vec![65], vec![1; 65]].concat()] {
        assert_eq!(
            native::decode::<Magnitude>(&raw, super::fixtures::LIMITS),
            Err(Error::Malformed)
        );
    }
}
