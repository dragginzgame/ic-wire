//! Minimal installed browser transport fixture; application policy is consumer-owned.
use crate::commands::{Apply, Failure, Inspect, Receipt, Snapshot};
use ic_wire::{Method, decode_request, encode_response};
use std::cell::Cell;

thread_local! { static BALANCE: Cell<u64> = const { Cell::new(0) }; }

#[ic_cdk::update(manual_reply = true, hidden = true)]
fn runtime_apply() {
    if !ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()) {
        ic_platform::ic0::msg_reject(b"controller required");
        return;
    }
    if ic_platform::ic0::msg_arg_data_size() > Apply::REQUEST_LIMITS.bytes {
        ic_platform::ic0::msg_reject(b"request limit");
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
    ic_platform::ic0::msg_reply_data_append(reply.as_bytes());
    ic_platform::ic0::msg_reply();
}

#[ic_cdk::query(manual_reply = true, hidden = true)]
fn runtime_inspect() {
    if ic_platform::ic0::msg_arg_data_size() > Inspect::REQUEST_LIMITS.bytes {
        ic_platform::ic0::msg_reject(b"request limit");
        return;
    }
    let response = decode_request::<Inspect>(&ic_cdk::api::msg_arg_data()).and_then(|inspection| {
        encode_response::<Inspect>(&Snapshot {
            balance: BALANCE.get(),
            inspection,
        })
    });
    match response {
        Ok(reply) => {
            ic_platform::ic0::msg_reply_data_append(reply.as_bytes());
            ic_platform::ic0::msg_reply();
        }
        Err(error) => ic_platform::ic0::msg_reject(error.to_string().as_bytes()),
    }
}

#[ic_cdk::query(manual_reply = true, hidden = true)]
fn native_echo() {
    use crate::commands::{NATIVE_ID, NATIVE_LIMITS, NativeTree};
    if ic_platform::ic0::msg_arg_data_size() > NATIVE_LIMITS.bytes {
        ic_platform::ic0::msg_reject(b"request limit");
        return;
    }
    let response = ic_wire::native::decode_request::<NativeTree>(
        NATIVE_ID,
        &ic_cdk::api::msg_arg_data(),
        NATIVE_LIMITS,
    )
    .and_then(|value| ic_wire::native::encode_response(NATIVE_ID, &value, NATIVE_LIMITS));
    match response {
        Ok(reply) => {
            ic_platform::ic0::msg_reply_data_append(&reply);
            ic_platform::ic0::msg_reply();
        }
        Err(error) => ic_platform::ic0::msg_reject(error.to_string().as_bytes()),
    }
}
