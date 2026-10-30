use super::*;

#[test]
fn automatic_method_identity_vectors_and_frame() {
    // Frozen vectors independently calculated from the documented UTF-8 domain.
    const ECHO: u64 = method_id("echo");
    assert_eq!(ECHO, 3_346_031_142_226_386_015);
    assert_eq!(method_id("match"), 6_850_280_557_138_878_871);
    assert_eq!(method_id("café"), 15_715_696_330_864_016_268);
    assert_eq!(method_id("stable-name"), 11_911_248_233_000_959_399);
    assert_ne!(method_id("Echo"), ECHO);
    let limits = Limits {
        bytes: 128,
        allocation: 8192,
        nodes: 32,
        depth: 8,
    };
    let bytes = encode_request(ECHO, &42u8, limits).unwrap();
    assert_eq!(
        bytes,
        [
            0x49, 0x57, 0x5f, 0x5c, 0xcc, 0x87, 0x98, 0x7d, 0x6f, 0x2e, 0, 42
        ]
    );
    assert_eq!(
        decode_request::<u8>(7, &bytes, limits),
        Err(Error::Identity)
    );
    assert_eq!(decode_request::<u8>(ECHO, &bytes, limits), Ok(42));
}

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
