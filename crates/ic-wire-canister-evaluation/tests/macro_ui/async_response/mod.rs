use crate::support::*;
#[ic_wire_adapter_fixture::update(method = Read, guard = guard)]
async fn handler(_: u64) -> bool {
    true
}
