use crate::support::*;
fn invalid() {
    let _ = ic_wire::encode_response::<Read>(&7_u64);
}
