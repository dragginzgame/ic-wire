use crate::support::*;
#[ic_wire_derive::query(method = Read, guard = guard)]
fn handler(_: bool) -> u64 {
    0
}
