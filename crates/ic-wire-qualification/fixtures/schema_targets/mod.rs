//! Consumer forwarding fixture: schema selection must preserve Wasm runtime use.
#![no_std]
extern crate alloc;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wire-schema", derive(ic_wire::Wire))]
#[serde(transparent)]
pub struct TemplateId(pub alloc::borrow::Cow<'static, str>);

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wire-schema", derive(ic_wire::Wire))]
pub struct Page<T> {
    pub entries: Vec<T>,
    pub total: u64,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wire-schema", derive(ic_wire::Wire))]
pub struct Upload {
    pub data: ic_wire::ByteBuf,
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
    // The public re-export needs only ic-wire + serde, including no_std/Wasm.
    // Its payload remains the existing byte-buffer encoding: 03 00 7f ff.
    let upload = Upload {
        data: ic_wire::ByteBuf::from(alloc::vec![0, 127, 255]),
    };
    let bytes = ic_wire::native::encode_request(1, &upload, limits)?;
    assert_eq!(bytes, [73, 87, 1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 127, 255]);
    assert_eq!(
        ic_wire::native::decode_request::<Upload>(1, &bytes, limits)?,
        upload
    );
    let id = TemplateId(alloc::borrow::Cow::Borrowed("id"));
    let bytes = ic_wire::native::encode_request(1, &id, limits)?;
    assert_eq!(&bytes[11..], &[2, b'i', b'd']);
    assert_eq!(
        ic_wire::native::decode_request::<TemplateId>(1, &bytes, limits)?,
        id
    );
    Ok(())
}

#[cfg(all(test, not(target_family = "wasm")))]
#[test]
fn metadata_and_runtime() {
    roundtrip().unwrap();
    #[cfg(feature = "wire-schema")]
    {
        assert_eq!(
            ic_wire::typescript::native_schema::<TemplateId>().unwrap()["k"],
            "text"
        );
        let schema = ic_wire::typescript::native_schema::<Page<u64>>().unwrap();
        assert_eq!(schema["fields"][0]["name"], "entries");
        assert_eq!(schema["fields"][0]["value"]["k"], "list");
        assert_eq!(schema["fields"][1]["name"], "total");
        let schema = ic_wire::typescript::native_schema::<Upload>().unwrap();
        assert_eq!(schema["fields"][0]["name"], "data");
        assert_eq!(schema["fields"][0]["value"]["k"], "bytes");
    }
}
