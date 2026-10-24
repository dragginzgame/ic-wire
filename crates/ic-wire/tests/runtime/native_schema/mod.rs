use ic_wire::{Wire, typescript::native_schema};
use serde::Serialize;
use serde_json::json;

trait HasItem {
    type Item: Serialize;
}
#[derive(Serialize)]
struct Provider;
impl HasItem for Provider {
    type Item = u16;
}
#[derive(Serialize, Wire)]
struct Associated<T: HasItem> {
    value: T::Item,
}
#[derive(Serialize, Wire)]
enum Recursive<T> {
    Value(T),
    Next(Box<Self>),
}

#[test]
fn metadata_bounds_follow_fields_and_preserve_recursive_refusal() {
    assert_eq!(
        native_schema::<Associated<Provider>>().unwrap(),
        json!({"k":"record","fields":[{"name":"value","value":{"k":"u","bits":16}}]})
    );
    let value = Associated::<Provider> { value: 128 };
    assert_eq!(postcard::to_allocvec(&value).unwrap(), [128, 1]);
    let _ = Recursive::Next(Box::new(Recursive::Value(1_u8)));
    assert_eq!(
        native_schema::<Recursive<u8>>(),
        Err(ic_wire::typescript::GenerationError::Complexity)
    );
}

#[derive(Wire, Serialize)]
struct Borrowed<'a> {
    r#type: &'a str,
    value: &'a u16,
}
#[derive(Wire, Serialize)]
struct Array<T, const N: usize>
where
    [T; N]: Serialize,
{
    values: [T; N],
}
#[derive(Wire, Serialize)]
struct Unit;
#[derive(Wire, Serialize)]
struct Empty();
#[derive(Wire, Serialize)]
struct Pair(u8, u16);
#[derive(Wire, Serialize)]
struct First {
    a: u8,
    b: u16,
}
#[derive(Wire, Serialize)]
struct Reordered {
    b: u16,
    a: u8,
}

#[test]
fn byte_buffers_keep_frames_and_admit_in_bulk() {
    use ic_wire::{Error, Limits, native};
    let limits = Limits {
        bytes: 8192,
        allocation: 1048576,
        nodes: 65536,
        depth: 64,
    };
    assert_eq!(
        native_schema::<ic_wire::ByteBuf>().unwrap(),
        json!({"k":"bytes"})
    );
    assert_eq!(
        native_schema::<&serde_bytes::Bytes>().unwrap(),
        json!({"k":"bytes"})
    );
    for length in [0, 1, 127, 128, 4096] {
        let vector = vec![255; length];
        let bytes: ic_wire::ByteBuf = serde_bytes::ByteBuf::from(vector.clone());
        let frame = native::encode_request(17, &vector, limits).unwrap();
        assert_eq!(frame, native::encode_request(17, &bytes, limits).unwrap());
        let bulk = Limits {
            nodes: 16,
            ..limits
        };
        assert_eq!(frame, native::encode_request(17, &bytes, bulk).unwrap());
        assert_eq!(
            native::decode_request::<ic_wire::ByteBuf>(17, &frame, bulk).unwrap(),
            bytes
        );
        let borrowed: &serde_bytes::Bytes = native::decode_request(17, &frame, bulk).unwrap();
        assert_eq!(borrowed.as_ref(), vector.as_slice());
        if length >= 127 {
            assert_eq!(native::encode_request(17, &vector, bulk), Err(Error::Limit));
        }
        assert!(
            native::decode_request::<ic_wire::ByteBuf>(17, &frame[..frame.len() - 1], bulk)
                .is_err()
        );
        let mut trailing = frame;
        trailing.push(0);
        assert!(native::decode_request::<ic_wire::ByteBuf>(17, &trailing, bulk).is_err());
    }
}

#[test]
fn generics_borrows_and_struct_forms_follow_serde() {
    let list = native_schema::<Vec<u16>>().unwrap();
    assert_eq!(native_schema::<[u16]>().unwrap(), list);
    assert_eq!(native_schema::<&[u16]>().unwrap(), list);
    assert_eq!(native_schema::<Box<[u16]>>().unwrap(), list);
    // A byte slice still uses Serde's sequence layout, not byte-buffer metadata.
    assert_eq!(
        native_schema::<&[u8]>().unwrap(),
        native_schema::<Vec<u8>>().unwrap()
    );
    assert_eq!(
        native_schema::<Borrowed<'_>>().unwrap(),
        json!({"k":"record","fields":[
            {"name":"type","value":{"k":"text"}}, {"name":"value","value":{"k":"u","bits":16}}
        ]})
    );
    assert_eq!(
        native_schema::<Array<u16, 2>>().unwrap(),
        json!({"k":"record","fields":[
            {"name":"values","value":{"k":"array","max":2,"item":{"k":"u","bits":16}}}
        ]})
    );
    assert_eq!(native_schema::<Unit>().unwrap(), json!({"k":"unit"}));
    assert_eq!(
        native_schema::<Empty>().unwrap(),
        json!({"k":"tuple","items":[]})
    );
    assert_eq!(
        native_schema::<Pair>().unwrap(),
        json!({"k":"tuple","items":[{"k":"u","bits":8},{"k":"u","bits":16}]})
    );
    assert_eq!(
        postcard::to_allocvec(&First { a: 1, b: 128 }).unwrap(),
        [1, 128, 1]
    );
    assert_eq!(
        postcard::to_allocvec(&Reordered { b: 128, a: 1 }).unwrap(),
        [128, 1, 1]
    );
    assert_eq!(native_schema::<First>().unwrap()["fields"][0]["name"], "a");
    assert_eq!(
        native_schema::<Reordered>().unwrap()["fields"][0]["name"],
        "b"
    );
}
