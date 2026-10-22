use super::*;

#[test]
fn allocation_units_match_byte_accounting_at_remainders_and_overflow() {
    let allocations = (0..=80).chain([usize::MAX - 8, usize::MAX - 1, usize::MAX]);
    for allocation in allocations {
        for reserved in [0, 1, 11, 64, usize::MAX] {
            for first in [0, usize::MAX / 8, usize::MAX / 8 + 1, usize::MAX] {
                let budget = Budget::new(Limits {
                    bytes: reserved,
                    allocation,
                    nodes: 16,
                    depth: 8,
                });
                let mut remaining = allocation.saturating_sub(reserved);
                let mut refused = allocation < reserved;
                for charge in [
                    first,
                    0,
                    1,
                    7,
                    2,
                    usize::MAX / 8,
                    usize::MAX / 8 + 1,
                    usize::MAX,
                    0,
                ] {
                    // Independent reference: the previous byte-denominated arithmetic.
                    let next = charge
                        .checked_mul(8)
                        .and_then(|cost| remaining.checked_sub(cost));
                    assert_eq!(
                        budget.bytes(charge),
                        next.is_some(),
                        "{allocation}/{reserved}/{charge}"
                    );
                    if let Some(next) = next {
                        remaining = next;
                    } else {
                        refused = true;
                    }
                    assert_eq!(budget.heap_units.get(), remaining / 8);
                    assert_eq!(budget.refused.get(), refused);
                }
                assert!(refused);
                let nodes = budget.nodes.get();
                assert!(!budget.enter(1, 0));
                assert_eq!(budget.nodes.get(), nodes);
                assert_eq!(budget.error(), Error::Limit);
            }
        }
    }
}
