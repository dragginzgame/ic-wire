use crate::support::*;
#[ic_wire_adapter_fixture::update(method = Read, guard = guard)]
unsafe fn handler(value: u64) -> u64 {
    value
}
