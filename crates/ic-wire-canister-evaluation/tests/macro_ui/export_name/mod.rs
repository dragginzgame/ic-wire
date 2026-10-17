use crate::support::*;
#[ic_wire_adapter_fixture::query(method = Read, guard = guard)]
fn wrong_export(value: u64) -> u64 {
    value
}
