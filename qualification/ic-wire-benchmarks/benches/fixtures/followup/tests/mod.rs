#[test]
fn container_cases_append_without_reordering_historical_indices() {
    use super::*;
    use sha2::{Digest, Sha256};
    let cases = cases_selected(None);
    assert_eq!(cases.len(), 154);
    let appended = cases[134..]
        .iter()
        .map(|case| case.name.clone())
        .collect::<Vec<_>>();
    let expected = [0, 1, 128, 4096]
        .into_iter()
        .flat_map(|n| {
            [
                "page-packed-records",
                "mixed-change-batch",
                "sparse-optional-blobs",
                "map-result-blobs",
                "mixed-width-tuples",
            ]
            .map(|shape| format!("{shape}-{n}"))
        })
        .collect::<Vec<_>>();
    assert_eq!(appended, expected);
    for (index, full) in cases.iter().enumerate().skip(134) {
        let selected = cases_selected(Some(index));
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, full.name);
        assert_eq!(selected[0].input_json, full.input_json);
        for (a, b) in selected[0].lanes.iter().zip(&full.lanes) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.bytes, b.bytes);
        }
    }
    let historical = cases[..94]
        .iter()
        .map(|case| case.name.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        Sha256::digest(historical.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "c62153b64bd91c86603104afcf4d1e8bab90830f2086e8e3a0f093bfc7920dfe"
    );
    for n in [0, 1, 128, 4096] {
        let arrays = cases
            .iter()
            .find(|c| c.name == format!("byte-array-hashes-{n}"))
            .unwrap();
        let packed = cases
            .iter()
            .find(|c| c.name == format!("packed-hashes-{n}"))
            .unwrap();
        assert_eq!(arrays.input_json, packed.input_json);
        let candid = |case: &Case| {
            case.lanes
                .iter()
                .find(|l| l.name == "candid")
                .unwrap()
                .bytes
                .clone()
        };
        assert_eq!(candid(arrays), candid(packed));
    }
}

#[test]
fn nested_byte_options_preserve_absent_inner_absent_and_empty() {
    use super::*;
    type Value = Option<Option<ic_wire::ByteBuf>>;
    for (value, bytes) in [
        (None, vec![0]),
        (Some(None), vec![1, 0]),
        (Some(Some(ic_wire::ByteBuf::new())), vec![1, 1, 0]),
        (
            Some(Some(ic_wire::ByteBuf::from(vec![7]))),
            vec![1, 1, 1, 7],
        ),
    ] {
        assert_eq!(ic_wire::native::encode(&value, LIMITS).unwrap(), bytes);
        assert_eq!(
            ic_wire::native::decode::<Value>(&bytes, LIMITS).unwrap(),
            value
        );
        for end in 0..bytes.len() {
            assert!(ic_wire::native::decode::<Value>(&bytes[..end], LIMITS).is_err());
        }
    }
    for invalid in [&[2][..], &[1, 2], &[1, 1, 2, 7]] {
        assert!(ic_wire::native::decode::<Value>(invalid, LIMITS).is_err());
    }
}

#[test]
fn nonzero_list_rejects_a_late_zero_without_affecting_the_next_decode() {
    use super::*;
    use std::num::NonZeroI64;
    let valid = vec![i64::MIN, -1, i64::MAX];
    let mut invalid = valid.clone();
    invalid.push(0);
    let bytes = ic_wire::native::encode(&invalid, LIMITS).unwrap();
    assert!(ic_wire::native::decode::<Vec<NonZeroI64>>(&bytes, LIMITS).is_err());
    let bytes = ic_wire::native::encode(&valid, LIMITS).unwrap();
    let decoded: Vec<NonZeroI64> = ic_wire::native::decode(&bytes, LIMITS).unwrap();
    assert_eq!(
        decoded.into_iter().map(NonZeroI64::get).collect::<Vec<_>>(),
        valid
    );
}

