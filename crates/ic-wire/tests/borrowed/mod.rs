//! Borrowed bounded contracts preserve the owned wire format and input custody.
use core::marker::PhantomData;
use ic_wire::{Bytes, BytesRef, Error, Limits, List, Method, Text, TextRef, Wire};
use serde::{Deserialize, Serialize};

struct Contract<T>(PhantomData<T>);
impl<T: Wire> Method for Contract<T> {
    type Request = T;
    type Response = Result<T, u8>;
    const ID: u64 = 71;
    const NAME: &'static str = "chunk";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 2 * 1024 * 1024,
        allocation: 8 * 1024 * 1024,
        nodes: 4096,
        depth: 32,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
#[derive(Debug, PartialEq, Wire)]
struct OwnedChunk {
    #[wire(id = 1)]
    name: Text<64>,
    #[wire(id = 2)]
    data: Bytes<1048576>,
}
#[derive(Debug, PartialEq, Wire)]
struct Chunk<'data, 'name> {
    // Declaration order differs deliberately; IDs own the layout.
    #[wire(id = 2)]
    data: BytesRef<'data, 1048576>,
    #[wire(id = 1)]
    name: TextRef<'name, 64>,
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
        ic_wire::encode_request_into::<Contract<Chunk<'_, '_>>>(&view, &mut scratch).unwrap();
        let frame = ic_wire::encode_request::<Contract<Chunk<'_, '_>>>(&view)
            .unwrap()
            .into_bytes();
        assert_eq!(
            frame,
            ic_wire::encode_request::<Contract<OwnedChunk>>(&owned)
                .unwrap()
                .into_bytes()
        );
        assert_eq!(scratch, frame);
        let decoded = ic_wire::decode_request::<Contract<Chunk<'_, '_>>>(&frame).unwrap();
        assert_eq!(decoded, view);
        let range = frame.as_ptr() as usize..frame.as_ptr() as usize + frame.len();
        assert!(range.contains(&(decoded.name.as_str().as_ptr() as usize)));
        if size != 0 {
            assert!(range.contains(&(decoded.data.as_slice().as_ptr() as usize)));
        }
        assert_eq!(
            ic_wire::encode_request::<Contract<Chunk<'_, '_>>>(&decoded)
                .unwrap()
                .as_bytes(),
            frame
        );
        assert_eq!(Chunk::SHAPE.heap, 0);
        assert_eq!(Chunk::SHAPE.bytes, OwnedChunk::SHAPE.bytes);
        let response = ic_wire::encode_response::<Contract<Chunk<'_, '_>>>(&Ok(view)).unwrap();
        assert!(
            ic_wire::decode_response::<Contract<Chunk<'_, '_>>>(response.as_bytes())
                .unwrap()
                .is_ok()
        );
        assert_eq!(
            ic_wire::decode_request::<Contract<Chunk<'_, '_>>>(response.as_bytes()),
            Err(Error::Identity)
        );
    }
    // Keep immutable owned bytes for retries independently of the source's lifetime.
    let retained = {
        let source = Text::<64>::new("retry".into()).unwrap();
        ic_wire::encode_request::<Contract<TextRef<'_, 64>>>(&source.as_borrowed())
            .unwrap()
            .into_bytes()
    };
    assert_eq!(
        ic_wire::decode_request::<Contract<TextRef<'_, 64>>>(&retained)
            .unwrap()
            .as_str(),
        "retry"
    );
}

