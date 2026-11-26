//! Recursive projection through auto-derived arrays and sequences.
#![cfg(feature = "evolution-derive")]

use ic_wire::{Error, LimitKind, Limits, TextRef, Wire, evolution::Method};

const LIMITS: Limits = Limits {
    bytes: 128,
    allocation: 16_384,
    nodes: 1024,
    depth: 24,
};

#[derive(Debug, PartialEq, Eq, Wire)]
struct ItemV1<'a> {
    name: TextRef<'a, 16>,
}

#[derive(Debug, PartialEq, Eq, Wire)]
struct ItemV2<'a> {
    name: TextRef<'a, 16>,
    #[wire(id = 1)]
    detail: Option<TextRef<'a, 16>>,
}

#[derive(Debug, PartialEq, Eq, Wire)]
struct Collection<T> {
    fixed_count: [T; 2],
    spare: Vec<T>,
    witness: u16,
}

fn method_v1<'a>() -> Method<Collection<ItemV1<'a>>, Collection<ItemV1<'a>>> {
    Method::new("collections", LIMITS, LIMITS)
}

fn method_v2<'a>() -> Method<Collection<ItemV2<'a>>, Collection<ItemV2<'a>>> {
    Method::new("collections", LIMITS, LIMITS)
}

#[test]
fn nested_arrays_and_vecs_project_known_values_and_reuse_input_borrows() {
    let first = ItemV2 {
        name: TextRef::new("a").unwrap(),
        detail: None,
    };
    let latest = ItemV2 {
        name: TextRef::new("b").unwrap(),
        detail: Some(TextRef::new("detail Δ").unwrap()),
    };
    let value = Collection {
        fixed_count: [first, latest],
        spare: vec![ItemV2 {
            name: TextRef::new("c").unwrap(),
            detail: None,
        }],
        witness: 42,
    };
    let mut frame = Vec::new();
    let mut scratch = [0; 32];
    method_v2()
        .encode_reply(&value, &mut frame, &mut scratch)
        .unwrap();

    let latest = method_v2().decode_reply(&frame).unwrap();
    assert_eq!(latest, value);
    let detail = latest.fixed_count[1].detail.unwrap().as_str();
    assert!(frame.as_ptr_range().contains(&detail.as_ptr()));

    let first = method_v1().decode_reply(&frame).unwrap();
    assert_eq!(first.witness, 42);
    assert_eq!(first.fixed_count[0].name.as_str(), "a");
    assert_eq!(first.fixed_count[1].name.as_str(), "b");
    assert_eq!(first.spare[0].name.as_str(), "c");

    let mut old_frame = Vec::new();
    method_v1()
        .encode_reply(&first, &mut old_frame, &mut [])
        .unwrap();
    let upgraded = method_v2().decode_reply(&old_frame).unwrap();
    assert_eq!(upgraded.witness, 42);
    assert_eq!(upgraded.fixed_count[1].detail, None);
    assert_eq!(upgraded.spare[0].detail, None);
}

#[test]
fn nested_projection_obeys_the_frame_limits_and_scratch_capacity() {
    let value = Collection {
        fixed_count: [
            ItemV2 {
                name: TextRef::new("a").unwrap(),
                detail: None,
            },
            ItemV2 {
                name: TextRef::new("b").unwrap(),
                detail: Some(TextRef::new("detail").unwrap()),
            },
        ],
        spare: Vec::new(),
        witness: 42,
    };
    let mut frame = vec![0];
    assert_eq!(
        method_v2().encode_reply(&value, &mut frame, &mut [0; 2]),
        Err(Error::Limit(LimitKind::Output))
    );
    assert!(frame.is_empty());

    let mut scratch = [0; 32];
    method_v2()
        .encode_reply(&value, &mut frame, &mut scratch)
        .unwrap();
    let low_allocation = Limits {
        allocation: 256,
        ..LIMITS
    };
    let constrained = Method::<Collection<ItemV2<'_>>, Collection<ItemV2<'_>>>::new(
        "collections",
        LIMITS,
        low_allocation,
    );
    assert_eq!(
        constrained.decode_reply(&frame),
        Err(Error::Limit(LimitKind::Allocation))
    );
    let mut constrained_output = Vec::new();
    assert_eq!(
        constrained.encode_reply(&value, &mut constrained_output, &mut scratch),
        Err(Error::Limit(LimitKind::Allocation)),
    );
    assert!(constrained_output.is_empty());

    let shallow = Limits { depth: 2, ..LIMITS };
    let method = Method::<Collection<ItemV2<'_>>, Collection<ItemV2<'_>>>::new(
        "collections",
        LIMITS,
        shallow,
    );
    assert_eq!(
        method.decode_reply(&frame),
        Err(Error::Limit(LimitKind::Depth))
    );
    assert_eq!(
        method.encode_reply(&value, &mut frame, &mut scratch),
        Err(Error::Limit(LimitKind::Depth))
    );
    assert!(frame.is_empty());
}
