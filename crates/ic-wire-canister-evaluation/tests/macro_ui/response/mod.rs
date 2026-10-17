use crate::support::*;
#[ic_wire_adapter_fixture::query(method = Read, guard = guard)]
fn handler(_: u64) -> bool {
    true
}
