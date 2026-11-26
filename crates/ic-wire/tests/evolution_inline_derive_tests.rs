//! Inline sparse declarations use one stock Serde codec and one operation meter.
#![cfg(all(
    feature = "evolution",
    any(feature = "derive", feature = "evolution-derive")
))]

use ic_wire::{Encoding, Error, LimitKind, Limits, TextRef, Wire, evolution::Evolving};

const LIMITS: Limits = Limits {
    bytes: 256,
    allocation: 8192,
    nodes: 256,
    depth: 12,
};

#[derive(Debug, PartialEq, Wire)]
struct Previous {
    sequence: u16,
}

#[derive(Debug, PartialEq, Wire)]
struct Current<'a, T: Copy, const MAX: usize>
where
    T: PartialEq,
{
    sequence: T,
    #[wire(id = 9)]
    stamp: Option<()>,
    #[wire(id = 4)]
    note: Option<TextRef<'a, MAX>>,
}

#[test]
fn inline_record_revisions_share_exact_bytes_and_borrowed_fields() {
    let old = Previous { sequence: 257 };
    let current = Current::<u16, 16> {
        sequence: 257,
        stamp: Some(()),
        note: Some(TextRef::new("hi").unwrap()),
    };
    let payload = Encoding::Payload;
    let mut output = Vec::new();
    let mut scratch = [0; 16];
    old.encode_evolving(&mut output, payload, LIMITS, &mut [])
        .unwrap();
    assert_eq!(output, [0x81, 2, 0]);
    assert_eq!(
        Current::<u16, 16>::decode_evolving(&output, payload, LIMITS),
        Ok(Current {
            sequence: 257,
            note: None,
            stamp: None
        })
    );
    current
        .encode_evolving(&mut output, payload, LIMITS, &mut scratch)
        .unwrap();
    assert_eq!(output, [0x81, 2, 7, 4, 3, 2, b'h', b'i', 9, 0]);
    assert_eq!(Previous::decode_evolving(&output, payload, LIMITS), Ok(old));
    let decoded = Current::<u16, 16>::decode_evolving(&output, payload, LIMITS).unwrap();
    assert_eq!(decoded, current);
    assert_eq!(
        decoded.note.unwrap().as_str().as_ptr(),
        output[6..].as_ptr()
    );

    let mut malformed = output.clone();
    malformed[8] = 4;
    assert_eq!(
        Previous::decode_evolving(&malformed, payload, LIMITS),
        Err(Error::Malformed)
    );
    malformed[8] = 9;
    malformed[6] = 0xff;
    assert_eq!(
        Current::<u16, 16>::decode_evolving(&malformed, payload, LIMITS),
        Err(Error::Malformed)
    );
    let too_shallow = Limits { depth: 2, ..LIMITS };
    assert_eq!(
        Previous::decode_evolving(&output, payload, too_shallow),
        Err(Error::Limit(LimitKind::Depth))
    );
    assert_eq!(
        current.encode_evolving(&mut output, payload, LIMITS, &mut [0; 2]),
        Err(Error::Limit(LimitKind::Output))
    );
    assert!(output.is_empty());
}

#[derive(Debug, PartialEq, Wire)]
struct PreviousTuple(u8);

