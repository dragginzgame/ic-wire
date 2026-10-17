//! Minimal installed browser transport fixture; application policy is consumer-owned.
use crate::commands::{Apply, Failure, Inspect, Receipt, Snapshot};
use ic_wire::{Method, decode_request, encode_response};
use std::cell::Cell;

thread_local! { static BALANCE: Cell<u64> = const { Cell::new(0) }; }

#[ic_cdk::update(manual_reply = true)]
fn runtime_apply() {
    if !ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()) {
        ic_cdk::api::msg_reject("controller required");
        return;
    }
    if ic0::msg_arg_data_size() > Apply::REQUEST_LIMITS.bytes {
        ic_cdk::api::msg_reject("request limit");
        return;
    }
    let request = decode_request::<Apply>(&ic_cdk::api::msg_arg_data()).expect("request codec");
    let balance = BALANCE.get();
    let next = balance.checked_add(request.amount);
    let receipt = Receipt {
        id: request.id,
        balance: next.unwrap_or(balance),
    };
    let response = if next.is_some() {
        Ok(receipt)
    } else {
        Err(Failure::Overflow(receipt))
    };
    let reply = encode_response::<Apply>(&response).expect("response codec");
    BALANCE.set(next.unwrap_or(balance));
    ic_cdk::api::msg_reply(reply.as_bytes());
}

#[ic_cdk::query(manual_reply = true)]
fn runtime_inspect() {
    if ic0::msg_arg_data_size() > Inspect::REQUEST_LIMITS.bytes {
        ic_cdk::api::msg_reject("request limit");
        return;
    }
    let response = decode_request::<Inspect>(&ic_cdk::api::msg_arg_data()).and_then(|inspection| {
        encode_response::<Inspect>(&Snapshot {
            balance: BALANCE.get(),
            inspection,
        })
    });
    match response {
        Ok(reply) => ic_cdk::api::msg_reply(reply.as_bytes()),
        Err(error) => ic_cdk::api::msg_reject(error.to_string()),
    }
}
