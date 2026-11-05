fn escape() -> ic_wire::TextRef<'static, 32> {
    let frame = ic_wire::native::encode_request(
        8,
        &ic_wire::TextRef::<32>::new("local").unwrap(),
        crate::support::LIMITS,
    )
    .unwrap();
    ic_wire::native::decode_request::<ic_wire::TextRef<'_, 32>>(8, &frame, crate::support::LIMITS)
        .unwrap()
}
