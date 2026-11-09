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

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wire-schema", derive(ic_wire::Wire))]
pub struct Digest {
    pub key: ic_wire::ByteArray<2>,
}

type NonzeroWords = Vec<core::num::NonZeroI64>;
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wire-schema", derive(ic_wire::Wire))]
pub struct Domain<T> {
    pub tag: T,
    #[serde(with = "::ic_wire::serde_packed::nonzero_i64")]
    pub ids: NonzeroWords,
    #[serde(with = "ic_wire::serde_packed::heap_u64")]
    pub queue: alloc::collections::BinaryHeap<u64>,
    #[serde(with = "::ic_wire::serde_packed::bools")]
    pub flags: Vec<bool>,
    #[serde(with = "::ic_wire::serde_packed::finite_f32")]
    pub samples: Vec<f32>,
    #[serde(with = "ic_wire::serde_packed::finite_f64")]
    pub measures: Vec<f64>,
    pub opaque_ids: ic_wire::Packed<[u8; 16]>,
}

pub fn roundtrip() -> Result<(), ic_wire::Error> {
    let limits = ic_wire::Limits {
        bytes: 256,
        allocation: 8192,
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
    let digest = Digest {
        key: ic_wire::ByteArray::new([0, 255]),
    };
    let bytes = ic_wire::native::encode_request(1, &digest, limits)?;
    assert_eq!(&bytes[11..], &[2, 0, 255]);
    assert_eq!(
        ic_wire::native::decode_request::<Digest>(1, &bytes, limits)?,
        digest
    );
    let id = TemplateId(alloc::borrow::Cow::Borrowed("id"));
    let bytes = ic_wire::native::encode_request(1, &id, limits)?;
    assert_eq!(&bytes[11..], &[2, b'i', b'd']);
    assert_eq!(
        ic_wire::native::decode_request::<TemplateId>(1, &bytes, limits)?,
        id
    );
    let domain = Domain {
        tag: 7_u8,
        ids: alloc::vec![core::num::NonZeroI64::new(-1).unwrap()],
        queue: alloc::collections::BinaryHeap::from(alloc::vec![1]),
        flags: alloc::vec![false, true],
        samples: alloc::vec![1.0],
        measures: alloc::vec![-0.0],
        opaque_ids: ic_wire::Packed::new(alloc::vec![[0x95; 16]]),
    };
    let bytes = ic_wire::native::encode(&domain, limits)?;
    assert_eq!(
        &bytes[..22],
        &[
            7, 8, 255, 255, 255, 255, 255, 255, 255, 255, 8, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 1
        ]
    );
    assert_eq!(
        &bytes[22..36],
        &[4, 0, 0, 128, 63, 8, 0, 0, 0, 0, 0, 0, 0, 128]
    );
    assert_eq!(
        &bytes[36..],
        &[alloc::vec![16], alloc::vec![0x95;16]].concat()
    );
    let decoded: Domain<u8> = ic_wire::native::decode(&bytes, limits)?;
    assert_eq!(decoded.ids, domain.ids);
    assert_eq!(decoded.flags, domain.flags);
    assert_eq!(decoded.samples, domain.samples);
    assert_eq!(decoded.measures[0].to_bits(), domain.measures[0].to_bits());
    assert_eq!(decoded.opaque_ids, domain.opaque_ids);
    assert_eq!(decoded.queue.into_sorted_vec(), [1]);
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
        let schema = ic_wire::typescript::native_schema::<Domain<u8>>().unwrap();
        assert_eq!(schema["fields"][1]["value"]["nonzero"], true);
        assert_eq!(schema["fields"][2]["value"]["word"], "u64");
        assert_eq!(schema["fields"][3]["value"]["word"], "bool");
        assert_eq!(schema["fields"][4]["value"]["word"], "finite-f32");
        assert_eq!(schema["fields"][5]["value"]["word"], "finite-f64");
        assert_eq!(schema["fields"][6]["value"]["word"], "hash16");
        let schema = ic_wire::typescript::native_schema::<Page<u64>>().unwrap();
        assert_eq!(schema["fields"][0]["name"], "entries");
        assert_eq!(schema["fields"][0]["value"]["k"], "list");
        assert_eq!(schema["fields"][1]["name"], "total");
        let schema = ic_wire::typescript::native_schema::<Digest>().unwrap();
        let key = &schema["fields"][0]["value"];
        assert_eq!(key["k"], "bytes");
        assert_eq!(key["min"], 2);
        assert_eq!(key["max"], 2);
        let schema = ic_wire::typescript::native_schema::<Upload>().unwrap();
        assert_eq!(schema["fields"][0]["name"], "data");
        assert_eq!(schema["fields"][0]["value"]["k"], "bytes");
    }
}
