use super::*;
use alloc::vec;

#[test]
fn growing_output_checks_limits_before_reserving_and_handles_capacity_overflow() {
    let failed = Cell::new(false);
    let mut output = GrowingOutput {
        output: Output {
            bytes: Vec::new(),
            limit: 257,
        },
        allocation_failed: &failed,
    };
    output.try_extend(&[1; 64]).unwrap();
    output.try_push(2).unwrap();
    output.try_extend(&[3; 192]).unwrap();
    let capacity = output.output.bytes.capacity();
    assert_eq!(
        output.try_push(4),
        Err(postcard::Error::SerializeBufferFull)
    );
    assert_eq!(
        output.try_extend(&[4]),
        Err(postcard::Error::SerializeBufferFull)
    );
    assert_eq!(output.output.bytes.capacity(), capacity);
    assert!(!failed.get());
    let bytes = output.finalize().unwrap();
    assert_eq!(bytes.len(), 257);
    assert_eq!(&bytes[..64], &[1; 64]);
    assert_eq!(bytes[64], 2);
    assert_eq!(&bytes[65..], &[3; 192]);

    let mut overflow = GrowingOutput {
        output: Output {
            bytes: Vec::new(),
            limit: usize::MAX,
        },
        allocation_failed: &failed,
    };
    assert_eq!(
        overflow.reserve(usize::MAX),
        Err(postcard::Error::SerializeBufferFull)
    );
    assert!(failed.get());
    assert!(overflow.output.bytes.is_empty());

    failed.set(false);
    let mut spare = GrowingOutput {
        output: Output {
            bytes: Vec::with_capacity(512),
            limit: 1,
        },
        allocation_failed: &failed,
    };
    spare.try_push(1).unwrap();
    assert_eq!(spare.try_push(2), Err(postcard::Error::SerializeBufferFull));
    assert_eq!(
        spare.try_extend(&[2]),
        Err(postcard::Error::SerializeBufferFull)
    );
    spare.try_extend(&[]).unwrap();
    assert_eq!(spare.finalize().unwrap(), vec![1]);
    assert!(!failed.get());
}

#[test]
fn finite_encoding_allocates_for_emitted_bytes_without_a_second_visit() {
    struct Once(Cell<bool>);
    impl serde::Serialize for Once {
        fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            assert!(!self.0.replace(true), "encoder visited the value twice");
            s.serialize_bytes(&[])
        }
    }
    impl Wire for Once {
        const SHAPE: crate::Shape = crate::Bytes::<196608>::SHAPE;
    }
    let limits = Limits {
        bytes: 200000,
        allocation: 400000,
        nodes: 8,
        depth: 8,
    };
    let bytes = encode(1, 0, limits, &Once(Cell::new(false))).unwrap();
    assert_eq!(bytes.len(), HEADER + 1);
    assert!(
        bytes.capacity() < 1024,
        "tiny message reserved the schema maximum"
    );
    assert_eq!(bytes, [header(1, 0).as_slice(), &[0]].concat());
}

#[test]
fn exact_bulk_equality_covers_word_boundaries_tails_and_every_mismatch() {
    // Native unit builds exercise the same chunk branch as production Wasm.
    for len in 0..=161 {
        let storage: Vec<_> = (0..len + 1).map(|i| i as u8).collect();
        // Exercise unaligned starts as well as every 32-byte block/tail boundary.
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

#[test]
fn small_fresh_frames_reserve_the_ceiling_and_preserve_scratch_reuse() {
    fn check<const N: usize>() {
        let limits = Limits {
            bytes: 1024,
            allocation: 2048,
            nodes: 8,
            depth: 8,
        };
        let size = admit::<crate::Bytes<N>>(limits).unwrap();
        let empty = crate::Bytes::<N>::new(vec![]).unwrap();
        let fresh = encode(1, 0, limits, &empty).unwrap();
        assert!(fresh.capacity() >= size);
        let mut scratch = Vec::new();
        let largest = crate::Bytes::<N>::new(vec![42; N]).unwrap();
        encode_into(1, 0, limits, &largest, &mut scratch).unwrap();
        assert!(scratch.capacity() >= size);
        let capacity = scratch.capacity();
        let pointer = scratch.as_ptr();
        for len in [0, 32, 52, 53, N] {
            let value = crate::Bytes::<N>::new(vec![42; len]).unwrap();
            encode_into(1, 0, limits, &value, &mut scratch).unwrap();
            assert_eq!(scratch.capacity(), capacity);
            assert_eq!(scratch.as_ptr(), pointer);
            assert_eq!(scratch, encode(1, 0, limits, &value).unwrap());
            assert_eq!(&scratch[HEADER..], postcard::to_allocvec(&value).unwrap());
        }
    }
    check::<53>();
    check::<80>();
    check::<107>();
}
