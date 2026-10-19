fn escape() -> ic_wire::TextRef<'static, 32> {
    let frame = ic_wire::encode_request::<crate::support::BorrowedRead<'_>>(
        &ic_wire::TextRef::new("local").unwrap(),
    )
    .unwrap()
    .into_bytes();
    ic_wire::decode_request::<crate::support::BorrowedRead<'_>>(&frame).unwrap()
}
