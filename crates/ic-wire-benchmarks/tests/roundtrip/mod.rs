//! Corpus correctness and fail-closed framing checks, independent of timing.
#[path = "../../fixtures/mod.rs"]
mod fixtures;
#[test]
fn corpus_roundtrips_and_rejects_damaged_frames() {
    let cases = fixtures::cases(); // Asserts semantic equality for every lane.
    assert!(cases.len() >= 100);
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
