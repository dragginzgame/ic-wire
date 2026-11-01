//! Native Serde compatibility, independent of generated TypeScript metadata.
use super::*;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

fn roundtrip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let raw = postcard::to_allocvec(value).unwrap();
    assert_eq!(native::encode(value, LIMITS).unwrap(), raw);
    assert_eq!(native::decode::<T>(&raw, LIMITS).unwrap(), *value);
    for response in [false, true] {
        let frame = if response {
            native::encode_response(7, value, LIMITS).unwrap()
        } else {
            native::encode_request(7, value, LIMITS).unwrap()
        };
        assert_eq!(&frame[11..], raw);
        let decode = |bytes: &[u8]| {
            if response {
                native::decode_response::<T>(7, bytes, LIMITS)
            } else {
                native::decode_request::<T>(7, bytes, LIMITS)
            }
        };
        assert_eq!(decode(&frame).unwrap(), *value);
        for end in 0..frame.len() {
            assert!(decode(&frame[..end]).is_err(), "accepted prefix {end}");
        }
        let mut trailing = frame;
        trailing.push(0);
        assert_eq!(decode(&trailing), Err(Error::Trailing));
    }
}

#[test]
fn nonzero_integers_preserve_primitive_bytes_and_reject_zero() {
    macro_rules! check {
        ($($primitive:ty => $nonzero:ty),* $(,)?) => {$(
            for value in [1, <$primitive>::MIN, <$primitive>::MAX] {
                if let Some(nonzero) = <$nonzero>::new(value) {
                    roundtrip(&nonzero);
                    assert_eq!(native::encode(&nonzero, LIMITS), native::encode(&value, LIMITS));
                    roundtrip(&Some(nonzero));
                    roundtrip(&[nonzero; 2]);
                }
            }
            let zero = native::encode(&(0 as $primitive), LIMITS).unwrap();
            assert_eq!(native::decode::<$nonzero>(&zero, LIMITS), Err(Error::Malformed));
            roundtrip(&None::<$nonzero>);
        )*};
    }
    check! {
        u8 => std::num::NonZeroU8,
        u16 => std::num::NonZeroU16,
        u32 => std::num::NonZeroU32,
        u64 => std::num::NonZeroU64,
        u128 => std::num::NonZeroU128,
        usize => std::num::NonZeroUsize,
        i8 => std::num::NonZeroI8,
        i16 => std::num::NonZeroI16,
        i32 => std::num::NonZeroI32,
        i64 => std::num::NonZeroI64,
        i128 => std::num::NonZeroI128,
        isize => std::num::NonZeroIsize,
    }
}

#[test]
fn network_addresses_preserve_binary_serde_shapes() {
    let v4 = std::net::Ipv4Addr::new(192, 0, 2, 1);
    let v6 = std::net::Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
    roundtrip(&v4);
    roundtrip(&v6);
    for address in [std::net::IpAddr::V4(v4), std::net::IpAddr::V6(v6)] {
        roundtrip(&address);
        roundtrip(&std::net::SocketAddr::new(address, u16::MAX));
    }
    assert_eq!(
        native::decode::<std::net::IpAddr>(&[2], LIMITS),
        Err(Error::Malformed)
    );
}

#[test]
fn standard_ranges_and_nested_nonzero_results_follow_serde() {
    roundtrip(&(0_u64..u64::MAX));
    roundtrip(&std::ops::RangeInclusive::new(i64::MIN, i64::MAX));
    // Range ordering is consumer semantics, not malformed primitive input.
    roundtrip(&std::ops::RangeInclusive::new(5_u32, 2));
    let nonzero = std::num::NonZeroU16::new(128).unwrap();
    let value = vec![Ok(ic_wire::ByteBuf::from(vec![0; 128])), Err(nonzero)];
    roundtrip(&value);
    // A valid blob followed by a consumer-invalid zero must fail without
    // contaminating the next decode's bulk/primitive state or allowances.
    let mut bad = native::encode(&value, LIMITS).unwrap();
    let end = bad.len();
    bad.splice(end - 2.., [0]);
    assert_eq!(
        native::decode::<Vec<Result<ic_wire::ByteBuf, std::num::NonZeroU16>>>(&bad, LIMITS),
        Err(Error::Malformed)
    );
    roundtrip(&value);
}

#[test]
fn raw_ieee_float_bits_survive_without_partial_eq_assumptions() {
    macro_rules! check {
        ($primitive:ty, $bits:expr) => {
            for bits in $bits {
                let value = <$primitive>::from_bits(bits);
                let bytes = native::encode(&value, LIMITS).unwrap();
                assert_eq!(bytes, bits.to_le_bytes());
                assert_eq!(
                    native::decode::<$primitive>(&bytes, LIMITS)
                        .unwrap()
                        .to_bits(),
                    bits
                );
                let frame = native::encode_response(7, &value, LIMITS).unwrap();
                assert_eq!(
                    native::decode_response::<$primitive>(7, &frame, LIMITS)
                        .unwrap()
                        .to_bits(),
                    bits
                );
            }
        };
    }
    check!(
        f32,
        [
            0_u32,
            0x8000_0000,
            1,
            0x7f80_0000,
            0xff80_0000,
            0x7fc0_0001,
            0xffc0_1234
        ]
    );
    check!(
        f64,
        [
            0_u64,
            0x8000_0000_0000_0000,
            1,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff8_0000_0000_0001,
            0xfff8_0000_0000_1234
        ]
    );
}
