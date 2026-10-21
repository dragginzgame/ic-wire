//! Consumer-shaped packed hashes can borrow existing array storage in both directions.
#![cfg(feature = "allocation-fixture")]
use ic_wire::{Limits, native};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Owned {
    chunks: serde_bytes::ByteBuf,
}
#[derive(Deserialize, Serialize)]
struct View<'a> {
    #[serde(borrow)]
    chunks: &'a serde_bytes::Bytes,
}
#[test]
fn packed_hash_projection_borrows_arrays_without_a_staging_copy() {
    let hashes = vec![[1_u8; 32], [2; 32], [255; 32]];
    let view = View {
        chunks: serde_bytes::Bytes::new(hashes.as_flattened()),
    };
    assert_eq!(view.chunks.as_ptr(), hashes.as_ptr().cast::<u8>());
    let owned = Owned {
        chunks: serde_bytes::ByteBuf::from(hashes.as_flattened().to_vec()),
    };
    let limits = Limits {
        bytes: 2048,
        allocation: 65536,
        nodes: 4096,
        depth: 32,
    };
    let frame = native::encode_request(7, &view, limits).unwrap();
    assert_eq!(frame, native::encode_request(7, &owned, limits).unwrap());
    let decoded: View<'_> = native::decode_request(7, &frame, limits).unwrap();
    let (chunks, remainder) = decoded.chunks.as_chunks::<32>();
    assert!(remainder.is_empty());
    assert_eq!(chunks, hashes);
    assert!(decoded.chunks.len() <= 1024 * 32);
    let start = frame.as_ptr() as usize;
    assert!((start..start + frame.len()).contains(&(decoded.chunks.as_ptr() as usize)));
}
