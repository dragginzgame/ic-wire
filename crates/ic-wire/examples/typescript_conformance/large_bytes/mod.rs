//! Large-buffer vectors use selected boundary mutations instead of quadratic scans.
use ic_wire::{Bytes, Limits, Method};
use std::{fs, path::Path};
struct Large;
impl Method for Large {
    type Request = Bytes<{ 1024 * 1024 }>;
    type Response = Self::Request;
    const ID: u64 = 0x6c61_7267_6500_0001;
    const NAME: &'static str = "large_bytes";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 1024 * 1024 + 21,
        allocation: 2 * 1024 * 1024 + 21,
        nodes: 1,
        depth: 1,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
pub(super) fn generate(directory: &Path) -> String {
    let value = Bytes::new((0..1024 * 1024).map(|i| (i % 251) as u8).collect()).unwrap();
    fs::write(
        directory.join("large.ts"),
        ic_wire::typescript::generate::<Large>().unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("large-request.bin"),
        ic_wire::encode_request::<Large>(&value).unwrap().as_bytes(),
    )
    .unwrap();
    fs::write(
        directory.join("large-response.bin"),
        ic_wire::encode_response::<Large>(&value)
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    include_str!("checks.mjs").to_owned()
}
