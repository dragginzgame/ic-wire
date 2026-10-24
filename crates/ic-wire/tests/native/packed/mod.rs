//! Bulk byte layouts, malformed admission and exhaustive i16 compatibility.
use ic_wire::Packed;
const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 1_048_576,
    allocation: 4_194_304,
    nodes: 1_048_576,
    depth: 64,
};

#[test]
fn packed_words_match_little_endian_bytes_and_unaligned_decode() {
    macro_rules! check {
        ($($ty:ty),*) => {$(
            let values = vec![0 as $ty, 1, <$ty>::MIN, <$ty>::MAX];
            let mut expected = vec![(values.len() * size_of::<$ty>()) as u8];
            expected.extend(values.iter().flat_map(|v| v.to_le_bytes()));
            let packed = Packed::<_>::new(values);
            let encoded = ic_wire::native::encode(&packed, LIMITS).unwrap();
            assert_eq!(encoded, expected);
            let mut unaligned = vec![0xff];
            unaligned.extend(encoded);
            let decoded: Packed<$ty> = ic_wire::native::decode(&unaligned[1..], LIMITS).unwrap();
            assert_eq!(decoded, packed);
        )*};
    }
    check!(i16, u16, i32, u32, i64, u64);
    let packed = Packed::<_>::new(vec![[0xab; 32]]);
    let bytes = ic_wire::native::encode(&packed, LIMITS).unwrap();
    assert_eq!(bytes, [vec![32], vec![0xab; 32]].concat());
    assert_eq!(
        ic_wire::native::decode::<Packed<[u8; 32]>>(&bytes, LIMITS).unwrap(),
        packed
    );
}

#[test]
fn invalid_widths_and_noncanonical_lengths_are_rejected() {
    macro_rules! check {
        ($($ty:ty),*) => {$(
            let width = size_of::<$ty>();
            for len in 0..=width * 3 {
                let bytes = ic_wire::native::encode(&ic_wire::Bytes::<256>::new(vec![0; len]).unwrap(), LIMITS).unwrap();
                let decoded = ic_wire::native::decode::<Packed<$ty>>(&bytes, LIMITS);
                if len.is_multiple_of(width) {
                    assert!(decoded.is_ok());
                } else {
                    assert_eq!(decoded, Err(ic_wire::Error::Malformed));
                }
            }
            assert_eq!(ic_wire::native::decode::<Packed<$ty>>(&[0x80, 0], LIMITS), Err(ic_wire::Error::Malformed));
        )*};
    }
    check!(i16, u16, i32, u32, i64, u64, [u8; 32]);
}

#[test]
fn runtime_byte_and_allocation_limits_still_reject() {
    let packed = Packed::new(vec![u64::MAX; 4]);
    let bytes = ic_wire::native::encode(&packed, LIMITS).unwrap();
    let limits = ic_wire::Limits {
        bytes: bytes.len() - 1,
        ..LIMITS
    };
    assert_eq!(
        ic_wire::native::decode::<Packed<u64>>(&bytes, limits),
        Err(ic_wire::Error::Limit)
    );
    let limits = ic_wire::Limits {
        allocation: 0,
        ..LIMITS
    };
    assert_eq!(
        ic_wire::native::decode::<Packed<u64>>(&bytes, limits),
        Err(ic_wire::Error::Limit)
    );
}

#[test]
fn generic_i16_matches_existing_packed_type_for_every_word() {
    let values: Vec<i16> = (i16::MIN..=i16::MAX).collect();
    let existing = ic_wire::PackedI16::<65536>::new(values.clone()).unwrap();
    let generic = Packed::<i16>::new(values);
    let bytes = ic_wire::native::encode(&generic, LIMITS).unwrap();
    assert_eq!(bytes, ic_wire::native::encode(&existing, LIMITS).unwrap());
    assert_eq!(
        ic_wire::native::decode::<Packed<i16>>(&bytes, LIMITS).unwrap(),
        generic
    );
}

