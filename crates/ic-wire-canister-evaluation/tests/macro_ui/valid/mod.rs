use crate::support::*;
#[ic_wire_derive::query(method = Read, guard = guard)]
fn handler(value: u64) -> u64 {
    value
}
mod sync_update {
    use crate::support::*;
    #[ic_wire_derive::update(method = Read, guard = guard)]
    fn handler(value: u64) -> u64 {
        value
    }
}
mod async_update {
    use crate::support::*;
    #[ic_wire_derive::update(method = Read, guard = guard)]
    async fn handler(value: u64) -> u64 {
        value
    }
}
