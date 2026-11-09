#[test]
fn packed_hash_shape_refuses_partial_and_oversized_elements() {
    assert_eq!(
        super::packed_checksum(&[0; 31], true),
        Err(ic_wire::Error::Malformed)
    );
    assert_eq!(
        super::packed_checksum(&[0; 33], true),
        Err(ic_wire::Error::Malformed)
    );
    assert_eq!(
        super::packed_checksum(&vec![0; 32769], true),
        Err(ic_wire::Error::Limit)
    );
    assert_eq!(super::packed_checksum(&[1; 32], true), Ok(32));
}

#[test]
fn codec_only_preserves_frames_and_capacity() {
    for case in 0..super::CASE_COUNT {
        let full = super::run(case, 1, &mut || 0);
        let codec = super::run_with_scan(case, [1; 2], &mut || 0, false);
        assert_eq!(full.bytes, codec.bytes);
        assert_eq!(full.capacity, codec.capacity);
    }
    assert_eq!(
        super::packed_checksum(&[0; 31], false),
        Err(ic_wire::Error::Malformed)
    );
    assert_eq!(
        super::packed_checksum(&vec![0; 32769], false),
        Err(ic_wire::Error::Limit)
    );
}

#[test]
fn same_layout_lanes_match_and_layout_changes_have_distinct_identities() {
    for group in 0..4 {
        let samples = (0..7)
            .map(|lane| super::run(group * 7 + lane, 1, &mut || 0))
            .collect::<Vec<_>>();
        for lane in [1, 2] {
            assert_eq!(samples[0].bytes, samples[lane].bytes);
        }
        for lane in [4, 5, 6] {
            assert_eq!(samples[3].bytes, samples[lane].bytes);
        }
        assert_ne!(&samples[0].bytes[..11], &samples[3].bytes[..11]);
        let blobs = (0..4)
            .map(|lane| super::run(28 + group * 4 + lane, 1, &mut || 0))
            .collect::<Vec<_>>();
        for lane in 1..4 {
            assert_eq!(blobs[0].bytes, blobs[lane].bytes);
        }
    }
}
