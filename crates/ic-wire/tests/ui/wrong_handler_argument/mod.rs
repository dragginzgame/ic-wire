use crate::support::*;
fn invalid() {
    let _ = adapter(&[], |_: bool| Ok(7));
}
