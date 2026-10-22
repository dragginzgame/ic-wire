//! Borrowed bounded contracts preserve the owned wire format and input custody.
use ic_wire::{Bytes, BytesRef, Error, Limits, List, Text, TextRef};
use serde::{Deserialize, Serialize};

const ID: u64 = 71;
const LIMITS: Limits = Limits {
    bytes: 2 * 1024 * 1024,
    allocation: 16 * 1024 * 1024,
    nodes: 4096,
    depth: 32,
};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct OwnedChunk {
    name: Text<64>,
    data: Bytes<1048576>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Chunk<'data, 'name> {
    #[serde(borrow)]
    name: TextRef<'name, 64>,
    #[serde(borrow)]
    data: BytesRef<'data, 1048576>,
}
#[test]
fn blob_requests_borrow_owned_storage_and_decode_without_payload_copies() {
    for size in [0, 1, 32, 4096, 1048576] {
        let owned = OwnedChunk {
            name: Text::new("application/octet-stream".into()).unwrap(),
            data: Bytes::new(vec![42; size]).unwrap(),
        };
        let view = Chunk {
            data: owned.data.as_borrowed(),
            name: owned.name.as_borrowed(),
        };
        assert_eq!(
            view.data.as_slice().as_ptr(),
            owned.data.as_slice().as_ptr()
        );
        let mut scratch = Vec::with_capacity(2 * 1024 * 1024);
        ic_wire::native::encode_request_into::<Chunk<'_, '_>>(ID, &view, LIMITS, &mut scratch)
            .unwrap_or_else(|e| panic!("size={size}: {e}"));
        let frame = ic_wire::native::encode_request::<Chunk<'_, '_>>(ID, &view, LIMITS).unwrap();
        assert_eq!(
            frame,
            ic_wire::native::encode_request::<OwnedChunk>(ID, &owned, LIMITS).unwrap()
        );
        assert_eq!(scratch, frame);
        let decoded = ic_wire::native::decode_request::<Chunk<'_, '_>>(ID, &frame, LIMITS).unwrap();
        assert_eq!(decoded, view);
        let range = frame.as_ptr() as usize..frame.as_ptr() as usize + frame.len();
        assert!(range.contains(&(decoded.name.as_str().as_ptr() as usize)));
        if size != 0 {
            assert!(range.contains(&(decoded.data.as_slice().as_ptr() as usize)));
        }
        assert_eq!(
            ic_wire::native::encode_request::<Chunk<'_, '_>>(ID, &decoded, LIMITS)
                .unwrap()
                .as_slice(),
            frame
        );
        let response =
            ic_wire::native::encode_response::<Result<Chunk<'_, '_>, u8>>(ID, &Ok(view), LIMITS)
                .unwrap();
        assert!(
            ic_wire::native::decode_response::<Result<Chunk<'_, '_>, u8>>(
                ID,
                response.as_slice(),
                LIMITS
            )
            .unwrap()
            .is_ok()
        );
        assert_eq!(
            ic_wire::native::decode_request::<Chunk<'_, '_>>(ID, response.as_slice(), LIMITS),
            Err(Error::Identity)
        );
    }
    // Keep immutable owned bytes for retries independently of the source's lifetime.
    let retained = {
        let source = Text::<64>::new("retry".into()).unwrap();
        ic_wire::native::encode_request::<TextRef<'_, 64>>(ID, &source.as_borrowed(), LIMITS)
            .unwrap()
    };
    assert_eq!(
        ic_wire::native::decode_request::<TextRef<'_, 64>>(ID, &retained, LIMITS)
            .unwrap()
            .as_str(),
        "retry"
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Nested<'a: 'b, 'b> {
    #[serde(borrow)]
    items: List<TextRef<'a, 16>, 3>,
    #[serde(borrow)]
    chunks: [BytesRef<'b, 8>; 2],
    #[serde(borrow)]
    optional: Option<Box<TextRef<'a, 16>>>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Choice<'a: 'b, 'b> {
    Chunk(#[serde(borrow)] Chunk<'a, 'b>),
    Empty,
    Nested(#[serde(borrow)] Nested<'a, 'b>),
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Collision<'__ic_wire_de> {
    #[serde(borrow)]
    text: TextRef<'__ic_wire_de, 8>,
}
#[test]
fn lifetime_derives_cover_enums_nested_containers_bounds_and_name_collisions() {
    let nested = Nested {
        items: List::new(vec![
            TextRef::new("one").unwrap(),
            TextRef::new("two").unwrap(),
        ])
        .unwrap(),
        chunks: [BytesRef::new(&[1, 2]).unwrap(), BytesRef::new(&[]).unwrap()],
        optional: Some(Box::new(TextRef::new("three").unwrap())),
    };
    for value in [
        Choice::Empty,
        Choice::Nested(nested),
        Choice::Chunk(Chunk {
            data: BytesRef::new(&[1, 2, 3]).unwrap(),
            name: TextRef::new("chunk").unwrap(),
        }),
    ] {
        let frame = ic_wire::native::encode_request::<Choice<'_, '_>>(ID, &value, LIMITS).unwrap();
        assert_eq!(
            ic_wire::native::decode_request::<Choice<'_, '_>>(ID, frame.as_slice(), LIMITS)
                .unwrap(),
            value
        );
        for len in 0..frame.as_slice().len() {
            assert!(
                ic_wire::native::decode_request::<Choice<'_, '_>>(
                    ID,
                    &frame.as_slice()[..len],
                    LIMITS
                )
                .is_err()
            );
        }
    }
    let collision = Collision {
        text: TextRef::new("name").unwrap(),
    };
    let frame = ic_wire::native::encode_request::<Collision<'_>>(ID, &collision, LIMITS).unwrap();
    assert_eq!(
        ic_wire::native::decode_request::<Collision<'_>>(ID, frame.as_slice(), LIMITS).unwrap(),
        collision
    );
}

#[test]
fn borrowed_leaves_refuse_bounds_noncanonical_lengths_utf8_and_nonborrowed_input() {
    assert_eq!(BytesRef::<2>::new(&[1, 2, 3]), Err(Error::Limit));
    assert_eq!(TextRef::<2>::new("€"), Err(Error::Limit));
    assert!(BytesRef::<0>::new(&[]).is_ok());
    assert!(TextRef::<0>::new("").is_ok());
    for text in ["", "é", "€", "🦀"] {
        let frame = ic_wire::native::encode_request::<Text<4>>(
            ID,
            &Text::new(text.into()).unwrap(),
            LIMITS,
        )
        .unwrap();
        assert_eq!(
            ic_wire::native::decode_request::<TextRef<'_, 4>>(ID, frame.as_slice(), LIMITS)
                .unwrap()
                .as_str(),
            text
        );
    }
    let frame =
        ic_wire::native::encode_request::<Text<4>>(ID, &Text::new("abc".into()).unwrap(), LIMITS)
            .unwrap();
    let decode = |bytes: &[u8]| {
        ic_wire::native::decode_request::<TextRef<'_, 4>>(ID, bytes, LIMITS)
            .map(|v| v.as_str().len())
    };
    for n in 0..frame.len() {
        assert!(decode(&frame[..n]).is_err());
    }
    let mut bad = frame.clone();
    bad.push(0);
    assert_eq!(decode(&bad), Err(Error::Trailing));
    let mut bad = frame.clone();
    bad.splice(11..12, [0x83, 0]);
    assert_eq!(decode(&bad), Err(Error::Noncanonical));
    let mut bad = frame.clone();
    bad[12] = 0xff;
    assert_eq!(decode(&bad), Err(Error::Malformed));
    let mut bad = frame[..11].to_vec();
    bad.extend_from_slice(&[5, b'a', b'a', b'a', b'a', b'a']);
    assert_eq!(decode(&bad), Err(Error::Malformed));
    assert!(postcard::from_bytes::<BytesRef<'_, 2>>(&[3, 1, 2, 3]).is_err());
    assert!(postcard::from_bytes::<BytesRef<'_, 2>>(&[255; 10]).is_err());
    assert!(
        TextRef::<4>::deserialize(
            serde::de::value::StrDeserializer::<serde::de::value::Error>::new("abc")
        )
        .is_err()
    );
    assert!(
        BytesRef::<4>::deserialize(
            serde::de::value::BytesDeserializer::<serde::de::value::Error>::new(b"abc")
        )
        .is_err()
    );
}

#[derive(Serialize)]
struct Outgoing(u32);

#[test]
fn encoding_and_sizing_do_not_require_a_decoder() {
    let frame = ic_wire::native::encode_request::<Outgoing>(ID, &Outgoing(7), LIMITS).unwrap();
    assert_eq!(
        frame.as_slice(),
        ic_wire::native::encode_request::<u32>(ID, &7, LIMITS)
            .unwrap()
            .as_slice()
    );
    assert_eq!(
        frame.as_slice().len(),
        ic_wire::native::encoded_frame_len::<Outgoing>(&Outgoing(7), LIMITS).unwrap()
    );
}

#[test]
fn owned_and_borrowed_leaves_agree_on_mutated_frames() {
    for len in [0, 1, 127, 128, 129] {
        let value = Text::<129>::new("a".repeat(len)).unwrap();
        let frame = ic_wire::native::encode_request::<Text<129>>(ID, &value, LIMITS).unwrap();
        let mut candidates = vec![frame.clone()];
        for end in 0..frame.len() {
            candidates.push(frame[..end].to_vec());
        }
        for offset in 11..frame.len() {
            for mask in [1, 128, 255] {
                let mut bytes = frame.clone();
                bytes[offset] ^= mask;
                candidates.push(bytes);
            }
        }
        for prefix in [vec![0x80, 0], vec![0xff; 10], vec![0x80; 10]] {
            let mut bytes = frame[..11].to_vec();
            bytes.extend(prefix);
            bytes.extend_from_slice(&frame[11..]);
            candidates.push(bytes);
        }
        for bytes in candidates {
            let owned = ic_wire::native::decode_request::<Text<129>>(ID, &bytes, LIMITS)
                .map(|value| value.as_str().to_owned());
            let borrowed = ic_wire::native::decode_request::<TextRef<'_, 129>>(ID, &bytes, LIMITS)
                .map(|value| value.as_str().to_owned());
            assert_eq!(owned, borrowed);
            let owned = ic_wire::native::decode_request::<Bytes<129>>(ID, &bytes, LIMITS)
                .map(|value| value.as_slice().to_vec());
            let borrowed = ic_wire::native::decode_request::<BytesRef<'_, 129>>(ID, &bytes, LIMITS)
                .map(|value| value.as_slice().to_vec());
            assert_eq!(owned, borrowed);
        }
    }
}
