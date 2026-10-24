//! Consumer normalization and strict primitive parsing are separate contracts.
use ic_wire::{Bytes, Error, Limits, native};
use serde::{Deserialize, Deserializer, Serialize};
const ID: u64 = 0x43414e4f4e;
const LIMITS: Limits = Limits {
    bytes: 8192,
    allocation: 131072,
    nodes: 512,
    depth: 32,
};
fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    native::encode_request(ID, value, LIMITS)
}
fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Error> {
    native::decode_request(ID, bytes, LIMITS)
}

// A trusted custom contract that normalizes bytes in place on decode. Thus
// arbitrary nonzero input now normalizes successfully without reserialization.
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

#[test]
fn custom_normalization_is_consumer_owned() {
    for len in 0..=65 {
        let value = Normalized(Bytes::new(vec![0; len]).unwrap());
        let canonical = encode::<Normalized>(&value).unwrap();
        let decoded = decode::<Normalized>(&canonical).unwrap();
        assert_eq!(
            encode::<Normalized>(&decoded).unwrap().as_slice(),
            canonical
        );
        for position in 0..len {
            let mut altered = canonical.clone();
            // Header followed by one-byte length for every value in this test.
            altered[12 + position] = 1;
            assert!(
                decode::<Normalized>(&altered)
                    .unwrap()
                    .0
                    .as_slice()
                    .iter()
                    .all(|b| *b == 0),
                "length {len}, position {position}"
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

#[derive(Serialize, Deserialize)]
struct Shifted {
    before: Prefix,
    bytes: Bytes<8>,
    after: Suffix,
}

#[test]
fn custom_field_boundaries_can_have_a_different_roundtrip() {
    let mut frame = encode::<Normalized>(&Normalized(Bytes::new(vec![]).unwrap())).unwrap();
    frame.truncate(11);
    frame.extend_from_slice(&[7, 1, 1, 42, 9]);
    let value = decode::<Shifted>(&frame).unwrap();
    assert_ne!(encode(&value).unwrap(), frame);
}

#[derive(Serialize, Deserialize)]
struct CopiedRecord {
    prefix: u16,
    first: Bytes<4096>,
    second: Bytes<8>,
    suffix: u32,
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
        let mut frame = encode::<CopiedRecord>(&value).unwrap();
        let decoded = decode::<CopiedRecord>(&frame).unwrap();
        assert_eq!(encode::<CopiedRecord>(&decoded).unwrap().as_slice(), frame);
        for end in 0..frame.len() {
            assert!(decode::<CopiedRecord>(&frame[..end]).is_err());
        }
        // Nonminimal scalar and length prefixes must still be rejected.
        for start in [11, 13] {
            let mut longer = frame.clone();
            let end = start + longer[start..].iter().position(|b| b & 128 == 0).unwrap();
            longer[end] |= 128;
            longer.insert(end + 1, 0);
            assert!(matches!(
                decode::<CopiedRecord>(&longer),
                Err(Error::Malformed)
            ));
        }
        let mut changed = frame.clone();
        if len != 0 {
            let start = 13 + if len < 128 { 1 } else { 2 };
            changed[start] ^= 1;
            let decoded = decode::<CopiedRecord>(&changed).unwrap();
            assert_eq!(
                encode::<CopiedRecord>(&decoded).unwrap().as_slice(),
                changed
            );
        }
        frame.fill(0);
        assert_eq!(decoded.first.as_slice(), value.first.as_slice());
    }
}
