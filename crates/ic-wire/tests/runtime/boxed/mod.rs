//! Box storage is admitted statically without changing the finite wire layout.
use ic_wire::{
    Bytes, Error, Limits, Method, Shape, Text, Wire, decode_request, decode_response,
    encode_request, encode_response,
};
use std::{
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, PartialEq, Wire)]
struct Proof {
    #[wire(id = 1)]
    signer: Bytes<29>,
    #[wire(id = 2)]
    claim: Text<32>,
}

#[derive(Debug, PartialEq, Wire)]
enum Inline {
    #[wire(id = 0)]
    Certified(Proof),
    #[wire(id = 1)]
    Signature(Bytes<64>),
}

#[derive(Debug, PartialEq, Wire)]
enum Indirect {
    #[wire(id = 0)]
    Certified(Box<Proof>),
    #[wire(id = 1)]
    Signature(Bytes<64>),
}

struct Contract<T, const ALLOCATION: usize = 4096>(PhantomData<T>);
impl<T: Wire, const ALLOCATION: usize> Method for Contract<T, ALLOCATION> {
    type Request = T;
    type Response = T;
    const ID: u64 = 0xb07;
    const NAME: &'static str = "boxed";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 256,
        allocation: ALLOCATION,
        nodes: 64,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}

fn proof() -> Proof {
    Proof {
        signer: Bytes::new(vec![7; 29]).unwrap(),
        claim: Text::new("é🦀".into()).unwrap(),
    }
}

#[test]
fn record_enum_and_nested_boxes_preserve_frames_and_rejections() {
    let inline = Inline::Certified(proof());
    let indirect = Indirect::Certified(Box::new(proof()));
    for (left, right) in [
        (
            encode_request::<Contract<Inline>>(&inline)
                .unwrap()
                .into_bytes(),
            encode_request::<Contract<Indirect>>(&indirect)
                .unwrap()
                .into_bytes(),
        ),
        (
            encode_response::<Contract<Inline>>(&inline)
                .unwrap()
                .into_bytes(),
            encode_response::<Contract<Indirect>>(&indirect)
                .unwrap()
                .into_bytes(),
        ),
    ] {
        assert_eq!(left, right);
    }
    let frame = encode_request::<Contract<Inline>>(&inline)
        .unwrap()
        .into_bytes();
    assert_eq!(
        decode_request::<Contract<Indirect>>(&frame).unwrap(),
        indirect
    );
    assert_eq!(
        encode_request::<Contract<Indirect>>(&indirect)
            .unwrap()
            .as_bytes(),
        frame
    );
    let reply = encode_response::<Contract<Indirect>>(&indirect).unwrap();
    assert_eq!(
        decode_response::<Contract<Inline>>(reply.as_bytes()).unwrap(),
        inline
    );
    for end in 0..frame.len() {
        assert!(decode_request::<Contract<Indirect>>(&frame[..end]).is_err());
    }
    let mut trailing = frame.clone();
    trailing.push(0);
    assert_eq!(
        decode_request::<Contract<Indirect>>(&trailing),
        Err(Error::Trailing)
    );
    let mut unknown = frame.clone();
    unknown[11] = 2;
    assert_eq!(
        decode_request::<Contract<Indirect>>(&unknown),
        Err(Error::Malformed)
    );
    let mut malformed_length = frame;
    malformed_length[12] = 30;
    assert_eq!(
        decode_request::<Contract<Indirect>>(&malformed_length),
        Err(Error::Malformed)
    );
    let direct = encode_request::<Contract<Proof>>(&proof()).unwrap();
    const PROOF_ALLOCATION: usize =
        core::mem::size_of::<Proof>() + Proof::SHAPE.heap + Proof::SHAPE.bytes + 11;
    assert!(matches!(
        decode_request::<Contract<Box<Proof>, { PROOF_ALLOCATION - 1 }>>(direct.as_bytes()),
        Err(Error::SchemaLimit)
    ));
    assert_eq!(
        encode_request::<Contract<Box<Box<Proof>>>>(&Box::new(Box::new(proof())))
            .unwrap()
            .as_bytes(),
        direct.as_bytes()
    );
    assert_eq!(
        decode_request::<Contract<Box<Box<Proof>>>>(direct.as_bytes()).unwrap(),
        Box::new(Box::new(proof()))
    );
    assert_eq!(
        <Box<Proof>>::SHAPE.heap,
        core::mem::size_of::<Proof>() + Proof::SHAPE.heap
    );
    assert_eq!(
        <Box<Box<Proof>>>::SHAPE.heap,
        core::mem::size_of::<Box<Proof>>() + <Box<Proof>>::SHAPE.heap
    );
    assert_eq!(<Box<Proof>>::SHAPE.bytes, Proof::SHAPE.bytes);
    assert_eq!(<Box<Proof>>::SHAPE.nodes, Proof::SHAPE.nodes);
    assert_eq!(<Box<Proof>>::SHAPE.depth, Proof::SHAPE.depth);
    #[cfg(feature = "typescript")]
    assert_eq!(
        ic_wire::typescript::generate::<Contract<Inline>>().unwrap(),
        ic_wire::typescript::generate::<Contract<Indirect>>().unwrap()
    );
}

static DESERIALIZATIONS: AtomicUsize = AtomicUsize::new(0);
#[derive(serde::Serialize)]
struct Observed(u64);
impl<'de> serde::Deserialize<'de> for Observed {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        DESERIALIZATIONS.fetch_add(1, Ordering::SeqCst);
        <u64 as serde::Deserialize>::deserialize(deserializer).map(Self)
    }
}
impl Wire for Observed {
    const SHAPE: Shape = u64::SHAPE;
    const SCHEMA: ic_wire::schema::Schema = u64::SCHEMA;
}

#[test]
fn allocation_shortfall_refuses_before_deserialization_and_zst_boxes_are_free() {
    const NEEDED: usize = core::mem::size_of::<Observed>() + u64::SHAPE.bytes + 11;
    let value = Box::new(Observed(7));
    let frame = encode_request::<Contract<Box<Observed>, NEEDED>>(&value).unwrap();
    assert!(matches!(
        encode_request::<Contract<Box<Observed>, { NEEDED - 1 }>>(&value),
        Err(Error::SchemaLimit)
    ));
    DESERIALIZATIONS.store(0, Ordering::SeqCst);
    assert!(matches!(
        decode_request::<Contract<Box<Observed>, { NEEDED - 1 }>>(frame.as_bytes()),
        Err(Error::SchemaLimit)
    ));
    assert_eq!(DESERIALIZATIONS.load(Ordering::SeqCst), 0);
    assert_eq!(
        decode_request::<Contract<Box<Observed>, NEEDED>>(frame.as_bytes())
            .unwrap()
            .0,
        7
    );
    assert_eq!(DESERIALIZATIONS.load(Ordering::SeqCst), 1);
    assert_eq!(<Box<()>>::SHAPE.heap, 0);
    let unit = encode_request::<Contract<Box<()>, 11>>(&Box::new(())).unwrap();
    assert_eq!(
        decode_request::<Contract<Box<()>, 11>>(unit.as_bytes()).unwrap(),
        Box::new(())
    );
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Overflow(u64);
    impl Wire for Overflow {
        const SHAPE: Shape = Shape {
            heap: usize::MAX - 1,
            ..u64::SHAPE
        };
    }
    assert_eq!(<Box<Overflow>>::SHAPE.heap, usize::MAX);
    assert!(matches!(
        decode_request::<Contract<Box<Overflow>>>(&[]),
        Err(Error::SchemaLimit)
    ));
}
