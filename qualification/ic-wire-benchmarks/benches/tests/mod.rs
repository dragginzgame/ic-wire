//! Corpus correctness and fail-closed framing checks.
use super::fixtures;

#[test]
fn recursive_tree_uses_upstream_candid_without_platform_depth_policy() {
    for depth in [7, 8] {
        let value = fixtures::tree(depth);
        let bytes = candid::encode_one(&value).unwrap();
        assert_eq!(
            fixtures::decode_candid::<fixtures::Node>(&bytes).unwrap(),
            value
        );
    }
}

#[test]
fn candid_reference_requires_exactly_one_result_and_complete_bytes() {
    assert!(fixtures::decode_candid::<u8>(&candid::encode_args(()).unwrap()).is_err());
    assert!(fixtures::decode_candid::<u8>(&candid::encode_args((1_u8, 2_u8)).unwrap()).is_err());
    let mut bytes = candid::encode_one(1_u8).unwrap();
    assert_eq!(fixtures::decode_candid::<u8>(&bytes).unwrap(), 1);
    bytes.push(0);
    assert!(fixtures::decode_candid::<u8>(&bytes).is_err());
    assert!(fixtures::decode_candid::<u8>(&vec![0; fixtures::LIMITS.bytes + 1]).is_err());
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
        feature = "hotspots",
        feature = "signed32",
        feature = "byte-keys",
        feature = "contract-shapes",
        feature = "cost-smoke",
        feature = "id16",
        feature = "coverage-shapes"
    )))]
    {
        use sha2::{Digest, Sha256};
        // Freeze names in IC index order: roundtrips alone cannot detect deleted,
        // renamed or reordered cases. Review this digest when deliberately adding
        // coverage; do not regenerate it inside a test or measurement run.
        assert_eq!(cases.len(), 391);
        let inventory = cases
            .iter()
            .map(|case| case.name.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            Sha256::digest(inventory.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            "727cb175a2460a051490b723693e2f1d4c74713a866d6f89681028043e3f053e"
        );
    }
    #[cfg(feature = "id16")]
    assert_eq!(cases.len(), 20);
    #[cfg(feature = "signed32")]
    assert_eq!(cases.len(), 49);
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
        feature = "hotspots",
        feature = "signed32",
        feature = "byte-keys",
        feature = "contract-shapes",
        feature = "cost-smoke",
        feature = "id16",
        feature = "coverage-shapes"
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