#[derive(Debug, PartialEq, Wire, serde::Serialize, serde::Deserialize)]
struct CurrentTuple(u8, #[wire(id = 2)] Option<bool>);

#[test]
fn inline_tuple_does_not_change_its_separately_derived_serde_layout() {
    let latest = CurrentTuple(17, Some(true));
    let payload = Encoding::Payload;
    let mut output = Vec::new();
    latest
        .encode_evolving(&mut output, payload, LIMITS, &mut [0; 8])
        .unwrap();
    assert_eq!(output, [17, 3, 2, 1, 1]);
    assert_eq!(
        PreviousTuple::decode_evolving(&output, payload, LIMITS),
        Ok(PreviousTuple(17))
    );
    PreviousTuple(17)
        .encode_evolving(&mut output, payload, LIMITS, &mut [])
        .unwrap();
    assert_eq!(output, [17, 0]);
    assert_eq!(
        CurrentTuple::decode_evolving(&output, payload, LIMITS),
        Ok(CurrentTuple(17, None))
    );
    assert_eq!(
        ic_wire::encode(&latest, payload, LIMITS).unwrap(),
        [17, 1, 1]
    );
}

#[derive(Debug, PartialEq, Wire)]
struct Wide(
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
    u8,
);

#[test]
fn inline_tuple_core_exceeds_serde_tuple_trait_arity() {
    let wide = Wide(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17);
    let mut output = Vec::new();
    let limits = Limits {
        allocation: 16_384,
        ..LIMITS
    };
    wide.encode_evolving(&mut output, Encoding::Payload, limits, &mut [])
        .unwrap();
    assert_eq!(
        output,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0
        ]
    );
    assert_eq!(
        Wide::decode_evolving(&output, Encoding::Payload, limits),
        Ok(wide)
    );
}

#[derive(Debug, PartialEq, Wire)]
struct Empty;

#[test]
fn inline_unit_sends_and_validates_an_extension_block() {
    let payload = Encoding::Payload;
    let mut bytes = Vec::new();
    Empty
        .encode_evolving(&mut bytes, payload, LIMITS, &mut [])
        .unwrap();
    assert_eq!(bytes, [0]);
    assert_eq!(Empty::decode_evolving(&bytes, payload, LIMITS), Ok(Empty));
    assert_eq!(
        Empty::decode_evolving(&[2, 7, 0], payload, LIMITS),
        Ok(Empty)
    );
    assert_eq!(
        Empty::decode_evolving(&[2, 0, 0], payload, LIMITS),
        Err(Error::Malformed)
    );
}

mod public_record {
    use super::*;

    #[derive(Debug, PartialEq, Wire)]
    struct PrivateRow<'a, T> {
        id: T,
        #[wire(id = 1)]
        note: Option<TextRef<'a, 5>>,
    }

    /// A public DTO can keep its nested projection's type private.
    #[derive(Debug, PartialEq, Wire)]
    pub struct Packet<'a, T> {
        row: PrivateRow<'a, T>,
        checksum: u16,
    }

    #[test]
    fn public_inline_record_keeps_private_nested_fields() {
        let packet = Packet {
            row: PrivateRow {
                id: 7_u8,
                note: Some(TextRef::new("x").unwrap()),
            },
            checksum: 8,
        };
        let mut output = Vec::new();
        packet
            .encode_evolving(&mut output, Encoding::Payload, LIMITS, &mut [0; 8])
            .unwrap();
        assert_eq!(output, [7, 4, 1, 2, 1, b'x', 8, 0]);
        assert_eq!(
            Packet::<u8>::decode_evolving(&output, Encoding::Payload, LIMITS),
            Ok(packet)
        );
    }
}

#[test]
fn codec_only_wire_accepts_fixed_struct_and_closed_enum_leaves() {
    #[derive(Debug, PartialEq, Wire, serde::Serialize, serde::Deserialize)]
    #[wire(fixed)]
    struct Key(u8);

    #[derive(Debug, PartialEq, Wire, serde::Serialize, serde::Deserialize)]
    enum Mode {
        Set(u8),
    }

    #[derive(Debug, PartialEq, Wire)]
    struct Root {
        key: Key,
        mode: Mode,
    }

    let value = Root {
        key: Key(9),
        mode: Mode::Set(42),
    };
    let mut output = Vec::new();
    value
        .encode_evolving(&mut output, Encoding::Payload, LIMITS, &mut [])
        .unwrap();
    assert_eq!(output, [9, 0, 42, 0]);
    assert_eq!(
        Root::decode_evolving(&output, Encoding::Payload, LIMITS),
        Ok(value)
    );
}
