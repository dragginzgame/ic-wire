//! Postcard-only command/replay probe. Storage and policy belong to this fixture.
use candid::Principal;
use ic_cdk::stable;

const CONTRACT: u32 = 0x5250;
const MAX_REQUEST: usize = 128;
const MAX_REPLY: usize = 32;
const CAPACITY: usize = 8;
const SLOT: usize = 2 + MAX_REQUEST + MAX_REPLY;
const STATE_BYTES: usize = 16 + CAPACITY * SLOT;

// Explicit tuple authority: contract, direction, subject, audience, expiry,
// request identity, amount. Borrowed principal bytes require no input allocation.
type Request<'a> = (u32, u8, &'a [u8], &'a [u8], u64, u64, u64);

#[derive(Debug, PartialEq)]
enum CommandError {
    Limit,
    Codec,
    Noncanonical,
    Identity,
    Caller,
    Audience,
    Expired,
    Conflict,
    Capacity,
    State,
}

fn request(bytes: &[u8]) -> Result<Request<'_>, CommandError> {
    if bytes.len() > MAX_REQUEST {
        return Err(CommandError::Limit);
    }
    let (value, rest): (Request<'_>, _) =
        postcard::take_from_bytes(bytes).map_err(|_| CommandError::Codec)?;
    let mut buffer = [0; MAX_REQUEST];
    if !rest.is_empty()
        || postcard::to_slice(&value, &mut buffer).map_err(|_| CommandError::Codec)? != bytes
    {
        return Err(CommandError::Noncanonical);
    }
    if value.0 != CONTRACT || value.1 != 0 {
        return Err(CommandError::Identity);
    }
    Ok(value)
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
    if value.2 != caller.as_slice() {
        return Err(CommandError::Caller);
    }
    if value.3 != audience.as_slice() {
        return Err(CommandError::Audience);
    }
    if value.4 <= now {
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
        if previous.2 == value.2 && previous.5 == value.5 {
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
    // Explicit application result tag: 0 = applied; 1 = arithmetic overflow.
    // Even an application failure is retained for this request identity.
    let (tag, next) = balance
        .checked_add(value.6)
        .map_or((1_u8, balance), |n| (0, n));
    let mut buffer = [0; MAX_REPLY];
    let reply = postcard::to_slice(&(CONTRACT, 1_u8, value.5, tag, next), &mut buffer)
        .map_err(|_| CommandError::Codec)?;
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
