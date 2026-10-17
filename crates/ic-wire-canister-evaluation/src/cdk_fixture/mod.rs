//! Installed raw endpoint proof; permissions and state remain consumer-owned.
use ic_wire::{Limits, Method};
/// Read the current fixture state.
pub struct Read;
/// Write state, then deliberately fail response admission.
pub struct BadReply;
const LIMITS: Limits = Limits {
    bytes: 32,
    allocation: 64,
    nodes: 4,
    depth: 4,
};
impl Method for Read {
    type Request = ();
    type Response = u64;
    const ID: u64 = 0x43444b01;
    const NAME: &'static str = "cdk_read";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}
impl Method for BadReply {
    type Request = u64;
    type Response = u64;
    const ID: u64 = 0x43444b02;
    const NAME: &'static str = "cdk_bad_reply";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = Limits {
        bytes: 11,
        ..LIMITS
    };
}
#[cfg(target_arch = "wasm32")]
fn authorize() {
    assert!(
        ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()),
        "controller required"
    );
}
#[cfg(target_arch = "wasm32")]
#[ic_cdk::query(manual_reply = true, hidden = true)]
fn cdk_read() {
    ic_wire_cdk::endpoint::read::<Read>(authorize);
    let mut bytes = [0; 8];
    if ic_cdk::stable::stable_size() > 0 {
        ic_cdk::stable::stable_read(0, &mut bytes);
    }
    ic_wire_cdk::endpoint::reply::<Read>(&u64::from_le_bytes(bytes));
}
#[cfg(target_arch = "wasm32")]
#[ic_cdk::update(manual_reply = true, hidden = true)]
fn cdk_bad_reply() {
    let value = ic_wire_cdk::endpoint::read::<BadReply>(authorize);
    if ic_cdk::stable::stable_size() == 0 {
        ic_cdk::stable::stable_grow(1).unwrap();
    }
    ic_cdk::stable::stable_write(0, &value.to_le_bytes());
    ic_wire_cdk::endpoint::reply::<BadReply>(&value);
}
