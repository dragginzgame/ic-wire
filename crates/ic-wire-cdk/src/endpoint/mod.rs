//! Method-specific input admission and trap-based response encoding.
use ic_wire::Method;

/// Authorize, check the method's raw byte cap, copy and decode its request.
///
/// `authorize` must return only for an admitted caller; it can trap on refusal.
/// It runs before even reading the argument size. Both over-limit and malformed
/// inputs trap before a handler can run. Use a raw/manual-reply CDK export with
/// no generated argument decoder, then call this synchronously before awaiting.
pub fn read<M: Method>(authorize: impl FnOnce()) -> M::Request {
    read_with::<M>(authorize, ic0::msg_arg_data_size, ic_cdk::api::msg_arg_data)
}

fn read_with<M: Method>(
    authorize: impl FnOnce(),
    size: impl FnOnce() -> usize,
    copy: impl FnOnce() -> Vec<u8>,
) -> M::Request {
    authorize();
    assert!(size() <= M::REQUEST_LIMITS.bytes, "request limit");
    ic_wire::decode_request::<M>(&copy()).expect("request codec")
}

/// Encode and send a method's response, trapping if encoding fails.
///
/// A reject after handler writes could commit them. Encoding failure therefore
/// traps so writes in the current IC message roll back. This does not roll back
/// earlier messages or effects before an await. Application errors encoded in
/// `M::Response` are ordinary replies and retain the handler's commit semantics.
pub fn reply<M: Method>(response: &M::Response) {
    let bytes = ic_wire::encode_response::<M>(response).expect("response codec");
    ic_cdk::api::msg_reply(bytes.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_wire::{Limits, encode_request};
    use std::{
        cell::RefCell,
        panic::{AssertUnwindSafe, catch_unwind},
    };

    struct Read;
    impl Method for Read {
        type Request = u64;
        type Response = u64;
        const ID: u64 = 1;
        const NAME: &'static str = "read";
        const REQUEST_LIMITS: Limits = Limits {
            bytes: 32,
            allocation: 64,
            nodes: 2,
            depth: 2,
        };
        const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
    }

    #[test]
    fn admission_precedes_copy_and_uses_the_selected_method_limit() {
        let events = RefCell::new(Vec::new());
        let bytes = encode_request::<Read>(&42).unwrap().into_bytes();
        let result = read_with::<Read>(
            || events.borrow_mut().push("authorize"),
            || {
                events.borrow_mut().push("size");
                bytes.len()
            },
            || {
                events.borrow_mut().push("copy");
                bytes.clone()
            },
        );
        assert_eq!(result, 42);
        assert_eq!(*events.borrow(), ["authorize", "size", "copy"]);
        let error = catch_unwind(AssertUnwindSafe(|| {
            read_with::<Read>(
                || {},
                || Read::REQUEST_LIMITS.bytes + 1,
                || panic!("must not copy"),
            )
        }))
        .unwrap_err();
        assert_eq!(error.downcast_ref::<&str>(), Some(&"request limit"));
    }

    #[test]
    fn denied_call_never_reads_size_or_arguments() {
        let error = catch_unwind(|| {
            read_with::<Read>(
                || panic!("denied"),
                || panic!("must not size"),
                || panic!("must not copy"),
            )
        })
        .unwrap_err();
        assert_eq!(error.downcast_ref::<&str>(), Some(&"denied"));
    }

    #[test]
    fn copied_malformed_arguments_never_reach_a_handler() {
        let error =
            catch_unwind(|| read_with::<Read>(|| {}, || 6, || b"DIDL\0\0".to_vec())).unwrap_err();
        assert!(
            error
                .downcast_ref::<String>()
                .unwrap()
                .contains("request codec")
        );
    }
}
