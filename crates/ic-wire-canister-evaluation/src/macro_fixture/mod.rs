//! Local macro integration proof; auth/startup policy here is deliberately synthetic.
use ic_wire::{Limits, Method};

/// Query binding.
pub struct Read;
/// Async update binding.
pub struct Add;
/// Synchronous empty-request/unit-response update binding.
pub struct Reset;
/// Controller-only fault injection: response budget cannot admit its type.
pub struct BadReply;
const LIMITS: Limits = Limits {
    bytes: 64,
    allocation: 128,
    nodes: 8,
    depth: 4,
};
impl Method for Read {
    type Request = u64;
    type Response = u64;
    const ID: u64 = 0x4d515259;
    const NAME: &'static str = "macro_read";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}
impl Method for Add {
    type Request = u64;
    type Response = Result<u64, u8>;
    const ID: u64 = 0x4d555044;
    const NAME: &'static str = "macro_add";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}
impl Method for Reset {
    type Request = ();
    type Response = ();
    const ID: u64 = 0x4d525354;
    const NAME: &'static str = "macro_reset";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}
impl Method for BadReply {
    type Request = ();
    type Response = u64;
    const ID: u64 = 0x4d424144;
    const NAME: &'static str = "macro_bad_reply";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = Limits {
        bytes: 11,
        ..LIMITS
    };
}
#[ic_wire_adapter_fixture::update(method = BadReply, guard = unit_guard)]
fn macro_bad_reply(_: ()) -> u64 {
    transport::set_balance(999);
    999
}
fn unit_guard(_: &()) -> Result<(), &'static str> {
    transport::event("guard");
    Ok(())
}
#[ic_wire_adapter_fixture::update(method = Reset, guard = unit_guard)]
fn macro_reset(_: ()) {
    transport::event("handler");
    transport::set_balance(0);
}
fn guard(value: &u64) -> Result<(), &'static str> {
    transport::event("guard");
    if *value == 13 {
        Err("request guard")
    } else {
        Ok(())
    }
}
#[ic_wire_adapter_fixture::query(method = Read, guard = guard)]
fn macro_read(correlation: u64) -> u64 {
    transport::event("handler");
    transport::balance().wrapping_add(correlation)
}
#[ic_wire_adapter_fixture::update(method = Add, guard = guard)]
async fn macro_add(amount: u64) -> Result<u64, u8> {
    transport::event("handler");
    // Exercise async expansion without claiming external-await recovery.
    let amount = core::future::ready(amount).await;
    let balance = transport::balance().checked_add(amount).ok_or(7_u8)?;
    transport::set_balance(balance);
    Ok(balance)
}

#[cfg(target_arch = "wasm32")]
mod transport {
    pub fn event(_: &'static str) {}
    pub fn startup() -> Result<(), &'static str> {
        Ok(())
    }
    pub fn argument_size() -> usize {
        ic0::msg_arg_data_size()
    }
    pub fn authorize_caller() -> Result<(), &'static str> {
        if ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()) {
            Ok(())
        } else {
            Err("caller")
        }
    }
    pub fn arguments() -> Vec<u8> {
        ic_cdk::api::msg_arg_data()
    }
    pub fn reply(bytes: &[u8]) {
        ic_cdk::api::msg_reply(bytes);
    }
    pub fn reject(message: &str) {
        ic_cdk::api::msg_reject(message);
    }
    pub fn trap(message: &str) -> ! {
        ic_cdk::trap(message)
    }
    pub fn balance() -> u64 {
        let mut bytes = [0; 8];
        if ic_cdk::stable::stable_size() > 0 {
            ic_cdk::stable::stable_read(0, &mut bytes);
        }
        u64::from_le_bytes(bytes)
    }
    pub fn set_balance(value: u64) {
        if ic_cdk::stable::stable_size() == 0 {
            ic_cdk::stable::stable_grow(1).unwrap();
        }
        ic_cdk::stable::stable_write(0, &value.to_le_bytes());
    }
}
#[cfg(not(target_arch = "wasm32"))]
mod transport {
    use std::cell::RefCell;
    #[derive(Default)]
    pub struct State {
        pub input: Vec<u8>,
        pub reply: Vec<u8>,
        pub rejection: String,
        pub events: Vec<&'static str>,
        pub balance: u64,
        pub stopped: bool,
        pub denied: bool,
    }
    thread_local! { pub static STATE: RefCell<State> = RefCell::default(); }
    pub fn event(name: &'static str) {
        STATE.with_borrow_mut(|s| s.events.push(name));
    }
    pub fn startup() -> Result<(), &'static str> {
        event("startup");
        STATE.with_borrow(|s| if s.stopped { Err("startup") } else { Ok(()) })
    }
    pub fn argument_size() -> usize {
        event("size");
        STATE.with_borrow(|s| s.input.len())
    }
    pub fn authorize_caller() -> Result<(), &'static str> {
        event("caller");
        STATE.with_borrow(|s| if s.denied { Err("caller") } else { Ok(()) })
    }
    pub fn arguments() -> Vec<u8> {
        event("copy");
        STATE.with_borrow(|s| s.input.clone())
    }
    pub fn reply(bytes: &[u8]) {
        event("reply");
        STATE.with_borrow_mut(|s| s.reply = bytes.to_vec());
    }
    pub fn reject(message: &str) {
        event("reject");
        STATE.with_borrow_mut(|s| s.rejection = message.into());
    }
    pub fn trap(message: &str) -> ! {
        panic!("{message}")
    }
    pub fn balance() -> u64 {
        STATE.with_borrow(|s| s.balance)
    }
    pub fn set_balance(value: u64) {
        STATE.with_borrow_mut(|s| s.balance = value);
    }
}

#[cfg(test)]
mod tests;
