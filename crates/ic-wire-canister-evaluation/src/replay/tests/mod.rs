use super::*;

#[test]
fn rejects_bad_wire_without_mutation() {
    let caller = Principal::anonymous();
    let bytes = postcard::to_allocvec(&(
        CONTRACT,
        0_u8,
        caller.as_slice(),
        caller.as_slice(),
        100_u64,
        1_u64,
        7_u64,
    ))
    .unwrap();
    let mut state = [0; STATE_BYTES];
    for end in 0..bytes.len() {
        assert!(execute(&mut state, &bytes[..end], caller, caller, 0).is_err());
        assert_eq!(state, [0; STATE_BYTES]);
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    let mut overlong = bytes.clone();
    // Replace the final canonical 7 with a nonminimal varint for 7.
    *overlong.last_mut().unwrap() = 0x87;
    overlong.push(0);
    for invalid in [trailing, overlong, vec![0; MAX_REQUEST + 1]] {
        assert!(execute(&mut state, &invalid, caller, caller, 0).is_err());
        assert_eq!(state, [0; STATE_BYTES]);
    }
}

#[test]
fn scopes_receipts_to_caller_and_refuses_full_table() {
    let audience = Principal::anonymous();
    let mut state = [0; STATE_BYTES];
    for i in 0..CAPACITY {
        let caller = Principal::from_slice(&[i as u8]);
        let bytes = postcard::to_allocvec(&(
            CONTRACT,
            0_u8,
            caller.as_slice(),
            audience.as_slice(),
            100_u64,
            1_u64,
            1_u64,
        ))
        .unwrap();
        let reply = execute(&mut state, &bytes, caller, audience, 0).unwrap();
        assert_eq!(
            execute(&mut state, &bytes, caller, audience, 0).unwrap(),
            reply
        );
    }
    assert_eq!(u64::from_le_bytes(state[..8].try_into().unwrap()), 8);
    let before = state;
    let bytes = postcard::to_allocvec(&(
        CONTRACT,
        0_u8,
        audience.as_slice(),
        audience.as_slice(),
        100_u64,
        2_u64,
        1_u64,
    ))
    .unwrap();
    assert_eq!(
        execute(&mut state, &bytes, audience, audience, 0),
        Err(CommandError::Capacity)
    );
    assert_eq!(state, before);
}
