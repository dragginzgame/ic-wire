//! Count consumer visits, independently of codec budget internals or wall time.
use super::*;
use std::cell::Cell;

thread_local! {
    static VISITS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Debug, PartialEq)]
struct Tracked<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Tracked<T> {
    fn deserialize<D: serde::Deserializer<'de>>(input: D) -> Result<Self, D::Error> {
        VISITS.with(|visits| visits.set(visits.get() + 1));
        T::deserialize(input).map(Self)
    }
}

#[test]
fn frame_rejection_precedes_consumer_deserialization_and_calls_are_independent() {
    for response in [false, true] {
        let frame = if response {
            native::encode_response(7, &42_u8, LIMITS).unwrap()
        } else {
            native::encode_request(7, &42_u8, LIMITS).unwrap()
        };
        let decode = |id, bytes: &[u8], limits| -> Result<Tracked<u8>, Error> {
            if response {
                native::decode_response(id, bytes, limits)
            } else {
                native::decode_request(id, bytes, limits)
            }
        };
        let mut bad_magic = frame.clone();
        bad_magic[0] ^= 1;
        let mut bad_direction = frame.clone();
        bad_direction[10] ^= 1;
        for (id, bytes, limits, error) in [
            (8, frame.as_slice(), LIMITS, Error::Identity),
            (7, bad_magic.as_slice(), LIMITS, Error::Identity),
            (7, bad_direction.as_slice(), LIMITS, Error::Identity),
            (7, &frame[..10], LIMITS, Error::Malformed),
            (
                7,
                frame.as_slice(),
                Limits {
                    bytes: frame.len() - 1,
                    ..LIMITS
                },
                Error::Limit,
            ),
        ] {
            VISITS.with(|v| v.set(0));
            assert_eq!(decode(id, bytes, limits), Err(error));
            assert_eq!(VISITS.with(Cell::get), 0);
            assert_eq!(decode(7, &frame, LIMITS), Ok(Tracked(42)));
            assert_eq!(VISITS.with(Cell::get), 1);
        }
        // Payload failures may enter Deserialize; their state must stay call-local.
        for bytes in [&frame[..11], &[frame.as_slice(), &[0]].concat()] {
            assert!(decode(7, bytes, LIMITS).is_err());
            assert_eq!(decode(7, &frame, LIMITS), Ok(Tracked(42)));
            let mixed = (
                vec![0_u16, 128, u16::MAX],
                Some(("é", vec![u64::MAX])),
                true,
            );
            let raw = postcard::to_allocvec(&mixed).unwrap();
            assert_eq!(
                native::decode::<(Vec<u16>, Option<(&str, Vec<u64>)>, bool)>(&raw, LIMITS).unwrap(),
                mixed
            );
        }
    }
}

#[test]
fn zero_byte_collection_work_is_linear_and_forged_lengths_stop_at_budget() {
    // Unit elements require no payload bytes: a tiny prefix can claim enormous work.
    for count in [0_usize, 1, 16, 32, 64, 128] {
        let raw = postcard::to_allocvec(&count).unwrap();
        VISITS.with(|v| v.set(0));
        let decoded = native::decode::<Vec<Tracked<()>>>(&raw, LIMITS).unwrap();
        assert_eq!(decoded.len(), count);
        assert_eq!(VISITS.with(Cell::get), count);
    }
    for nodes in [8, 16, 32] {
        let limits = Limits { nodes, ..LIMITS };
        let mut refused_visits = None;
        for count in [1024_usize, 1 << 20, usize::MAX] {
            let raw = postcard::to_allocvec(&count).unwrap();
            VISITS.with(|v| v.set(0));
            assert_eq!(
                native::decode::<Vec<Tracked<()>>>(&raw, limits),
                Err(Error::Limit)
            );
            let visits = VISITS.with(Cell::get);
            assert!(visits > 0 && visits <= nodes);
            assert_eq!(*refused_visits.get_or_insert(visits), visits);
            // A refusal must not retain exhausted allowance for the next operation.
            assert_eq!(
                native::decode::<Vec<Tracked<()>>>(&[1], limits).unwrap(),
                vec![Tracked(())]
            );
        }
    }
}

#[test]
fn nested_collections_share_one_budget_and_truncated_prefix_work_is_bounded() {
    // Each inner list is individually small; the enclosing list must not reset
    // its allowance. The payload contains lengths only, so bytes cannot bound work.
    let value = vec![vec![(); 16]; 16];
    let raw = postcard::to_allocvec(&value).unwrap();
    VISITS.with(|v| v.set(0));
    let decoded = native::decode::<Vec<Vec<Tracked<()>>>>(&raw, LIMITS).unwrap();
    assert_eq!(decoded.len(), 16);
    assert!(decoded.iter().all(|row| row.len() == 16));
    assert_eq!(VISITS.with(Cell::get), 256);
    for nodes in [16, 32, 64] {
        VISITS.with(|v| v.set(0));
        assert_eq!(
            native::decode::<Vec<Vec<Tracked<()>>>>(&raw, Limits { nodes, ..LIMITS }),
            Err(Error::Limit)
        );
        assert!(VISITS.with(Cell::get) <= nodes);
    }

    // A normal byte list must parse its valid prefix before detecting EOF.
    // Count visits rather than making a timing assertion. The declared length
    // must not cause work beyond the available prefix and the failing element.
    for available in [0, 1, 16, 127, 256] {
        for declared in [1024_usize, 1 << 20, usize::MAX] {
            let mut raw = postcard::to_allocvec(&declared).unwrap();
            raw.extend(std::iter::repeat_n(42_u8, available));
            VISITS.with(|v| v.set(0));
            assert_eq!(
                native::decode::<Vec<Tracked<u8>>>(&raw, LIMITS),
                Err(Error::Malformed)
            );
            assert_eq!(VISITS.with(Cell::get), available + 1);
            assert_eq!(native::decode::<Vec<u8>>(&[1, 7], LIMITS), Ok(vec![7]));
        }
    }
}
