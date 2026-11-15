use super::little_endian_bytes;

#[test]
fn alternate_endian_conversion_matches_literal_bytes() {
    assert_eq!(
        little_endian_bytes::<postcard::Error>(&[i16::MIN, -1, 0, 1, i16::MAX]).unwrap(),
        [0, 128, 255, 255, 0, 0, 1, 0, 255, 127]
    );
    assert!(
        little_endian_bytes::<postcard::Error>(&[])
            .unwrap()
            .is_empty()
    );
}
