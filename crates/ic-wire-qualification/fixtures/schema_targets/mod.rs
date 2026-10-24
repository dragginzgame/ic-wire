//! Consumer forwarding fixture: schema selection must preserve Wasm runtime use.
#![no_std]
extern crate alloc;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wire-schema", derive(ic_wire::Wire))]
pub struct Page<T> {
    pub entries: Vec<T>,
    pub total: u64,
}

pub fn roundtrip() -> Result<(), ic_wire::Error> {
    let limits = ic_wire::Limits {
        bytes: 256,
        allocation: 4096,
        nodes: 64,
        depth: 8,
    };
    let page = Page {
        entries: alloc::vec![1_u64, 128],
        total: 2,
    };
    let bytes = ic_wire::native::encode_request(1, &page, limits)?;
    // Feature forwarding cannot change header, declaration order or varints.
    assert_eq!(bytes, [73, 87, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 128, 1, 2]);
    assert_eq!(
        ic_wire::native::decode_request::<Page<u64>>(1, &bytes, limits)?,
        page
    );
    Ok(())
}

#[cfg(all(test, feature = "wire-schema", not(target_family = "wasm")))]
#[test]
fn metadata_and_runtime() {
    roundtrip().unwrap();
    let schema = ic_wire::typescript::native_schema::<Page<u64>>().unwrap();
    assert_eq!(schema["fields"][0]["name"], "entries");
    assert_eq!(schema["fields"][0]["value"]["k"], "list");
    assert_eq!(schema["fields"][1]["name"], "total");
}
