#[test]
fn pre_copy_header_agrees_with_complete_frame_admission() {
    let id = 7;
    let frame = super::encode_request(
        id,
        &(),
        crate::Limits {
            bytes: 64,
            allocation: 1024,
            nodes: 8,
            depth: 8,
        },
    )
    .unwrap();
    let header: [u8; super::REQUEST_HEADER_BYTES] =
        frame[..super::REQUEST_HEADER_BYTES].try_into().unwrap();
    assert_eq!(super::validate_request_header(id, &header), Ok(()));
    for index in [0, 2, 10] {
        let mut damaged = header;
        damaged[index] ^= 1;
        assert_eq!(
            super::validate_request_header(id, &damaged),
            Err(crate::Error::Identity)
        );
    }
}
