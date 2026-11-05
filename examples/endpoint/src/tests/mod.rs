use super::*;

#[test]
fn borrowed_fields_and_reply_retain_input_storage() {
    let value = Request {
        value: 5,
        bytes: ic_wire::Bytes::new(vec![1, 2]).unwrap(),
    };
    let frame = ic_wire::native::encode_request(17, &value, LIMITS).unwrap();
    let view: RequestRef<'_> = ic_wire_endpoint::decode_request(&frame, 17, LIMITS);
    assert_eq!(
        view.bytes.as_slice().as_ptr(),
        frame[frame.len() - 2..].as_ptr()
    );
    let reply = borrowed_echo(view);
    assert_eq!(
        ic_wire_endpoint::encode_response(&reply, 17, LIMITS),
        ic_wire::native::encode_response(17, &value, LIMITS).unwrap()
    );
    #[cfg(feature = "schema")]
    {
        assert_eq!(
            BORROWED.request_schema().unwrap(),
            ECHO.request_schema().unwrap()
        );
        assert_eq!(
            BORROWED.response_schema().unwrap(),
            ECHO.response_schema().unwrap()
        );
    }
}

#[test]
fn defaults_and_overrides_bind_the_actual_contract() {
    use ic_wire_endpoint::{DEFAULT_REQUEST_LIMITS, DEFAULT_RESPONSE_LIMITS};
    let fields = |value: Limits| (value.bytes, value.allocation, value.nodes, value.depth);
    assert_eq!(
        fields(DEFAULT_REQUEST_LIMITS),
        (65_536, 8_388_608, 65_536, 32)
    );
    assert_eq!(
        fields(DEFAULT_RESPONSE_LIMITS),
        fields(DEFAULT_REQUEST_LIMITS)
    );
    assert_eq!(fields(DEFAULT_BLOB.request), fields(DEFAULT_REQUEST_LIMITS));
    assert_eq!(
        fields(DEFAULT_BLOB.response),
        fields(DEFAULT_RESPONSE_LIMITS)
    );
    assert_eq!(DEFAULT_BLOB.id, 3_695_642_065_081_963_175);
    assert_eq!(NAMED.id, 11_911_248_233_000_959_399);
    assert_eq!(NAMED.name, "stable-name");
    assert_eq!(fields(NAMED.request), fields(LIMITS));
    assert_eq!(fields(NAMED.response), fields(DEFAULT_RESPONSE_LIMITS));
    assert_eq!(
        fields(RESPONSE_ONLY.request),
        fields(DEFAULT_REQUEST_LIMITS)
    );
    assert_eq!(fields(RESPONSE_ONLY.response), fields(LIMITS));
    assert_eq!(ECHO.id, 7, "explicit identity remains authoritative");
    #[cfg(feature = "schema")]
    {
        assert_eq!(
            DEFAULT_BLOB.request_schema().unwrap(),
            ic_wire::typescript::native_schema::<ByteBuf>().unwrap()
        );
        assert_eq!(
            DEFAULT_BLOB.response_schema().unwrap(),
            DEFAULT_BLOB.request_schema().unwrap()
        );
    }
}

#[test]
fn declaration_matches_types_constants_and_existing_projection() {
    // Actual aliases are accepted by the actual functions, including unit/tuple.
    let request: Request = Request {
        value: 5,
        bytes: ic_wire::Bytes::new(vec![1, 2]).unwrap(),
    };
    let result: Request = echo(request.clone());
    assert_eq!(request, result);
    let _: () = unit(());
    assert_eq!(ECHO.id, 7);
    assert_eq!(ECHO.name, "echo");
    assert_eq!(ECHO.mode, "query");
    assert_eq!(COMPOSITE.mode, "composite_query");
    assert_eq!(MUTATE.mode, "update");
    assert_eq!(UNIT.name, "unit-call");
    let fields = |value: Limits| (value.bytes, value.allocation, value.nodes, value.depth);
    assert_eq!(fields(ECHO.request), fields(LIMITS));
    assert_eq!(fields(ECHO.response), fields(LIMITS));
    assert_eq!(NODES.request.nodes, 3);
    #[cfg(feature = "schema")]
    {
        assert_eq!(
            ECHO.request_schema().unwrap(),
            ic_wire::typescript::native_schema::<Request>().unwrap()
        );
        assert_eq!(
            ECHO.response_schema().unwrap(),
            ECHO.request_schema().unwrap()
        );
    }
    #[cfg(feature = "schema")]
    {
        assert_eq!(
            COUNT_CONTRACT.request_schema().unwrap(),
            ic_wire::typescript::native_schema::<()>().unwrap()
        );
        assert_eq!(
            ASYNC_QUERY.response_schema().unwrap(),
            ic_wire::typescript::native_schema::<(u64, bool)>().unwrap()
        );
    }
    // Concrete pre-existing frame oracle: IW, little-endian ID, direction, value.
    assert_eq!(
        ic_wire::native::encode_request(7, &42u8, LIMITS).unwrap(),
        [73, 87, 7, 0, 0, 0, 0, 0, 0, 0, 0, 42]
    );
}
