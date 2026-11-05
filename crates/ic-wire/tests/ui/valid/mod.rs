use crate::support::*;
fn check() {
    let _ = adapter(&[], |_| Ok(7));
    let _future = async_adapter(&[], |_| async { Ok(7) });
}
fn borrowed_check() {
    let frame =
        ic_wire::native::encode_request(8, &ic_wire::TextRef::<32>::new("local").unwrap(), LIMITS)
            .unwrap();
    let view: ic_wire::TextRef<'_, 32> =
        ic_wire::native::decode_request(8, &frame, LIMITS).unwrap();
    assert_eq!(view.as_str(), "local");
}
