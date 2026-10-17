use crate::support::*;
fn invalid() {
    let _ = ic_wire::encode_request::<Read>(&true);
}
