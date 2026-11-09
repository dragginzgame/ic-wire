//! Compiled against explicitly selected published/current package combinations.
#[cfg(test)]
mod contract_tests;

#[derive(serde::Serialize, serde::Deserialize, ic_wire::Wire)]
pub struct Reply<T> {
    owner: ic_platform_types::Principal,
    value: T,
    bytes: ic_wire::ByteBuf,
}

#[ic_wire_endpoint::query(contract = READ)]
pub fn read() -> u64 {
    7
}
#[ic_wire_endpoint::query(borrow)]
pub fn echo(value: &str) -> &str {
    value
}

#[test]
fn schema_and_endpoint_share_the_selected_core() {
    let mut registry = ic_wire::typescript::SchemaRegistry::new();
    registry
        .register::<Reply<ic_wire::Packed<u64>>>("Reply")
        .unwrap();
    let types = registry.finish().unwrap();
    ic_wire::typescript::generate_registry(
        &types,
        &[],
        "./wire-runtime.js",
        &Default::default(),
        |r| assert!(r.is_clean()),
    )
    .unwrap();
    let frame = ic_wire::native::encode_request(READ.id, &(), READ.request).unwrap();
    let _: () = ic_wire_endpoint::decode_request(&frame, READ.id, READ.request);
    assert_eq!(read(), 7);
    assert_eq!(echo("borrowed"), "borrowed");
}
