#[test]
fn controls_keep_matching_inputs_and_candid_frames() {
    use super::*;
    use sha2::{Digest, Sha256};
    let cases = cases_selected(None);
    assert_eq!(cases.len(), 49);
    let names = cases
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        Sha256::digest(names.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "14277afce7a7ecbcca2475ab91306779beffa41fdef64e99ec28cd110017666c"
    );
    for n in [0, 1, 128, 4096] {
        let ordinary = cases
            .iter()
            .find(|c| c.name == format!("signed32-list-wide-{n}"))
            .unwrap();
        let packed = cases
            .iter()
            .find(|c| c.name == format!("signed32-packed-wide-{n}"))
            .unwrap();
        assert_eq!(ordinary.input_json, packed.input_json);
        let external = |c: &Case| {
            c.lanes
                .iter()
                .find(|l| l.name == "candid")
                .unwrap()
                .bytes
                .clone()
        };
        assert_eq!(external(ordinary), external(packed));
        let set = cases
            .iter()
            .find(|c| c.name == format!("signed32-set-{n}"))
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<Vec<i32>>(&set.input_json)
                .unwrap()
                .len(),
            n
        );
    }
}

#[test]
fn wrapped_deque_keeps_logical_order_and_list_bytes() {
    use super::*;
    for n in [0, 1, 128] {
        let original = (0..n)
            .map(|i| BOUNDARIES[i % BOUNDARIES.len()])
            .collect::<Vec<_>>();
        let bytes = ic_wire::native::encode(&original, LIMITS).unwrap();
        let candid = candid::encode_one(&original).unwrap();
        // Keep logical contents fixed while exercising every physical split.
        for split in 0..n.max(1) {
            let mut rotated = original.clone();
            rotated.rotate_right(split);
            let mut deque = VecDeque::from(rotated);
            deque.rotate_left(split);
            if split > 0 {
                let (a, b) = deque.as_slices();
                assert!(!a.is_empty() && !b.is_empty());
            }
            let exact = ic_wire::Limits {
                bytes: bytes.len(),
                allocation: bytes.len()
                    + 16 * size_of::<VecDeque<i32>>()
                    // Serde collect_seq visits iterator items of type &i32.
                    + 16 * n * size_of::<&i32>(),
                nodes: 4 * (n + 1),
                depth: if n == 0 { 3 } else { 4 },
            };
            assert_eq!(ic_wire::native::encode(&deque, exact).unwrap(), bytes);
            assert_eq!(
                ic_wire::native::encoded_len(&deque, exact).unwrap(),
                bytes.len()
            );
            for short in [
                ic_wire::Limits {
                    allocation: exact.allocation - 1,
                    ..exact
                },
                ic_wire::Limits {
                    nodes: exact.nodes - 1,
                    ..exact
                },
                ic_wire::Limits {
                    depth: exact.depth - 1,
                    ..exact
                },
            ] {
                assert_eq!(
                    ic_wire::native::encode(&deque, short),
                    Err(ic_wire::Error::Limit)
                );
            }
            assert_eq!(
                ic_wire::native::decode::<VecDeque<i32>>(&bytes, LIMITS).unwrap(),
                deque
            );
            assert_eq!(candid::encode_one(&deque).unwrap(), candid);
        }
    }
}

#[test]
fn scalar_boundaries_reject_truncation_and_overflow() {
    use super::*;
    // Include the four-/five-byte transition beyond the timed scalar inventory.
    for value in
        BOUNDARIES
            .into_iter()
            .chain([-134_217_729, -134_217_728, 134_217_727, 134_217_728])
    {
        let bytes = ic_wire::native::encode(&value, LIMITS).unwrap();
        for end in 0..bytes.len() {
            assert!(ic_wire::native::decode::<i32>(&bytes[..end], LIMITS).is_err());
        }
        assert_eq!(
            ic_wire::native::decode::<i32>(&bytes, LIMITS).unwrap(),
            value
        );
    }
    // Redundant zero is accepted; overflow and continuation past the width are not.
    assert_eq!(ic_wire::native::decode::<i32>(&[0x80, 0], LIMITS), Ok(0));
    for bytes in [&[0xff, 0xff, 0xff, 0xff, 0x10][..], &[0x80; 5]] {
        assert_eq!(
            ic_wire::native::decode::<i32>(bytes, LIMITS),
            Err(ic_wire::Error::Malformed)
        );
    }
}