#[test]
fn blob_manifest_boundary_preserves_order_and_header_bytes() {
    // Boundary shape inspected in ic-blob-storage dcbb8811, contracts DTO
    // upload/manifest. This qualifies Wire's proposed field substitution only;
    // it does not migrate the consumer, provider JSON or persisted records.
    #[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Header {
        name: String,
        value: String,
    }
    #[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Declaration<T> {
        chunks: T,
        headers: Vec<Header>,
    }
    #[cfg(feature = "typescript")]
    {
        let schema = ic_wire::typescript::native_schema::<Declaration<Packed<[u8; 32]>>>().unwrap();
        assert_eq!(schema["fields"][0]["name"], "chunks");
        assert_eq!(
            schema["fields"][0]["value"],
            serde_json::json!({"k":"packed","word":"hash32"})
        );
        assert_eq!(schema["fields"][1]["name"], "headers");
        assert_eq!(
            schema["fields"][1]["value"]["item"]["fields"][0]["name"],
            "name"
        );
        assert_eq!(
            schema["fields"][1]["value"]["item"]["fields"][1]["name"],
            "value"
        );
    }
    for (count, old_prefix, packed_prefix) in [(0, 1, 1), (1, 1, 1), (4, 1, 2), (1024, 2, 3)] {
        let hashes: Vec<_> = (0..count).map(|i| [i as u8; 32]).collect();
        let headers = || {
            vec![Header {
                name: "Content-Length".into(),
                value: "32".into(),
            }]
        };
        let old = Declaration {
            chunks: hashes.clone(),
            headers: headers(),
        };
        let proposed = Declaration {
            chunks: Packed::new(hashes.clone()),
            headers: headers(),
        };
        let before = ic_wire::native::encode(&old, LIMITS).unwrap();
        let after = ic_wire::native::encode(&proposed, LIMITS).unwrap();
        // Opt-in byte-count prefix replaces the element count. Hash order and
        // original header spelling/value bytes stay exact; clients must coordinate.
        assert_eq!(&before[old_prefix..], &after[packed_prefix..]);
        if count == 1 {
            assert_eq!((before[0], after[0]), (1, 32));
        }
        let decoded: Declaration<Packed<[u8; 32]>> =
            ic_wire::native::decode(&after, LIMITS).unwrap();
        assert_eq!(decoded, proposed);
        assert_eq!(decoded.chunks.into_vec(), hashes);
        let too_small = ic_wire::Limits {
            bytes: after.len() - 1,
            ..LIMITS
        };
        assert_eq!(
            ic_wire::native::decode::<Declaration<Packed<[u8; 32]>>>(&after, too_small),
            Err(ic_wire::Error::Limit)
        );
    }
}

#[test]
fn packed_length_prefixes_offsets_and_budget_boundaries() {
    fn check<T: ic_wire::PackedElement>() {
        let width = size_of::<T>();
        for len in (0..=65).chain([127, 128, 129, 255, 256, 257, 16383, 16384]) {
            let data: Vec<u8> = (0..len)
                .map(|i| (i as u8).wrapping_mul(73).wrapping_add(19))
                .collect();
            let bytes =
                ic_wire::native::encode(&ic_wire::Bytes::<32768>::new(data).unwrap(), LIMITS)
                    .unwrap();
            for offset in 0..8 {
                let mut input = vec![0xa5; offset];
                input.extend_from_slice(&bytes);
                let decoded = ic_wire::native::decode::<Packed<T>>(&input[offset..], LIMITS);
                if len % width != 0 {
                    assert_eq!(decoded, Err(ic_wire::Error::Malformed));
                    continue;
                }
                let decoded = decoded.unwrap();
                assert_eq!(decoded.as_slice().len(), len / width);
                assert_eq!(ic_wire::native::encode(&decoded, LIMITS).unwrap(), bytes);
            }
            if len % width != 0 {
                continue;
            }
            for end in [0, 1, bytes.len() / 2, bytes.len() - 1] {
                if end < bytes.len() {
                    assert!(ic_wire::native::decode::<Packed<T>>(&bytes[..end], LIMITS).is_err());
                }
            }
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert_eq!(
                ic_wire::native::decode::<Packed<T>>(&trailing, LIMITS),
                Err(ic_wire::Error::Trailing)
            );
            let last_prefix = bytes.iter().position(|b| b & 0x80 == 0).unwrap();
            let mut noncanonical = bytes.clone();
            noncanonical[last_prefix] |= 0x80;
            noncanonical.insert(last_prefix + 1, 0);
            assert_eq!(
                ic_wire::native::decode::<Packed<T>>(&noncanonical, LIMITS),
                Err(ic_wire::Error::Malformed)
            );
            let mut low = 0;
            let mut high = LIMITS.allocation;
            while low < high {
                let allocation = low + (high - low) / 2;
                let result = ic_wire::native::decode::<Packed<T>>(
                    &bytes,
                    ic_wire::Limits {
                        allocation,
                        ..LIMITS
                    },
                );
                match result {
                    Ok(_) => high = allocation,
                    Err(ic_wire::Error::Limit) => low = allocation + 1,
                    Err(error) => panic!("unexpected admission error: {error:?}"),
                }
            }
            assert!(
                ic_wire::native::decode::<Packed<T>>(
                    &bytes,
                    ic_wire::Limits {
                        allocation: high,
                        ..LIMITS
                    }
                )
                .is_ok()
            );
            if high > 0 {
                assert_eq!(
                    ic_wire::native::decode::<Packed<T>>(
                        &bytes,
                        ic_wire::Limits {
                            allocation: high - 1,
                            ..LIMITS
                        }
                    ),
                    Err(ic_wire::Error::Limit)
                );
            }
            for limits in [
                ic_wire::Limits {
                    bytes: bytes.len() - 1,
                    ..LIMITS
                },
                ic_wire::Limits { nodes: 0, ..LIMITS },
            ] {
                assert_eq!(
                    ic_wire::native::decode::<Packed<T>>(&bytes, limits),
                    Err(ic_wire::Error::Limit)
                );
            }
        }
        for bytes in [&[0xff; 12][..], &[0x80; 12][..]] {
            assert!(ic_wire::native::decode::<Packed<T>>(bytes, LIMITS).is_err());
        }
    }
    check::<i16>();
    check::<u16>();
    check::<i32>();
    check::<u32>();
    check::<i64>();
    check::<u64>();
    check::<[u8; 32]>();
}
