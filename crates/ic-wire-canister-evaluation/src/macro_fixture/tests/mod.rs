use super::*;
use ic_wire::{decode_response, encode_request};
use std::{
    future::Future,
    task::{Context, Poll, Waker},
};

fn run_update() {
    let mut future = std::pin::pin!(__ic_wire_entry_macro_add());
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(())
    ));
}
#[test]
fn generated_pipeline_order_and_early_rejection() {
    for (input, stopped, denied, expected) in [
        (
            encode_request::<Add>(&7).unwrap().into_bytes(),
            false,
            false,
            vec![
                "startup", "size", "caller", "copy", "guard", "handler", "reply",
            ],
        ),
        (vec![], true, false, vec!["startup", "reject"]),
        (vec![0; 65], false, false, vec!["startup", "size", "reject"]),
        (
            vec![],
            false,
            true,
            vec!["startup", "size", "caller", "reject"],
        ),
        (
            vec![],
            false,
            false,
            vec!["startup", "size", "caller", "copy", "reject"],
        ),
        (
            encode_request::<Add>(&13).unwrap().into_bytes(),
            false,
            false,
            vec!["startup", "size", "caller", "copy", "guard", "reject"],
        ),
    ] {
        transport::STATE.with_borrow_mut(|s| {
            *s = transport::State {
                input,
                stopped,
                denied,
                ..Default::default()
            }
        });
        run_update();
        transport::STATE.with_borrow(|s| {
            assert_eq!(s.events, expected);
            assert_eq!(s.balance, if s.rejection.is_empty() { 7 } else { 0 });
            if s.rejection.is_empty() {
                assert_eq!(decode_response::<Add>(&s.reply).unwrap(), Ok(7));
            } else {
                assert!(s.reply.is_empty());
            }
        });
    }
}
#[test]
fn query_and_application_failure_remain_typed() {
    transport::STATE.with_borrow_mut(|s| {
        *s = transport::State {
            input: encode_request::<Read>(&2).unwrap().into_bytes(),
            balance: 9,
            ..Default::default()
        }
    });
    __ic_wire_entry_macro_read();
    transport::STATE.with_borrow(|s| {
        assert_eq!(decode_response::<Read>(&s.reply).unwrap(), 11);
        assert_eq!(s.balance, 9);
    });
    transport::STATE.with_borrow_mut(|s| {
        *s = transport::State {
            input: encode_request::<Add>(&u64::MAX).unwrap().into_bytes(),
            balance: 9,
            ..Default::default()
        }
    });
    run_update();
    transport::STATE.with_borrow(|s| {
        assert_eq!(decode_response::<Add>(&s.reply).unwrap(), Err(7));
        assert_eq!(s.balance, 9);
        assert!(s.rejection.is_empty());
    });
}

#[test]
fn synchronous_unit_update_uses_the_same_pipeline() {
    transport::STATE.with_borrow_mut(|s| {
        *s = transport::State {
            input: encode_request::<Reset>(&()).unwrap().into_bytes(),
            balance: 9,
            ..Default::default()
        }
    });
    __ic_wire_entry_macro_reset();
    transport::STATE.with_borrow(|s| {
        decode_response::<Reset>(&s.reply).unwrap();
        assert_eq!(s.reply.len(), 11);
        assert_eq!(s.balance, 0);
        assert_eq!(
            s.events,
            [
                "startup", "size", "caller", "copy", "guard", "handler", "reply"
            ]
        );
    });
}
