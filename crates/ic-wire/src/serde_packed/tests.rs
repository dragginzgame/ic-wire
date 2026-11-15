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

#[test]
fn paired_float_scan_covers_every_exponent_combination() {
    #[derive(serde::Deserialize)]
    struct Values(#[serde(with = "super::finite_f32")] alloc::vec::Vec<f32>);

    // Independent Postcard literal: byte length 8 followed by two IEEE words.
    // Exercise both exponent lanes and both sign/mantissa extremes. Valid
    // words must retain their bits, including negative zero and subnormals.
    for left_exponent in 0_u32..256 {
        for right_exponent in 0_u32..256 {
            for left_other in [0, 0x807f_ffff] {
                for right_other in [0, 0x807f_ffff] {
                    let left = f32::from_bits((left_exponent << 23) | left_other);
                    let right = f32::from_bits((right_exponent << 23) | right_other);
                    let mut bytes = [8_u8; 9];
                    bytes[1..5].copy_from_slice(&left.to_le_bytes());
                    bytes[5..].copy_from_slice(&right.to_le_bytes());
                    let decoded = postcard::from_bytes::<Values>(&bytes);
                    if left.is_finite() && right.is_finite() {
                        let values = decoded.unwrap().0;
                        assert_eq!(values.len(), 2);
                        assert_eq!(values[0].to_bits(), left.to_bits());
                        assert_eq!(values[1].to_bits(), right.to_bits());
                    } else {
                        assert!(decoded.is_err());
                    }
                }
            }
        }
    }
}
