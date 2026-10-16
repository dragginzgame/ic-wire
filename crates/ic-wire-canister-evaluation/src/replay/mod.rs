//! Postcard-only command/replay probe. Storage and policy belong to this fixture.
use crate::commands::{Apply, Command, Failure, Inspect, Receipt, Snapshot};
use candid::Principal;
use ic_cdk::stable;
use ic_wire::{Method, decode_request, encode_response};

const MAX_REQUEST: usize = Apply::REQUEST_LIMITS.bytes;
const MAX_REPLY: usize = Apply::RESPONSE_LIMITS.bytes;
const CAPACITY: usize = 8;
const SLOT: usize = 2 + MAX_REQUEST + MAX_REPLY;
const STATE_BYTES: usize = 16 + CAPACITY * SLOT;

#[derive(Debug, PartialEq)]
enum CommandError {
    Codec,
    Caller,
    Audience,
    Expired,
    Conflict,
    Capacity,
    State,
}

fn request(bytes: &[u8]) -> Result<Command, CommandError> {
    decode_request::<Apply>(bytes).map_err(|_| CommandError::Codec)
}

// All checks and reply construction precede mutation. No await or external
// effect: the IC commits the local effect and receipt in one update transaction.
fn execute(
    state: &mut [u8; STATE_BYTES],
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
    let count = u64::from_le_bytes(state[8..16].try_into().unwrap());
    if count > CAPACITY as u64 {
        return Err(CommandError::State);
    }
    for slot in state[16..]
        .as_chunks::<SLOT>()
        .0
        .iter()
        .take(count as usize)
    {
        let (request_len, reply_len) = (usize::from(slot[0]), usize::from(slot[1]));
        if request_len > MAX_REQUEST || reply_len > MAX_REPLY {
            return Err(CommandError::State);
        }
        let retained = &slot[2..2 + request_len];
        let previous = request(retained).map_err(|_| CommandError::State)?;
        if previous.subject == value.subject && previous.id == value.id {
            if retained != bytes {
                return Err(CommandError::Conflict);
            }
            return Ok(slot[2 + MAX_REQUEST..2 + MAX_REQUEST + reply_len].to_vec());
        }
    }
    if count == CAPACITY as u64 {
        return Err(CommandError::Capacity);
    }
    let balance = u64::from_le_bytes(state[..8].try_into().unwrap());
    let next = balance.checked_add(value.amount);
    let receipt = Receipt {
        id: value.id,
        balance: next.unwrap_or(balance),
    };
    let response = if next.is_some() {
        Ok(receipt)
    } else {
        Err(Failure::Overflow(receipt))
    };
    let reply = encode_response::<Apply>(&response).map_err(|_| CommandError::Codec)?;
    let reply = reply.as_bytes();
    let next = next.unwrap_or(balance);
    let slot = &mut state[16 + count as usize * SLOT..16 + (count as usize + 1) * SLOT];
    slot[0] = bytes.len() as u8;
    slot[1] = reply.len() as u8;
    slot[2..2 + bytes.len()].copy_from_slice(bytes);
    slot[2 + MAX_REQUEST..2 + MAX_REQUEST + reply.len()].copy_from_slice(reply);
    state[..8].copy_from_slice(&next.to_le_bytes());
    state[8..16].copy_from_slice(&(count + 1).to_le_bytes());
    Ok(reply.to_vec())
}

fn run(trap_after_write: bool) {
    let caller = ic_cdk::api::msg_caller();
    if !ic_cdk::api::is_controller(&caller) {
        ic_cdk::api::msg_reject("controller required");
        return;
    }
    if ic0::msg_arg_data_size() > MAX_REQUEST {
        ic_cdk::api::msg_reject("request limit");
        return;
    }
    let mut state = [0; STATE_BYTES];
    if stable::stable_size() > 0 {
        stable::stable_read(0, &mut state);
    }
    match execute(
        &mut state,
        &ic_cdk::api::msg_arg_data(),
        caller,
        ic_cdk::api::canister_self(),
        ic_cdk::api::time(),
    ) {
        Ok(reply) => {
            if stable::stable_size() == 0 {
                stable::stable_grow(1).expect("fixture stable page");
            }
            stable::stable_write(0, &state);
            if trap_after_write {
                ic_cdk::trap("injected interruption before transaction commit");
            }
            ic_cdk::api::msg_reply(reply);
        }
        Err(error) => ic_cdk::api::msg_reject(format!("command {error:?}")),
    }
}

#[ic_cdk::update(manual_reply = true)]
fn replay_command() {
    run(false);
}

// Controller-only fault injection, absent from ordinary measurement artifacts.
#[ic_cdk::update(manual_reply = true)]
fn replay_trap() {
    run(true);
}

#[cfg(test)]
mod tests;

// This is the complete transport adapter a Canic query macro needs to emit,
// alongside its own startup/access stages. No codec dependency leaks into CDK.
#[ic_cdk::query(manual_reply = true)]
fn runtime_inspect() {
    if ic0::msg_arg_data_size() > Inspect::REQUEST_LIMITS.bytes {
        ic_cdk::api::msg_reject("request limit");
        return;
    }
    let response = decode_request::<Inspect>(&ic_cdk::api::msg_arg_data()).and_then(|inspection| {
        let mut bytes = [0; 8];
        if stable::stable_size() > 0 {
            stable::stable_read(0, &mut bytes);
        }
        encode_response::<Inspect>(&Snapshot {
            balance: u64::from_le_bytes(bytes),
            inspection,
        })
    });
    match response {
        Ok(reply) => ic_cdk::api::msg_reply(reply.as_bytes()),
        Err(error) => ic_cdk::api::msg_reject(error.to_string()),
    }
}

// Controller-only transport probe. Retries are specific to this qualified
// receipt-backed receiver, not automatic policy for arbitrary update methods.
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
