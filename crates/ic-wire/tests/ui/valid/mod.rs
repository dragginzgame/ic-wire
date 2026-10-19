use crate::support::*;
fn check() {
    let _ = adapter(&[], |_| Ok(7));
    let _future = async_adapter(&[], |_| async { Ok(7) });
}

#[derive(ic_wire::Wire)]
struct Borrowed<'de> {
    #[wire(id = 1)]
    text: ic_wire::TextRef<'de, 32>,
}
#[derive(ic_wire::Wire)]
enum BorrowedChoice<'de> {
    #[wire(id = 1)]
    Data(Borrowed<'de>),
    #[wire(id = 2)]
    Empty,
}
fn borrowed_check() {
    let frame =
        ic_wire::encode_request::<BorrowedRead<'_>>(&ic_wire::TextRef::new("local").unwrap())
            .unwrap();
    let view = ic_wire::decode_request::<BorrowedRead<'_>>(frame.as_bytes()).unwrap();
    assert_eq!(view.as_str(), "local");
}
