use crate::support::*;
#[ic_wire_derive::update(method = Read, guard = guard)]
async fn handler(_: u64) -> bool {
    true
}
