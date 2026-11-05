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
fn private_bulk_validates_bits_lengths_and_limits() {
    use super::*;
    use words::Bulk;
    let bools = Bulk(vec![false, true]);
    assert_eq!(ic_wire::native::encode(&bools, LIMITS).unwrap(), [2, 0, 1]);
    assert_eq!(
        ic_wire::native::decode::<Bulk<bool>>(&[2, 0, 1], LIMITS)
            .unwrap()
            .0,
        [false, true]
    );
    for bad in [vec![1, 2], vec![2, 0], vec![3, 0, 1]] {
        assert!(ic_wire::native::decode::<Bulk<bool>>(&bad, LIMITS).is_err());
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
    let bytes = ic_wire::native::encode(&Bulk(vec![true; 128]), LIMITS).unwrap();
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
            ic_wire::native::decode::<Bulk<bool>>(&bytes, limits).unwrap_err(),
            ic_wire::Error::Limit
        );
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
