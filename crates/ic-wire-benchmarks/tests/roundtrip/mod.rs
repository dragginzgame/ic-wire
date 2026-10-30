//! Corpus correctness and fail-closed framing checks, independent of timing.
use ic_wire_benchmarks::fixtures;
mod unsigned_magnitude;

#[test]
fn recursive_tree_keeps_wire_support_and_candid_nesting_refusal_explicit() {
    use ic_platform::candid::{self, nesting::NestingError};
    let allowed = fixtures::tree(7);
    let bytes = candid::contract::encode_one(&allowed).unwrap();
    assert_eq!(
        candid::decode_exactly_one::<fixtures::Node>(&bytes, fixtures::CANDID_LIMITS).unwrap(),
        allowed
    );
    let deep = fixtures::tree(8);
    let wire = ic_wire::native::encode_request(fixtures::METHOD, &deep, fixtures::LIMITS).unwrap();
    assert_eq!(
        ic_wire::native::decode_request::<fixtures::Node>(
            fixtures::METHOD,
            &wire,
            fixtures::LIMITS
        )
        .unwrap(),
        deep
    );
    let bytes = candid::contract::encode_one(&deep).unwrap();
    let error =
        candid::decode_exactly_one::<fixtures::Node>(&bytes, fixtures::CANDID_LIMITS).unwrap_err();
    assert!(matches!(
        error.nesting_error(),
        Some(NestingError::ValueDepth { maximum: 16 })
    ));
}

#[test]
fn corpus_roundtrips_and_rejects_damaged_frames() {
    let cases = fixtures::cases(); // Asserts semantic equality for every lane.
    let names: std::collections::BTreeSet<_> = cases.iter().map(|case| &case.name).collect();
    assert_eq!(
        names.len(),
        cases.len(),
        "benchmark case identities must be unique"
    );
    #[cfg(not(any(
        feature = "toko",
        feature = "derive-scale",
        feature = "packed-i16",
        feature = "bulk",
        feature = "hotspots"
    )))]
    assert!(cases.len() >= 100);
    #[cfg(feature = "bulk")]
    assert_eq!(cases.len(), 109); // Includes ten i8 small/wide length controls.
    #[cfg(feature = "toko")]
    assert_eq!(cases.len(), 10);
    #[cfg(feature = "derive-scale")]
    assert_eq!(
        cases.len(),
        if cfg!(feature = "derive-scale-1") {
            1
        } else if cfg!(feature = "derive-scale-10") {
            10
        } else {
            100
        }
    );
    #[cfg(not(any(
        feature = "toko",
        feature = "derive-scale",
        feature = "packed-i16",
        feature = "bulk",
        feature = "hotspots"
    )))]
    for length in [4096, 1_048_576] {
        let vector = cases
            .iter()
            .find(|c| c.name == format!("generic-byte-vector-{length}"))
            .unwrap();
        let blob = cases
            .iter()
            .find(|c| c.name == format!("blob-random-{length}"))
            .unwrap();
        assert_eq!(vector.input_json, blob.input_json);
        for lane in &vector.lanes {
            let bulk = blob.lanes.iter().find(|b| b.name == lane.name).unwrap();
            assert_eq!(
                lane.bytes, bulk.bytes,
                "byte representation: {length}/{}",
                lane.name
            );
        }
    }
    for (index, case) in cases.into_iter().enumerate() {
        let selected = fixtures::cases_selected(Some(index));
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, case.name);
        assert_eq!(selected[0].input_json, case.input_json);
        assert_eq!(selected[0].lanes.len(), case.lanes.len());
        for (a, b) in selected[0].lanes.iter().zip(&case.lanes) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.bytes, b.bytes);
        }
        for lane in case.lanes {
            assert_eq!((lane.encode)(), lane.bytes, "{} {}", case.name, lane.name);
            assert!((lane.decode)(&lane.bytes));
            assert!(!(lane.decode)(&lane.bytes[..lane.bytes.len() - 1]));
            let mut corrupt = lane.bytes.clone();
            corrupt[0] ^= 0xff;
            assert!(!(lane.decode)(&corrupt));
            let mut trailing = lane.bytes.clone();
            trailing.push(0);
            assert!(!(lane.decode)(&trailing));
            if let Some(reuse) = lane.reuse {
                let mut buffer = vec![255; lane.bytes.len() + 64];
                reuse(&mut buffer);
                assert_eq!(buffer, lane.bytes);
            }
        }
    }
}
