#![expect(dead_code, reason = "The selected driver uses only part of the shared fixture surface.")]
use ic_wire::Limits;
use ic_wire_endpoint::query;
const LIMITS: Limits = Limits {
    bytes: 128,
    allocation: 8192,
    nodes: 32,
    depth: 8,
};

#[cfg(ui_case = "valid")]
mod valid {
    use super::*;
    use ic_wire_endpoint as renamed;
    const ID: u64 = 17;
    type Request = u64;
    type Response = (u64, bool);
    #[renamed::query(crate = "renamed", contract = CONTRACT, id = ID, request = LIMITS, response = LIMITS)]
    fn r#match(request: self::Request) -> self::Response {
        (request, true)
    }
    #[query]
    async fn empty() {}
    #[query(contract = AUTOMATIC, name = "stable-name")]
    fn automatic(_: ()) -> u8 {
        1
    }
    #[query(request = LIMITS)]
    fn request_override() {}
    #[query(response = LIMITS)]
    fn response_override() {}
    #[query(id = 19, request = LIMITS, response = LIMITS)]
    #[cfg(any())]
    fn disabled(_: MissingType) -> MissingType {
        unreachable!()
    }
    const _: u64 = CONTRACT.id;
    #[renamed::query(borrow, crate = "renamed", contract = BORROWED)]
    fn borrowed(value: &str) -> &str {
        value
    }
    #[query(borrow)]
    fn bounded(value: ic_wire::BytesRef<'_, 32>) -> ic_wire::BytesRef<'_, 32> {
        value
    }
}

#[cfg(ui_case = "borrow_async")]
#[query(borrow)]
async fn borrow_async(_: &str) {}

#[cfg(ui_case = "borrow_unit")]
#[query(borrow)]
fn borrow_unit() {}

#[cfg(ui_case = "borrow_escape")]
#[query(borrow)]
fn borrow_escape(value: &str) -> &'static str {
    value
}

#[cfg(ui_case = "invalid_limits")]
#[query(request = 5)]
fn invalid_limits(_: u64) -> u64 {
    0
}

#[cfg(ui_case = "duplicate")]
#[query(id = 1, id = 2, request = LIMITS, response = LIMITS)]
fn duplicate() {}

#[cfg(ui_case = "multiple")]
#[query(id = 1, request = LIMITS, response = LIMITS)]
fn multiple(_: u64, _: u64) {}

#[cfg(ui_case = "borrowed")]
#[query(id = 1, request = LIMITS, response = LIMITS)]
fn borrowed(_: &str) {}

#[cfg(ui_case = "nonconstant")]
#[query(id = 1, request = dynamic_limits(), response = LIMITS)]
fn nonconstant() {}
fn dynamic_limits() -> Limits {
    LIMITS
}

#[cfg(ui_case = "nonserde")]
struct Unsupported;
#[cfg(ui_case = "nonserde")]
#[query(id = 1, request = LIMITS, response = LIMITS)]
fn nonserde(_: Unsupported) {}

#[cfg(ui_case = "composite_update")]
#[ic_wire_endpoint::update(composite, id = 1, request = LIMITS, response = LIMITS)]
fn composite_update() {}

#[cfg(ui_case = "name")]
#[query(name = "", id = 1, request = LIMITS, response = LIMITS)]
fn name() {}
