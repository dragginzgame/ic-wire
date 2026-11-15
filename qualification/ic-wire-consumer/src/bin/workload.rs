//! Retained standalone codec workload for native/Wasm builds.
use ic_wire_consumer::{
    Receipt, decode_receipt, decode_upload, encode_receipt, encode_upload, sample_upload,
};
use std::hint::black_box;
fn main() {
    let request = black_box(sample_upload().unwrap());
    let bytes = encode_upload(&request).unwrap();
    assert_eq!(decode_upload(black_box(&bytes)).unwrap(), request);
    let reply = black_box(Receipt {
        revision: 129,
        accepted: true,
    });
    let bytes = encode_receipt(&reply).unwrap();
    assert_eq!(decode_receipt(black_box(&bytes)).unwrap(), reply);
}
