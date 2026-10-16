//! Local two-canister transport probe. No durable replay or user auth protocol.
use super::{CONTRACT, MAX_WIRE, Message, decode, decode_value};
use candid::Principal;
use std::cell::RefCell;

thread_local! {
    static TARGET: RefCell<Option<Principal>> = const { RefCell::new(None) };
    static OBSERVATION: RefCell<Observation> = RefCell::new(Observation::default());
}

#[derive(Default)]
struct Observation {
    calls: u64,
    previous: Vec<u8>,
    repeated: bool,
    caller: Option<Principal>,
}

#[derive(Debug, PartialEq)]
enum PeerError {
    Unconfigured,
    Request,
    Transport,
    ReplyCodec,
    ReplyIdentity,
    ReplyValue,
    RetryMismatch,
}

// The receive path already checked the outer bound and canonical request.
pub(super) fn observe(bytes: &[u8]) {
    OBSERVATION.with_borrow_mut(|state| {
        state.calls += 1;
        state.repeated = state.previous == bytes;
        state.previous = bytes.to_vec();
        state.caller = Some(ic_cdk::api::msg_caller());
    });
}

#[ic_cdk::query(manual_reply = true)]
fn peer_observation() {
    OBSERVATION.with_borrow(|state| {
        let mut bytes = state.calls.to_le_bytes().to_vec();
        bytes.push(u8::from(state.repeated));
        let caller = state.caller.as_ref().map_or(&[][..], Principal::as_slice);
        bytes.push(caller.len() as u8);
        bytes.extend_from_slice(caller);
        ic_cdk::api::msg_reply(bytes);
    });
}

// Fixture configuration is controller-only. It is not an application endpoint.
#[ic_cdk::update(manual_reply = true)]
fn configure_peer() {
    if !ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()) {
        ic_cdk::api::msg_reject("controller required");
        return;
    }
    if ic0::msg_arg_data_size() > 29 {
        ic_cdk::api::msg_reject("principal limit");
        return;
    }
    let target = Principal::from_slice(&ic_cdk::api::msg_arg_data());
    TARGET.with_borrow_mut(|stored| *stored = Some(target));
    ic_cdk::api::msg_reply([]);
}

fn validate_reply(request: &Message, bytes: &[u8]) -> Result<(), PeerError> {
    let reply: Message = decode_value(bytes).map_err(|_| PeerError::ReplyCodec)?;
    if reply.contract != CONTRACT || reply.direction != 1 {
        return Err(PeerError::ReplyIdentity);
    }
    if reply.account != request.account
        || reply.revision != request.revision.wrapping_add(1)
        || reply.payload != request.payload
    {
        return Err(PeerError::ReplyValue);
    }
    Ok(())
}

async fn forward(bytes: Vec<u8>) -> Result<Vec<u8>, PeerError> {
    let request = decode(&bytes).map_err(|_| PeerError::Request)?;
    let target = TARGET
        .with_borrow(|stored| *stored)
        .ok_or(PeerError::Unconfigured)?;
    let mut first = None;
    for _ in 0..2 {
        // Deliberate repetition of a stateless experiment, not an automatic retry
        // policy for arbitrary effects. Reuse these exact bytes across awaits.
        let response = ic_cdk::call::Call::unbounded_wait(target, "evaluate")
            .with_raw_args(&bytes)
            .await
            .map_err(|_| PeerError::Transport)?
            .into_bytes();
        validate_reply(&request, &response)?;
        if let Some(previous) = &first {
            if previous != &response {
                return Err(PeerError::RetryMismatch);
            }
        } else {
            first = Some(response);
        }
    }
    Ok(first.expect("two calls completed"))
}

#[ic_cdk::update(manual_reply = true)]
async fn evaluate_via_peer() {
    if ic0::msg_arg_data_size() > MAX_WIRE {
        ic_cdk::api::msg_reject("wire limit");
        return;
    }
    match forward(ic_cdk::api::msg_arg_data()).await {
        Ok(reply) => ic_cdk::api::msg_reply(reply),
        Err(error) => ic_cdk::api::msg_reject(format!("peer {error:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Blob, encode};

    #[test]
    fn reply_validation_checks_bounds_identity_and_value() {
        let request = Message {
            contract: CONTRACT,
            direction: 0,
            account: u64::MAX,
            revision: u64::MAX,
            payload: Blob::new(&[1, 2]).unwrap(),
        };
        let mut reply = request.clone();
        reply.direction = 1;
        reply.revision = 0;
        assert_eq!(validate_reply(&request, &encode(&reply).unwrap()), Ok(()));
        assert_eq!(
            validate_reply(&request, &vec![0; MAX_WIRE + 1]),
            Err(PeerError::ReplyCodec)
        );
        assert_eq!(validate_reply(&request, &[0]), Err(PeerError::ReplyCodec));
        let mut trailing = encode(&reply).unwrap();
        trailing.push(0);
        assert_eq!(
            validate_reply(&request, &trailing),
            Err(PeerError::ReplyCodec)
        );
        reply.contract += 1;
        assert_eq!(
            validate_reply(&request, &encode(&reply).unwrap()),
            Err(PeerError::ReplyIdentity)
        );
        reply.contract = CONTRACT;
        reply.direction = 0;
        assert_eq!(
            validate_reply(&request, &encode(&reply).unwrap()),
            Err(PeerError::ReplyIdentity)
        );
        reply.direction = 1;
        reply.account = 1;
        assert_eq!(
            validate_reply(&request, &encode(&reply).unwrap()),
            Err(PeerError::ReplyValue)
        );
        reply.account = request.account;
        reply.revision = 1;
        assert_eq!(
            validate_reply(&request, &encode(&reply).unwrap()),
            Err(PeerError::ReplyValue)
        );
        reply.revision = 0;
        reply.payload = Blob::new(&[]).unwrap();
        assert_eq!(
            validate_reply(&request, &encode(&reply).unwrap()),
            Err(PeerError::ReplyValue)
        );
    }
}