#[test]
fn candid_byte_adapters_preserve_frames_and_owned_principal_boundary() {
    use super::*;
    let cases = cases_selected(None);
    for plain in cases.iter().filter(|c| c.name.contains("-plain-")) {
        let bytes = cases
            .iter()
            .find(|c| c.name == plain.name.replace("-plain-", "-bytes-"))
            .unwrap();
        assert_eq!(plain.input_json, bytes.input_json);
        for lane in &plain.lanes {
            assert_eq!(
                lane.bytes,
                bytes
                    .lanes
                    .iter()
                    .find(|b| b.name == lane.name)
                    .unwrap()
                    .bytes,
                "{} / {}",
                plain.name,
                lane.name
            );
        }
    }
    for n in [0, 1, 29] {
        let case = cases
            .iter()
            .find(|c| c.name == format!("owned-principal-{n}"))
            .unwrap();
        let lane = case.lanes.iter().find(|l| l.name == "wire-native").unwrap();
        let principal: ic_platform_types::Principal =
            ic_wire::native::decode_request(METHOD, &lane.bytes, LIMITS).unwrap();
        assert_eq!(principal.as_slice(), vec![4; n]);
    }
}
#[test]
fn public_bools_and_research_floats_validate_lengths_and_limits() {
    use super::*;
    use words::Bulk;
    let bools = words::BoolValue(vec![false, true]);
    assert_eq!(ic_wire::native::encode(&bools, LIMITS).unwrap(), [2, 0, 1]);
    assert_eq!(
        ic_wire::native::decode::<words::BoolValue>(&[2, 0, 1], LIMITS)
            .unwrap()
            .0,
        [false, true]
    );
    for bad in [vec![1, 2], vec![2, 0], vec![3, 0, 1]] {
        assert!(ic_wire::native::decode::<words::BoolValue>(&bad, LIMITS).is_err());
    }
    for bad in [vec![1, 0], vec![3, 0, 0, 0], vec![4, 0, 0, 0]] {
        assert!(ic_wire::native::decode::<Bulk<f32>>(&bad, LIMITS).is_err());
    }
    for bits in [
        0_u32, 0x80000000, 1, 0x7f800000, 0xff800000, 0x7fc00001, 0x7f800001,
    ] {
        let bytes = ic_wire::native::encode(&Bulk(vec![f32::from_bits(bits)]), LIMITS).unwrap();
        assert_eq!(bytes, [vec![4], bits.to_le_bytes().to_vec()].concat());
        assert_eq!(
            ic_wire::native::decode::<Bulk<f32>>(&bytes, LIMITS)
                .unwrap()
                .0[0]
                .to_bits(),
            bits
        );
    }
    let bytes = ic_wire::native::encode(&words::BoolValue(vec![true; 128]), LIMITS).unwrap();
    for limits in [
        ic_wire::Limits {
            bytes: bytes.len() - 1,
            ..LIMITS
        },
        ic_wire::Limits {
            allocation: 0,
            ..LIMITS
        },
        ic_wire::Limits { nodes: 0, ..LIMITS },
        ic_wire::Limits { depth: 0, ..LIMITS },
    ] {
        assert_eq!(
            ic_wire::native::decode::<words::BoolValue>(&bytes, limits).unwrap_err(),
            ic_wire::Error::Limit
        );
    }
}

#[test]
fn public_bool_rejects_every_invalid_bit_pattern() {
    use super::*;
    for len in [0, 1, 31, 32, 33] {
        let expected: Vec<_> = (0..len).map(|i| i % 2 == 1).collect();
        let frame = ic_wire::native::encode(&words::BoolValue(expected.clone()), LIMITS).unwrap();
        assert_eq!(
            ic_wire::native::decode::<words::BoolValue>(&frame, LIMITS)
                .unwrap()
                .0,
            expected
        );
        for position in 0..len {
            for invalid in 2..=u8::MAX {
                let mut malformed = frame.clone();
                malformed[1 + position] = invalid;
                assert_eq!(
                    ic_wire::native::decode::<words::BoolValue>(&malformed, LIMITS).unwrap_err(),
                    ic_wire::Error::Malformed
                );
            }
        }
    }
}

