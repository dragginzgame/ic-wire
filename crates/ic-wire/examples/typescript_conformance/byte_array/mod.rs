//! Standard byte arrays: exact-width browser values, frames and rejection.
use super::{Fixture, ID, LIMITS};
use std::fmt::Write as _;

fn fixed<const N: usize>(f: &mut Fixture, value: [u8; N]) {
    let index = f.count;
    f.add(
        ic_wire::ByteArray::new(value),
        &format!("new Uint8Array({value:?})"),
    );
    for wrong in [N.checked_sub(1), Some(N + 1)].into_iter().flatten() {
        // Full canonical bytes with a wrong length exercise width validation,
        // beyond the generic truncation/prefix differential mutations.
        let frame =
            ic_wire::native::encode_request(ID, &ic_wire::ByteBuf::from(vec![0; wrong]), LIMITS)
                .unwrap();
        assert!(
            ic_wire::native::decode_request::<ic_wire::ByteArray<N>>(ID, &frame, LIMITS).is_err()
        );
        writeln!(f.checks, "assert.throws(() => m{index}.encodeRequest(new Uint8Array({wrong})), WireError);\nassert.throws(() => m{index}.encodeResponse(new Uint8Array({wrong})), WireError);\nassert.throws(() => m{index}.decodeRequest(Uint8Array.from({frame:?})), WireError);").unwrap();
        let frame =
            ic_wire::native::encode_response(ID, &ic_wire::ByteBuf::from(vec![0; wrong]), LIMITS)
                .unwrap();
        writeln!(
            f.checks,
            "assert.throws(() => m{index}.decodeResponse(Uint8Array.from({frame:?})), WireError);"
        )
        .unwrap();
    }
}

pub(super) fn add(f: &mut Fixture) {
    fixed(f, []);
    fixed(f, [255]);
    fixed(f, [0, 128, 255]);
    fixed(f, [0x95; 32]);
    fixed(f, [0x95; 127]);
    fixed(f, [0x95; 128]);
    f.add(None::<ic_wire::ByteArray<2>>, "null");
    f.add(
        Some(ic_wire::ByteArray::new([0, 255])),
        "{some:new Uint8Array([0,255])}",
    );
    #[derive(serde::Serialize, serde::Deserialize, ic_wire::Wire)]
    struct Adapted {
        #[serde(with = "serde_bytes")]
        key: [u8; 2],
        #[serde(with = "serde_bytes")]
        optional: Option<[u8; 2]>,
    }
    for optional in [None, Some([128, 255])] {
        f.add(
            Adapted {
                key: [0, 255],
                optional,
            },
            if optional.is_some() {
                "{key:new Uint8Array([0,255]),optional:{some:new Uint8Array([128,255])}}"
            } else {
                "{key:new Uint8Array([0,255]),optional:null}"
            },
        );
    }
    // Maps use the same Postcard bytes as ordered entry pairs. Wire does not
    // infer map ordering/normalization policy; qualify the explicit pair schema.
    let pairs = vec![
        (
            ic_wire::ByteArray::new([0, 255]),
            Some(ic_wire::ByteBuf::from(vec![1, 2])),
        ),
        (ic_wire::ByteArray::new([128, 255]), None),
    ];
    let map = pairs
        .iter()
        .cloned()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        ic_wire::native::encode_request(ID, &map, LIMITS).unwrap(),
        ic_wire::native::encode_request(ID, &pairs, LIMITS).unwrap()
    );
    f.add(
        pairs,
        "[[new Uint8Array([0,255]),{some:new Uint8Array([1,2])}],[new Uint8Array([128,255]),null]]",
    );
}
