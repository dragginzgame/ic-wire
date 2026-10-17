use crate::support::*;
#[ic_wire_adapter_fixture::query(method = Read, guard = guard)]
async fn handler(value: u64) -> u64 {
    value
}
