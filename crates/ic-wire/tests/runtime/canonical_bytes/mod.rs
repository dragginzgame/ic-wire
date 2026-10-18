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