#[derive(Debug, PartialEq, Wire)]
struct Nested<'a: 'b, 'b> {
    #[wire(id = 1)]
    items: List<TextRef<'a, 16>, 3>,
    #[wire(id = 2)]
    chunks: [BytesRef<'b, 8>; 2],
    #[wire(id = 3)]
    optional: Option<Box<TextRef<'a, 16>>>,
}
#[derive(Debug, PartialEq, Wire)]
enum Choice<'a: 'b, 'b> {
    #[wire(id = 9)]
    Chunk(Chunk<'a, 'b>),
    #[wire(id = 2)]
    Empty,
    #[wire(id = 5)]
    Nested(Nested<'a, 'b>),
}
#[derive(Debug, PartialEq, Wire)]
struct Collision<'__ic_wire_de> {
    #[wire(id = 1)]
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
        let frame = ic_wire::encode_request::<Contract<Choice<'_, '_>>>(&value).unwrap();
        assert_eq!(
            ic_wire::decode_request::<Contract<Choice<'_, '_>>>(frame.as_bytes()).unwrap(),
            value
        );
        for len in 0..frame.as_bytes().len() {
            assert!(
                ic_wire::decode_request::<Contract<Choice<'_, '_>>>(&frame.as_bytes()[..len])
                    .is_err()
            );
        }
    }
    let collision = Collision {
        text: TextRef::new("name").unwrap(),
    };
    let frame = ic_wire::encode_request::<Contract<Collision<'_>>>(&collision).unwrap();
    assert_eq!(
        ic_wire::decode_request::<Contract<Collision<'_>>>(frame.as_bytes()).unwrap(),
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
        let frame =
            ic_wire::encode_request::<Contract<Text<4>>>(&Text::new(text.into()).unwrap()).unwrap();
        assert_eq!(
            ic_wire::decode_request::<Contract<TextRef<'_, 4>>>(frame.as_bytes())
                .unwrap()
                .as_str(),
            text
        );
    }
    let frame = ic_wire::encode_request::<Contract<Text<4>>>(&Text::new("abc".into()).unwrap())
        .unwrap()
        .into_bytes();
    let decode = |bytes: &[u8]| {
        ic_wire::decode_request::<Contract<TextRef<'_, 4>>>(bytes).map(|v| v.as_str().len())
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

#[test]
fn borrowed_fields_do_not_charge_owned_payload_heap() {
    struct Read<'a>(PhantomData<&'a ()>);
    impl<'a> Method for Read<'a> {
        type Request = BytesRef<'a, 32>;
        type Response = Bytes<32>;
        const ID: u64 = 1;
        const NAME: &'static str = "read";
        const REQUEST_LIMITS: Limits = Limits {
            bytes: 53,
            allocation: 53,
            nodes: 1,
            depth: 1,
        };
        const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
    }
    let value = Bytes::<32>::new(vec![42; 32]).unwrap();
    let frame = ic_wire::encode_request::<Read<'_>>(&value.as_borrowed()).unwrap();
    assert_eq!(
        ic_wire::decode_request::<Read<'_>>(frame.as_bytes())
            .unwrap()
            .as_slice(),
        value.as_slice()
    );
    assert!(matches!(
        ic_wire::encode_response::<Read<'_>>(&value),
        Err(Error::SchemaLimit)
    ));
    // Bounds stay enforced even when the borrowed representation needs no value heap.
    assert_eq!(BytesRef::<32>::SHAPE.nodes, Bytes::<32>::SHAPE.nodes);
    assert_eq!(TextRef::<32>::SHAPE.depth, Text::<32>::SHAPE.depth);
}

#[derive(Serialize)]
struct Outgoing(u32);
impl Wire for Outgoing {
    const SHAPE: ic_wire::Shape = <u32 as Wire>::SHAPE;
}
#[test]
fn encoding_and_sizing_do_not_require_a_decoder() {
    let frame = ic_wire::encode_request::<Contract<Outgoing>>(&Outgoing(7)).unwrap();
    assert_eq!(
        frame.as_bytes(),
        ic_wire::encode_request::<Contract<u32>>(&7)
            .unwrap()
            .as_bytes()
    );
    assert_eq!(
        frame.as_bytes().len(),
        ic_wire::encoded_request_len::<Contract<Outgoing>>(&Outgoing(7)).unwrap()
    );
}

#[cfg(feature = "typescript")]
#[test]
fn borrowed_and_owned_contracts_generate_identical_typescript() {
    assert_eq!(
        ic_wire::typescript::generate::<Contract<Chunk<'static, 'static>>>().unwrap(),
        ic_wire::typescript::generate::<Contract<OwnedChunk>>().unwrap()
    );
}
