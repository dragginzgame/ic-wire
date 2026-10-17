use crate::support::*;
fn invalid() {
    let _ = adapter(&[], |_| 7_u64);
}
