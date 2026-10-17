use crate::support::*;
#[ic_wire_derive::update(method = Read, guard = guard)]
unsafe fn handler(value: u64) -> u64 {
    value
}
