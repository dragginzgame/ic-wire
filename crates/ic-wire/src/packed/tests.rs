use super::little_endian_bytes;

#[test]
fn alternate_endian_conversion_matches_literal_bytes() {
    assert_eq!(
        little_endian_bytes::<i16, postcard::Error>(&[i16::MIN, -1, 0, 1, i16::MAX]).unwrap(),
        [0, 128, 255, 255, 0, 0, 1, 0, 255, 127]
    );
    assert!(
        little_endian_bytes::<i16, postcard::Error>(&[])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn generic_endian_conversion_preserves_words_and_opaque_ids() {
    macro_rules! literal {
        ($ty:ty,$values:expr,$bytes:expr) => {
            assert_eq!(
                little_endian_bytes::<$ty, postcard::Error>($values).unwrap(),
                $bytes
            );
        };
    }
    literal!(i8, &[-128, 127], [128, 127]);
    literal!(u16, &[0x1234], [0x34, 0x12]);
    literal!(i32, &[-1], [255; 4]);
    literal!(u32, &[0x12345678], [0x78, 0x56, 0x34, 0x12]);
    literal!(i64, &[-1], [255; 8]);
    literal!(u64, &[0x0102030405060708], [8, 7, 6, 5, 4, 3, 2, 1]);
    literal!([u8; 16], &[[7; 16]], [7; 16]);
    literal!([u8; 32], &[[9; 32]], [9; 32]);
}
