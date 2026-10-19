fn mutate() {
    let mut frame = ic_wire::encode_request::<crate::support::BorrowedRead<'_>>(
        &ic_wire::TextRef::new("local").unwrap(),
    )
    .unwrap()
    .into_bytes();
    let view = ic_wire::decode_request::<crate::support::BorrowedRead<'_>>(&frame).unwrap();
    frame.clear();
    let _ = view.as_str();
}
