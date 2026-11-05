use crate::support::*;
fn invalid() {
    let _future = async_adapter(&[], |_| async { 7_u64 });
}
