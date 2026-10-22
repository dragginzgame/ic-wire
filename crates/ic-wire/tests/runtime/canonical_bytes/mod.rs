//! Every differing byte must be rejected, including bulk boundaries and tails.
use ic_wire::{Bytes, Error, Limits, Method, Shape, Wire, decode_request, encode_request};
use serde::{Deserialize, Deserializer, Serialize};

// A trusted custom contract that normalizes bytes in place on decode. Thus
// arbitrary nonzero input is valid Serde but fails Wire's canonical comparison.
#[derive(Serialize)]
#[serde(transparent)]
struct Normalized(Bytes<65>);
impl<'de> Deserialize<'de> for Normalized {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut bytes = Bytes::<65>::deserialize(d)?.into_vec();
        bytes.fill(0);
        Ok(Self(Bytes::new(bytes).unwrap()))
    }
}
impl Wire for Normalized {
    const SHAPE: Shape = Bytes::<65>::SHAPE;
}
struct Blob;
impl Method for Blob {
    type Request = Normalized;
    type Response = ();
    const ID: u64 = 0x43414e4f4e;
    const NAME: &'static str = "canonical_blob";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 128,
        allocation: 256,
        nodes: 8,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}

#[test]
fn bulk_comparison_checks_every_byte_and_tail() {
    for len in 0..=65 {
        let value = Normalized(Bytes::new(vec![0; len]).unwrap());
        let canonical = encode_request::<Blob>(&value).unwrap().into_bytes();
        let decoded = decode_request::<Blob>(&canonical).unwrap();
        assert_eq!(
            encode_request::<Blob>(&decoded).unwrap().as_bytes(),
            canonical
        );
        for position in 0..len {
            let mut altered = canonical.clone();
            // Header followed by one-byte length for every value in this test.
            altered[12 + position] = 1;
            assert!(
                matches!(decode_request::<Blob>(&altered), Err(Error::Noncanonical)),
                "length {len}, position {position}"
            );
            assert_eq!(
                ic_wire::native::decode_request::<Normalized>(
                    Blob::ID,
                    &altered,
                    Limits {
                        allocation: 8192,
                        nodes: 64,
                        ..Blob::REQUEST_LIMITS
                    }
                )
                .err(),
                Some(Error::Noncanonical)
            );
        }
    }
}

// Custom fields can shift verification boundaries without a local mismatch.
struct Prefix(u8);
impl Serialize for Prefix {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}
impl<'de> Deserialize<'de> for Prefix {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let [first, _] = <[u8; 2]>::deserialize(d)?;
        Ok(Self(first))
    }
}
impl Wire for Prefix {
    const SHAPE: Shape = Shape::scalar(2);
}
struct Suffix(u8);
impl Serialize for Suffix {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        [42, self.0].serialize(s)
    }
}
impl<'de> Deserialize<'de> for Suffix {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self(u8::deserialize(d)?))
    }
}
impl Wire for Suffix {
    const SHAPE: Shape = Shape::scalar(2);
}
#[derive(Wire)]
struct Shifted {
    #[wire(id = 1)]
    before: Prefix,
    #[wire(id = 2)]
    bytes: Bytes<8>,
    #[wire(id = 3)]
    after: Suffix,
}
struct ShiftedMethod;
impl Method for ShiftedMethod {
    type Request = Shifted;
    type Response = ();
    const ID: u64 = Blob::ID;
    const NAME: &'static str = "shifted";
    const REQUEST_LIMITS: Limits = Blob::REQUEST_LIMITS;
    const RESPONSE_LIMITS: Limits = Blob::RESPONSE_LIMITS;
}
#[test]
fn custom_field_boundaries_cannot_hide_a_different_roundtrip() {
    let mut frame = encode_request::<Blob>(&Normalized(Bytes::new(vec![]).unwrap()))
        .unwrap()
        .into_bytes();
    frame.truncate(11);
    frame.extend_from_slice(&[7, 1, 1, 42, 9]);
    assert!(matches!(
        decode_request::<ShiftedMethod>(&frame),
        Err(Error::Noncanonical)
    ));
}

#[derive(Wire)]
struct CopiedRecord {
    #[wire(id = 1)]
    prefix: u16,
    #[wire(id = 2)]
    first: Bytes<4096>,
    #[wire(id = 3)]
    second: Bytes<8>,
    #[wire(id = 4)]
    suffix: u32,
}
struct CopiedMethod;
impl Method for CopiedMethod {
    type Request = CopiedRecord;
    type Response = ();
    const ID: u64 = 9;
    const NAME: &'static str = "copied";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 8192,
        allocation: 16384,
        nodes: 32,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
#[test]
fn copied_record_checks_prefixes_truncation_and_preserves_owned_bytes() {
    for len in [0, 1, 32, 63, 64, 65, 127, 128, 4096] {
        let value = CopiedRecord {
            prefix: 128,
            first: Bytes::new(vec![42; len]).unwrap(),
            second: Bytes::new(vec![7; 3]).unwrap(),
            suffix: 16384,
        };
        let mut frame = encode_request::<CopiedMethod>(&value).unwrap().into_bytes();
        let decoded = decode_request::<CopiedMethod>(&frame).unwrap();
        assert_eq!(
            encode_request::<CopiedMethod>(&decoded).unwrap().as_bytes(),
            frame
        );
        for end in 0..frame.len() {
            assert!(decode_request::<CopiedMethod>(&frame[..end]).is_err());
        }
        // Nonminimal scalar and length prefixes must still be rejected.
        for start in [11, 13] {
            let mut longer = frame.clone();
            let end = start + longer[start..].iter().position(|b| b & 128 == 0).unwrap();
            longer[end] |= 128;
            longer.insert(end + 1, 0);
            assert!(matches!(
                decode_request::<CopiedMethod>(&longer),
                Err(Error::Noncanonical)
            ));
        }
        let mut changed = frame.clone();
        if len != 0 {
            let start = 13 + if len < 128 { 1 } else { 2 };
            changed[start] ^= 1;
            let decoded = decode_request::<CopiedMethod>(&changed).unwrap();
            assert_eq!(
                encode_request::<CopiedMethod>(&decoded).unwrap().as_bytes(),
                changed
            );
        }
        frame.fill(0);
        assert_eq!(decoded.first.as_slice(), value.first.as_slice());
    }
}
