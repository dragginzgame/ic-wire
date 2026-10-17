use crate::support::*;
#[ic_wire_adapter_fixture::query(method = Read, guard = guard)]
fn handler(value: u64) -> u64 {
    value
}
mod sync_update {
    use crate::support::*;
    #[ic_wire_adapter_fixture::update(method = Read, guard = guard)]
    fn handler(value: u64) -> u64 {
        value
    }
}
mod async_update {
    use crate::support::*;
    #[ic_wire_adapter_fixture::update(method = Read, guard = guard)]
    async fn handler(value: u64) -> u64 {
        value
    }
}
