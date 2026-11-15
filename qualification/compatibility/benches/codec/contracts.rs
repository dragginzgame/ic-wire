//! Failure-position and borrowing checks complementary to timed owned fixtures.
use super::{Candidate, Reference, Supplier};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Borrowed<'a> {
    text: &'a str,
    #[serde(borrow, with = "serde_bytes")]
    bytes: &'a [u8],
}

fn borrowed<C: Supplier>() {
    for size in [1, 127, 128, 4096] {
        let text = "é🌍".repeat(size);
        let data = vec![0xa5; size];
        let value = Borrowed {
            text: &text,
            bytes: &data,
        };
        let encoded = C::encode(&value);
        let decoded: Borrowed<'_> = C::decode(&encoded).unwrap();
        assert_eq!(decoded, value);
        let start = encoded.as_ptr() as usize;
        let end = start + encoded.len();
        for (ptr, len) in [
            (decoded.text.as_ptr(), decoded.text.len()),
            (decoded.bytes.as_ptr(), decoded.bytes.len()),
        ] {
            let ptr = ptr as usize;
            assert!(
                ptr >= start && ptr + len <= end,
                "decoded data must borrow input"
            );
        }
    }
}
#[test]
fn text_and_bytes_borrow_the_original_frame() {
    borrowed::<Candidate>();
    borrowed::<Reference>();
}

fn prefixes<C: Supplier>() {
    for size in [0, 1, 127, 128] {
        let value = (
            "héllo",
            vec![
                Some(Ok::<u64, String>(128)),
                None,
                Some(Err("failure".into())),
            ],
            vec![7_u32; size],
        );
        let bytes = C::encode(&value);
        type Owned = (String, Vec<Option<Result<u64, String>>>, Vec<u32>);
        for end in 0..bytes.len() {
            assert!(
                C::decode::<Owned>(&bytes[..end]).is_err(),
                "accepted prefix {end}"
            );
        }
        for extra in [1, 2, 127] {
            let mut trailing = bytes.clone();
            trailing.extend(vec![0; extra]);
            assert!(C::decode::<Owned>(&trailing).is_err());
        }
    }
}
#[test]
fn every_truncated_prefix_and_trailing_span_refuses() {
    prefixes::<Candidate>();
    prefixes::<Reference>();
}

macro_rules! adapter {
    ($name:ident, $ty:ty, $path:literal) => {
        #[derive(Deserialize)]
        struct $name(#[serde(with = $path)] $ty);
    };
}
adapter!(NewF32, Vec<f32>, "candidate::serde_packed::finite_f32");
adapter!(OldF32, Vec<f32>, "reference::serde_packed::finite_f32");
adapter!(NewF64, Vec<f64>, "candidate::serde_packed::finite_f64");
adapter!(OldF64, Vec<f64>, "reference::serde_packed::finite_f64");

fn invalid_float<C: Supplier, T: DeserializeOwned>(width: usize, bad_words: &[Vec<u8>]) {
    for count in [1, 2, 7, 8, 9, 127, 128, 129] {
        for position in [0, count / 2, count - 1] {
            for bad in bad_words {
                let mut bytes = vec![0; count * width];
                bytes[position * width..(position + 1) * width].copy_from_slice(bad);
                assert!(C::decode::<T>(&C::encode(&serde_bytes::ByteBuf::from(bytes))).is_err());
            }
        }
        for remainder in 1..width {
            let bytes = vec![0; count * width + remainder];
            assert!(C::decode::<T>(&C::encode(&serde_bytes::ByteBuf::from(bytes))).is_err());
        }
    }
}
#[test]
fn nonfinite_words_at_first_middle_last_and_partial_words_refuse() {
    let f32s: Vec<_> = [
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NAN,
        f32::from_bits(0x7f80_0001),
    ]
    .map(|n| n.to_le_bytes().to_vec())
    .into();
    let f64s: Vec<_> = [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        f64::from_bits(0x7ff0_0000_0000_0001),
    ]
    .map(|n| n.to_le_bytes().to_vec())
    .into();
    invalid_float::<Candidate, NewF32>(4, &f32s);
    invalid_float::<Reference, OldF32>(4, &f32s);
    invalid_float::<Candidate, NewF64>(8, &f64s);
    invalid_float::<Reference, OldF64>(8, &f64s);
    // Inspect successful values by bits: ordinary float equality misses signed zero.
    let single = [0.0_f32, -0.0, f32::from_bits(1), f32::MAX, -f32::MAX];
    let double = [0.0_f64, -0.0, f64::from_bits(1), f64::MAX, -f64::MAX];
    let single_bytes: Vec<_> = single.iter().flat_map(|x| x.to_le_bytes()).collect();
    let double_bytes: Vec<_> = double.iter().flat_map(|x| x.to_le_bytes()).collect();
    let encoded = Candidate::encode(&serde_bytes::ByteBuf::from(single_bytes));
    assert_eq!(
        Candidate::decode::<NewF32>(&encoded)
            .unwrap()
            .0
            .iter()
            .map(|n| n.to_bits())
            .collect::<Vec<_>>(),
        single.map(f32::to_bits)
    );
    assert_eq!(
        Reference::decode::<OldF32>(&encoded)
            .unwrap()
            .0
            .iter()
            .map(|n| n.to_bits())
            .collect::<Vec<_>>(),
        single.map(f32::to_bits)
    );
    let encoded = Candidate::encode(&serde_bytes::ByteBuf::from(double_bytes));
    assert_eq!(
        Candidate::decode::<NewF64>(&encoded)
            .unwrap()
            .0
            .iter()
            .map(|n| n.to_bits())
            .collect::<Vec<_>>(),
        double.map(f64::to_bits)
    );
    assert_eq!(
        Reference::decode::<OldF64>(&encoded)
            .unwrap()
            .0
            .iter()
            .map(|n| n.to_bits())
            .collect::<Vec<_>>(),
        double.map(f64::to_bits)
    );
}
