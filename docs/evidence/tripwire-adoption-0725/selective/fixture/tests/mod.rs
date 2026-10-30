use super::*;

#[cfg(feature = "ic")]
mod physical;

fn view(input: &[u8]) -> Result<Window, Error> {
    locate(input.len(), |offset, output| {
        output.copy_from_slice(&input[offset..offset + output.len()]);
    })
}
fn parity(input: &[u8]) {
    let decoded = native::decode_request::<Borrowed<'_>>(ID, input, LIMITS);
    let owned = native::decode_request::<Owned>(ID, input, LIMITS);
    assert_eq!(decoded.is_ok(), owned.is_ok());
    let located = view(input);
    assert_eq!(
        decoded.is_ok(),
        located.is_ok(),
        "acceptance differs: {} bytes",
        input.len()
    );
    match (decoded, located) {
        (Ok(value), Ok(window)) => {
            let owned = owned.unwrap();
            assert_eq!((owned.mode, owned.nonce), (value.mode, value.nonce));
            assert_eq!(owned.body.as_slice(), value.body.as_slice());
            assert_eq!(window.mode, value.mode);
            assert_eq!(window.nonce, value.nonce);
            assert_eq!(&input[window.start..], value.body.as_slice());
        }
        (Err(expected), Err(actual)) => assert_eq!(expected, actual),
        _ => unreachable!(),
    }
}

#[test]
fn known_shape_matches_full_decoder_and_reads_only_selected_bytes() {
    for size in [0, 1, 127, 128, 16_383, 16_384, 32_768, MAX] {
        for nonce in [0, 127, 128, 16_383, 16_384, u32::MAX as u64, u64::MAX] {
            for mode in 0..3 {
                let input = packet(size, mode, nonce);
                parity(&input);
                let value: Borrowed<'_> = native::decode_request(ID, &input, LIMITS).unwrap();
                let copied = std::cell::Cell::new(0);
                let copy = |offset, output: &mut [u8]| {
                    copied.set(copied.get() + output.len());
                    output.copy_from_slice(&input[offset..offset + output.len()]);
                };
                let window = locate(input.len(), copy).unwrap();
                assert_eq!(
                    selected(&window, copy),
                    consume(mode, nonce, value.body.as_slice())
                );
                let selected = match mode {
                    0 => size,
                    1 => size.min(256),
                    _ => 0,
                };
                assert_eq!(copied.get(), input.len().min(27) + selected);
                assert_eq!(
                    direct_owned(&window, copy),
                    consume(mode, nonce, value.body.as_slice())
                );
            }
        }
    }
}

#[test]
fn malformed_frames_match_full_decoder_before_business_access() {
    let input = packet(128, 2, 0);
    for end in 0..input.len() {
        parity(&input[..end]);
    }
    for at in 0..input.len() {
        for byte in [0, 1, 127, 128, 255] {
            let mut mutated = input.clone();
            mutated[at] = byte;
            parity(&mutated);
        }
    }
    let mut trailing = input.clone();
    trailing.push(0);
    parity(&trailing);
    let mut redundant_nonce = input.clone();
    redundant_nonce[12] = 0x80;
    redundant_nonce.insert(13, 0);
    parity(&redundant_nonce);
    let mut redundant_length = packet(0, 2, 0);
    redundant_length[13] = 0x80;
    redundant_length.push(0);
    parity(&redundant_length);
    // Field-bound failure differs from the outer frame-size allowance.
    let too_long = native::encode_request(
        ID,
        &(2u8, 0u64, ic_wire::ByteBuf::from(vec![0; MAX + 1])),
        LIMITS,
    )
    .unwrap();
    parity(&too_long);
    parity(&vec![0; LIMITS.bytes + 1]);
    // Deterministic arbitrary inputs and valid-prefix corruptions.
    let mut state = 0x9823_4913u64;
    for i in 0..10_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let mut bytes = if i % 2 == 0 {
            input.clone()
        } else {
            vec![0; i % 64]
        };
        if !bytes.is_empty() {
            let at = state as usize % bytes.len();
            bytes[at] = (state >> 32) as u8;
        }
        parity(&bytes);
    }
}
