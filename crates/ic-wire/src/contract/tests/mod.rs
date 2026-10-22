use super::*;
use alloc::vec;

#[test]
fn exact_bulk_equality_covers_word_boundaries_tails_and_every_mismatch() {
    // Native unit builds exercise the same chunk branch as production Wasm.
    for len in 0..=257 {
        let storage: Vec<_> = (0..len + 1).map(|i| i as u8).collect();
        // Exercise unaligned starts as well as every 64-byte block/tail boundary.
        let original = &storage[1..];
        let compare = || Compare {
            remaining: original,
            allowance: len,
            equal: true,
        };
        let mut equal = compare();
        equal.try_extend(original).unwrap();
        assert!(equal.finalize().unwrap());
        let copy = original.to_vec();
        let mut separate = compare();
        separate.try_extend(&copy).unwrap();
        assert!(separate.finalize().unwrap());
        for position in 0..len {
            let mut different = original.to_vec();
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
                remaining: original,
                allowance: len + 1,
                equal: true,
            };
            unequal.try_extend(&vec![0; emitted]).unwrap();
            assert!(!unequal.finalize().unwrap());
        }
    }
}

#[test]
fn identical_slice_still_checks_budget_length_and_previous_mismatch() {
    let bytes = [42; 64];
    let mut limited = Compare {
        remaining: &bytes,
        allowance: 63,
        equal: true,
    };
    assert_eq!(
        limited.try_extend(&bytes),
        Err(postcard::Error::SerializeBufferFull)
    );
    let mut mismatch = Compare {
        remaining: &bytes,
        allowance: 64,
        equal: false,
    };
    mismatch.try_extend(&bytes).unwrap();
    assert!(!mismatch.finalize().unwrap());
    let mut short = Compare {
        remaining: &bytes[..32],
        allowance: 64,
        equal: true,
    };
    short.try_extend(&bytes).unwrap();
    assert!(!short.finalize().unwrap());
    let mut trailing = Compare {
        remaining: &bytes,
        allowance: 64,
        equal: true,
    };
    trailing.try_extend(&bytes[..32]).unwrap();
    assert!(!trailing.finalize().unwrap());
}

#[test]
fn a_mismatch_still_checks_later_bytes_and_the_output_allowance() {
    let mut compare = Compare {
        remaining: &[0; 129],
        allowance: 129,
        equal: true,
    };
    compare.try_extend(&[1; 128]).unwrap();
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
