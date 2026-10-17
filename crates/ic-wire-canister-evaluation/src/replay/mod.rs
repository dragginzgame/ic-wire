//! Browser transport fixture. Durable receipt-policy qualification lives in Relay.
use crate::commands::{Apply, Command, Failure, Inspect, Receipt, Snapshot};
use candid::Principal;
use ic_wire::{Method, decode_request, encode_response};
use std::cell::RefCell;

const MAX_REQUEST: usize = Apply::REQUEST_LIMITS.bytes;
// Bound this disposable browser fixture; no persistence or upgrade contract.
const CAPACITY: usize = 8;
#[derive(Default)]
struct State {
    balance: u64,
    replies: Vec<(Vec<u8>, Vec<u8>)>,
}
thread_local! { static STATE: RefCell<State> = RefCell::default(); }

#[derive(Debug, PartialEq)]
enum CommandError {
    Codec,
    Caller,
    Audience,
    Expired,
    Conflict,
    Capacity,
}

fn request(bytes: &[u8]) -> Result<Command, CommandError> {
    decode_request::<Apply>(bytes).map_err(|_| CommandError::Codec)
}

fn execute(
    state: &mut State,
    bytes: &[u8],
    caller: Principal,
    audience: Principal,
    now: u64,
) -> Result<Vec<u8>, CommandError> {
    let value = request(bytes)?;
    if value.subject.as_slice() != caller.as_slice() {
        return Err(CommandError::Caller);
    }
    if value.audience.as_slice() != audience.as_slice() {
        return Err(CommandError::Audience);
    }
    if value.expiry <= now {
        return Err(CommandError::Expired);
    }
    for (retained, reply) in &state.replies {
        let previous = request(retained)?;
        if previous.subject == value.subject && previous.id == value.id {
            if retained != bytes {
                return Err(CommandError::Conflict);
            }
            return Ok(reply.clone());
        }
    }
    if state.replies.len() == CAPACITY {
        return Err(CommandError::Capacity);
    }
    let next = state.balance.checked_add(value.amount);
    let receipt = Receipt {
        id: value.id,
        balance: next.unwrap_or(state.balance),
    };
    let response = if next.is_some() {
        Ok(receipt)
    } else {
        Err(Failure::Overflow(receipt))
    };
    let reply = encode_response::<Apply>(&response)
        .map_err(|_| CommandError::Codec)?
        .into_bytes();
    state.balance = next.unwrap_or(state.balance);
    state.replies.push((bytes.to_vec(), reply.clone()));
    Ok(reply)
}

#[ic_cdk::update(manual_reply = true)]
fn replay_command() {
    let caller = ic_cdk::api::msg_caller();
    if !ic_cdk::api::is_controller(&caller) {
        ic_cdk::api::msg_reject("controller required");
        return;
    }
    if ic0::msg_arg_data_size() > MAX_REQUEST {
        ic_cdk::api::msg_reject("request limit");
        return;
    }
    let response = STATE.with_borrow_mut(|state| {
        execute(
            state,
            &ic_cdk::api::msg_arg_data(),
            caller,
            ic_cdk::api::canister_self(),
            ic_cdk::api::time(),
        )
    });
    match response {
        Ok(reply) => ic_cdk::api::msg_reply(reply),
        Err(error) => ic_cdk::api::msg_reject(format!("command {error:?}")),
    }
}

// This is the complete transport adapter a Canic query macro needs to emit,
// alongside its own startup/access stages. No codec dependency leaks into CDK.
#[ic_cdk::query(manual_reply = true)]
fn runtime_inspect() {
    if ic0::msg_arg_data_size() > Inspect::REQUEST_LIMITS.bytes {
        ic_cdk::api::msg_reject("request limit");
        return;
    }
    let response = decode_request::<Inspect>(&ic_cdk::api::msg_arg_data()).and_then(|inspection| {
        encode_response::<Inspect>(&Snapshot {
            balance: STATE.with_borrow(|state| state.balance),
            inspection,
        })
    });
    match response {
        Ok(reply) => ic_cdk::api::msg_reply(reply.as_bytes()),
        Err(error) => ic_cdk::api::msg_reject(error.to_string()),
    }
}

// Controller-only transport probe. Retries are specific to this qualified
// volatile fixture receiver, not automatic policy for arbitrary update methods.
#[ic_cdk::update(manual_reply = true)]
async fn runtime_forward() {
    if !ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()) {
        ic_cdk::api::msg_reject("controller required");
        return;
    }
    if ic0::msg_arg_data_size() > Apply::REQUEST_LIMITS.bytes {
        ic_cdk::api::msg_reject("request limit");
        return;
    }
    let request = match decode_request::<Apply>(&ic_cdk::api::msg_arg_data()) {
        Ok(request) if request.subject.as_slice() == ic_cdk::api::canister_self().as_slice() => {
            request
        }
        _ => {
            ic_cdk::api::msg_reject("invalid forward request");
            return;
        }
    };
    let target = Principal::from_slice(request.audience.as_slice());
    let retained = ic_wire::encode_request::<Apply>(&request).expect("admitted request");
    let mut first = None;
    for _ in 0..2 {
        let response = match ic_cdk::call::Call::unbounded_wait(target, Apply::NAME)
            .with_raw_args(retained.as_bytes())
            .await
        {
            Ok(response) => response.into_bytes(),
            Err(_) => {
                ic_cdk::api::msg_reject("transport failure");
                return;
            }
        };
        let decoded = match ic_wire::decode_response::<Apply>(&response) {
            Ok(response) => response,
            Err(_) => {
                ic_cdk::api::msg_reject("reply codec failure");
                return;
            }
        };
        let receipt = match &decoded {
            Ok(receipt) | Err(Failure::Overflow(receipt)) => receipt,
        };
        if receipt.id != request.id {
            ic_cdk::api::msg_reject("reply request mismatch");
            return;
        }
        if let Some(previous) = &first {
            if previous != &response {
                ic_cdk::api::msg_reject("replay mismatch");
                return;
            }
        } else {
            first = Some(response);
        }
    }
    ic_cdk::api::msg_reply(first.expect("two verified replies"));
}
