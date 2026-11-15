use super::Budget;
use crate::{Error, Limits};

#[test]
fn refusal_is_sticky_and_extreme_charges_cannot_wrap() {
    let budget = Budget::new(Limits {
        bytes: 0,
        allocation: usize::MAX,
        nodes: 2,
        depth: 1,
    });
    assert!(!budget.charge_buffer(usize::MAX));
    assert!(!budget.enter(1, 0));
    assert_eq!(budget.error(), Error::Limit);
    let budget = Budget::new(Limits {
        bytes: 2,
        allocation: 1,
        nodes: 2,
        depth: 1,
    });
    assert!(!budget.enter(1, 0));
    assert_eq!(budget.error(), Error::Limit);
}
