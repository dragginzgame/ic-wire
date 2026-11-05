//! Consumer-side ownership conversion after one complete borrowed Serde decode.
//! This is not a production decoder mode; conversion allocations belong to callers.
use ic_wire::{Bytes, BytesRef, Error, Limits, List, native};
use serde::{Deserialize, Serialize};
use std::cell::Cell;

const ID: u64 = 77;
const LIMITS: Limits = Limits {
    bytes: 128 * 1024,
    allocation: 2 * 1024 * 1024,
    nodes: 4096,
    depth: 32,
};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct Owned {
    chunks: List<Bytes<65536>, 128>,
}
#[derive(Serialize, Deserialize)]
struct View<'a> {
    #[serde(borrow)]
    chunks: List<BytesRef<'a, 65536>, 128>,
}
fn decode_then_own(frame: &[u8], before_copy: impl FnOnce()) -> Result<Owned, Error> {
    let view: View<'_> = native::decode_request(ID, frame, LIMITS)?;
    before_copy(); // No payload copies until full-input/identity/primitive checks pass.
    let chunks = view
        .chunks
        .into_vec()
        .into_iter()
        .map(|chunk| Bytes::new(chunk.as_slice().to_vec()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Owned {
        chunks: List::new(chunks)?,
    })
}

#[test]
fn borrowed_batch_preserves_bytes_and_defers_owned_payload_copies() {
    let value = Owned {
        chunks: List::new(vec![
            Bytes::new(vec![]).unwrap(),
            Bytes::new(vec![0x95; 32768]).unwrap(),
            Bytes::new(vec![]).unwrap(),
        ])
        .unwrap(),
    };
    let frame = native::encode_request(ID, &value, LIMITS).unwrap();
    let view: View<'_> = native::decode_request(ID, &frame, LIMITS).unwrap();
    assert_eq!(native::encode_request(ID, &view, LIMITS).unwrap(), frame);
    let range = frame.as_ptr().addr()..=frame.as_ptr().addr() + frame.len();
    for chunk in view.chunks.as_slice() {
        assert!(range.contains(&chunk.as_slice().as_ptr().addr()));
    }
    let copies = Cell::new(0);
    assert_eq!(decode_then_own(&frame, || copies.set(1)).unwrap(), value);
    assert_eq!(copies.get(), 1);
    for length in [
        0,
        native::REQUEST_HEADER_BYTES,
        frame.len() - 2,
        frame.len() - 1,
    ] {
        copies.set(0);
        assert!(decode_then_own(&frame[..length], || copies.set(1)).is_err());
        assert_eq!(copies.get(), 0);
    }
    let mut trailing = frame.clone();
    trailing.push(0);
    copies.set(0);
    assert_eq!(
        decode_then_own(&trailing, || copies.set(1)),
        Err(Error::Trailing)
    );
    assert_eq!(copies.get(), 0);
}

#[test]
fn field_bounds_refuse_before_ownership_conversion() {
    #[derive(Serialize)]
    struct Unbounded {
        chunks: Vec<ic_wire::ByteBuf>,
    }
    for chunks in [
        vec![ic_wire::ByteBuf::new(); 129],
        vec![ic_wire::ByteBuf::from(vec![0; 65537])],
    ] {
        let frame = native::encode_request(ID, &Unbounded { chunks }, LIMITS).unwrap();
        let copied = Cell::new(false);
        assert!(decode_then_own(&frame, || copied.set(true)).is_err());
        assert!(!copied.get());
    }
}
