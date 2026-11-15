#[test]
fn alternate_endian_float_conversion_preserves_literal_bits_and_refuses_nonfinite() {
    assert_eq!(
        super::finite_f32::little_endian::<postcard::Error>(&[-0.0, f32::from_bits(1)]).unwrap(),
        [0, 0, 0, 128, 1, 0, 0, 0]
    );
    assert_eq!(
        super::finite_f64::little_endian::<postcard::Error>(&[-0.0, f64::from_bits(1)]).unwrap(),
        [0, 0, 0, 0, 0, 0, 0, 128, 1, 0, 0, 0, 0, 0, 0, 0]
    );
    assert!(super::finite_f32::little_endian::<postcard::Error>(&[f32::NAN]).is_err());
    assert!(super::finite_f64::little_endian::<postcard::Error>(&[f64::INFINITY]).is_err());
}