#[test]
fn public_bool_preserves_list_bytes_at_length_boundaries() {
    use super::*;
    for len in [0, 1, 31, 32, 33, 127, 128, 255, 256, 16_383, 16_384] {
        for pattern in 0..3 {
            let values: Vec<_> = (0..len)
                .map(|index| match pattern {
                    0 => false,
                    1 => true,
                    _ => index % 2 == 1,
                })
                .collect();
            let bulk = words::BoolValue(values.clone());
            let bytes = ic_wire::native::encode(&values, LIMITS).unwrap();
            assert_eq!(ic_wire::native::encode(&bulk, LIMITS).unwrap(), bytes);
            assert_eq!(
                ic_wire::native::encode_request(METHOD, &values, LIMITS).unwrap(),
                ic_wire::native::encode_request(METHOD, &bulk, LIMITS).unwrap()
            );
            assert_eq!(
                ic_wire::native::decode::<words::BoolValue>(&bytes, LIMITS)
                    .unwrap()
                    .0,
                values
            );
            assert_eq!(
                ic_wire::native::decode::<Vec<bool>>(&bytes, LIMITS).unwrap(),
                values
            );
            let prefix = bytes.len() - len;
            for cut in [0, prefix - 1, prefix + len / 2, bytes.len() - 1] {
                if cut == bytes.len() {
                    continue;
                }
                assert_eq!(
                    ic_wire::native::decode::<Vec<bool>>(&bytes[..cut], LIMITS).unwrap_err(),
                    ic_wire::Error::Malformed
                );
                assert_eq!(
                    ic_wire::native::decode::<words::BoolValue>(&bytes[..cut], LIMITS).unwrap_err(),
                    ic_wire::Error::Malformed
                );
            }
            let mut trailing = bytes;
            trailing.push(0);
            assert_eq!(
                ic_wire::native::decode::<words::BoolValue>(&trailing, LIMITS).unwrap_err(),
                ic_wire::Error::Trailing
            );
            assert_eq!(
                ic_wire::native::decode::<Vec<bool>>(&trailing, LIMITS).unwrap_err(),
                ic_wire::Error::Trailing
            );
        }
    }
    // Both readers follow upstream Postcard's redundant length acceptance.
    for bytes in [&[0x80, 0][..], &[0x81, 0, 1][..]] {
        assert!(ic_wire::native::decode::<words::BoolValue>(bytes, LIMITS).is_ok());
        assert!(ic_wire::native::decode::<Vec<bool>>(bytes, LIMITS).is_ok());
    }
}

#[test]
fn public_bool_adoption_changes_admission_despite_identical_bytes() {
    use super::*;
    let values = vec![true; 128];
    let bulk = words::BoolValue(values.clone());
    let bytes = ic_wire::native::encode(&values, LIMITS).unwrap();
    for encoding in [false, true] {
        for packed in [false, true] {
            let admit = |limits| {
                if encoding {
                    if packed {
                        ic_wire::native::encode(&bulk, limits).map(|_| ())
                    } else {
                        ic_wire::native::encode(&values, limits).map(|_| ())
                    }
                } else if packed {
                    ic_wire::native::decode::<words::BoolValue>(&bytes, limits).map(|_| ())
                } else {
                    ic_wire::native::decode::<Vec<bool>>(&bytes, limits).map(|_| ())
                }
            };
            // Byte admission changes traversal, not values: one buffer charge replaces
            // per-element visits. These are allowance units, not heap bytes.
            let root_units = size_of::<Vec<bool>>() * if encoding { 2 } else { 1 };
            // Serde's collect_seq passes each borrowed element as &bool, so
            // ordinary encode charges the reference size, not one bool byte.
            let element_units = if packed {
                1
            } else if encoding {
                2 * size_of::<&bool>()
            } else {
                2
            };
            let limits = ic_wire::Limits {
                bytes: bytes.len(),
                allocation: bytes.len() + 8 * (root_units + values.len() * element_units),
                nodes: if encoding { 4 } else { 1 }
                    + if packed {
                        0
                    } else {
                        values.len() * if encoding { 4 } else { 2 }
                    },
                depth: if encoding { 3 } else { 1 } + usize::from(!packed),
            };
            assert_eq!(
                admit(limits),
                Ok(()),
                "encoding={encoding}, packed={packed}"
            );
            for axis in 0..4 {
                let mut short = limits;
                match axis {
                    0 => short.bytes -= 1,
                    1 => short.allocation -= 1,
                    2 => short.nodes -= 1,
                    _ => short.depth -= 1,
                }
                assert_eq!(
                    admit(short),
                    Err(ic_wire::Error::Limit),
                    "encoding={encoding}, packed={packed}, axis={axis}"
                );
            }
        }
    }
}

#[test]
fn borrowed_record_views_remain_inside_the_owned_frame() {
    use super::*;
    let case = borrowed_case(65536);
    let frame = &case
        .lanes
        .iter()
        .find(|l| l.name == "wire-native")
        .unwrap()
        .bytes;
    let view: BorrowedRecord<'_> = ic_wire::native::decode_request(METHOD, frame, LIMITS).unwrap();
    let start = frame.as_ptr() as usize;
    let end = start + frame.len();
    for (p, n) in [
        (view.label.as_ptr(), view.label.len()),
        (view.payload.as_ptr(), view.payload.len()),
    ] {
        let p = p as usize;
        assert!(p >= start && p + n <= end);
    }
    assert_eq!(
        (case
            .lanes
            .iter()
            .find(|l| l.name == "wire-borrowed-record")
            .unwrap()
            .encode)(),
        *frame
    );
}
