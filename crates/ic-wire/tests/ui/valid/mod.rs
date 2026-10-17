use crate::support::*;
fn check() {
    let _ = adapter(&[], |_| Ok(7));
    let _future = async_adapter(&[], |_| async { Ok(7) });
}
