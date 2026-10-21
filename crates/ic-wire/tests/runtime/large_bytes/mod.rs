//! Large opaque buffers retain canonical bytes and bounded failure behavior.
use ic_wire::{Bytes, Error, Limits, Method, Wire, decode_request, encode_request, native};

const MAX: usize = 1024 * 1024;
struct Large;
impl Method for Large {
    type Request = Bytes<MAX>;
    type Response = Bytes<MAX>;
    const ID: u64 = 0x6c61_7267_6500_0001;
    const NAME: &'static str = "large_bytes";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: MAX + 21,
        allocation: 2 * MAX + 21,
        nodes: 1,
        depth: 1,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}

#[test]
fn mebibyte_buffers_match_finite_native_and_raw_payloads() {
    for len in [0, 127, 128, MAX - 1, MAX] {
        let value = Bytes::<MAX>::new((0..len).map(|i| (i % 251) as u8).collect()).unwrap();
        let frame = encode_request::<Large>(&value).unwrap().into_bytes();
        assert_eq!(
            ic_wire::encoded_request_len::<Large>(&value).unwrap(),
            frame.len()
        );
        assert_eq!(decode_request::<Large>(&frame).unwrap(), value);
        let limits = Limits {
            allocation: 32 * MAX,
            nodes: 8,
            depth: 8,
            ..Large::REQUEST_LIMITS
        };
        assert_eq!(
            native::encode_request(Large::ID, &value, limits).unwrap(),
            frame
        );
        assert_eq!(
            native::decode_request::<Bytes<MAX>>(Large::ID, &frame, limits).unwrap(),
            value
        );
        assert_eq!(&frame[11..], postcard::to_allocvec(&value).unwrap());
    }
}

#[test]
fn large_buffers_refuse_bad_lengths_truncation_identity_and_noncanonical_prefixes() {
    let value = Bytes::<MAX>::new(vec![255; MAX]).unwrap();
    let frame = encode_request::<Large>(&value).unwrap().into_bytes();
    for end in [0, 10, 11, 12, frame.len() / 2, frame.len() - 1] {
        assert!(decode_request::<Large>(&frame[..end]).is_err());
    }
    let mut wrong = frame.clone();
    wrong[2] ^= 1;
    assert_eq!(decode_request::<Large>(&wrong), Err(Error::Identity));
    assert_eq!(
        ic_wire::decode_response::<Large>(&frame),
        Err(Error::Identity)
    );
    let mut trailing = frame.clone();
    trailing.push(0);
    assert_eq!(decode_request::<Large>(&trailing), Err(Error::Trailing));
    let oversized = Bytes::<{ MAX + 1 }>::new(vec![0; MAX + 1]).unwrap();
    let mut bytes = frame[..11].to_vec();
    bytes.extend(postcard::to_allocvec(&oversized).unwrap());
    assert_eq!(decode_request::<Large>(&bytes), Err(Error::Malformed));
    let mut nonminimal = frame[..11].to_vec();
    nonminimal.extend([128, 0]);
    assert_eq!(
        decode_request::<Large>(&nonminimal),
        Err(Error::Noncanonical)
    );
    assert!(Bytes::<MAX>::new(vec![0; MAX + 1]).is_err());
    assert!(matches!(
        native::decode_request::<Bytes<MAX>>(
            Large::ID,
            &frame,
            Limits {
                allocation: MAX,
                ..Large::REQUEST_LIMITS
            }
        ),
        Err(Error::Limit)
    ));
    assert_eq!(Bytes::<MAX>::SHAPE.nodes, 1);
}

#[test]
fn scratch_reuse_preserves_frames_capacity_and_frozen_retries() {
    let mut scratch = Vec::new();
    let mut frozen = Vec::new();
    let mut high_water = 0;
    for len in [0, 32, 4096, MAX, 32, 0] {
        let value = Bytes::<MAX>::new(vec![42; len]).unwrap();
        ic_wire::encode_request_into::<Large>(&value, &mut scratch).unwrap();
        assert_eq!(scratch, encode_request::<Large>(&value).unwrap().as_bytes());
        assert!(scratch.capacity() >= high_water);
        high_water = scratch.capacity();
        if len == 4096 {
            frozen = scratch.clone();
        }
        ic_wire::encode_response_into::<Large>(&value, &mut scratch).unwrap();
        assert_eq!(
            scratch,
            ic_wire::encode_response::<Large>(&value)
                .unwrap()
                .as_bytes()
        );
        assert_eq!(ic_wire::decode_response::<Large>(&scratch).unwrap(), value);
    }
    assert_eq!(
        decode_request::<Large>(&frozen).unwrap().as_slice(),
        &[42; 4096]
    );
}
