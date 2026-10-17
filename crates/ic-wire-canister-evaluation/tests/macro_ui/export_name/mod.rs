use crate::support::*;
#[ic_wire_derive::query(method = Read, guard = guard)]
fn wrong_export(value: u64) -> u64 {
    value
}
