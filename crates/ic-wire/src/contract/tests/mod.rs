use super::*;
use alloc::vec;

#[test]
fn exact_bulk_equality_covers_word_boundaries_tails_and_every_mismatch() {
    // Native unit builds exercise the same chunk branch as production Wasm.
    for len in 0..=97 {
        let original: Vec<_> = (0..len).map(|i| i as u8).collect();
        let compare = || Compare {
            remaining: &original,
            allowance: len,
            equal: true,
        };
        let mut equal = compare();
        equal.try_extend(&original).unwrap();
        assert!(equal.finalize().unwrap());
        for position in 0..len {
            let mut different = original.clone();
            different[position] ^= 1;
            let mut refused = compare();
            refused.try_extend(&different).unwrap();
            assert!(
                !refused.finalize().unwrap(),
                "length {len}, position {position}"
            );
        }
        for emitted in [len.saturating_sub(1), len + 1] {
            if emitted == len {
                continue;
            }
            let mut unequal = Compare {
                remaining: &original,
                allowance: len + 1,
                equal: true,
            };
            unequal.try_extend(&vec![0; emitted]).unwrap();
            assert!(!unequal.finalize().unwrap());
        }
    }
}

#[test]
fn a_mismatch_still_checks_later_bytes_and_the_output_allowance() {
    let mut compare = Compare {
        remaining: &[0; 9],
        allowance: 9,
        equal: true,
    };
    compare.try_extend(&[1; 8]).unwrap();
    compare.try_push(0).unwrap();
    assert_eq!(
        compare.try_push(0),
        Err(postcard::Error::SerializeBufferFull)
    );
    assert!(!compare.finalize().unwrap());
    let mut exhausted = Compare {
        remaining: &[],
        allowance: 1,
        equal: true,
    };
    exhausted.try_push(0).unwrap();
    assert!(!exhausted.finalize().unwrap());
}
