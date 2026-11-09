//! Fixture-specific representation and matched-logical-input controls.

#[test]
fn account_comparison_preserves_representation_despite_normalized_equality() {
    use super::*;
    let mut frames = Vec::new();
    for state in 0..3 {
        let value = account(0, state);
        let reference = external(value);
        assert_eq!(reference.owner.as_slice(), value.owner.as_slice());
        assert_eq!(reference.subaccount, value.subaccount);
        let wire = ic_wire::native::encode_request(METHOD, &value, LIMITS).unwrap();
        let decoded: ic_platform_types::Account =
            ic_wire::native::decode_request(METHOD, &wire, LIMITS).unwrap();
        assert_eq!(decoded.owner.as_slice(), value.owner.as_slice());
        assert_eq!(decoded.subaccount, value.subaccount);
        let candid = candid::encode_one(&reference).unwrap();
        let decoded: ExternalAccount = crate::fixtures::decode_candid(&candid).unwrap();
        assert_eq!(decoded.owner.as_slice(), value.owner.as_slice());
        assert_eq!(decoded.subaccount, value.subaccount);
        for prefix in 0..wire.len() {
            assert!(
                ic_wire::native::decode_request::<ic_platform_types::Account>(
                    METHOD,
                    &wire[..prefix],
                    LIMITS
                )
                .is_err()
            );
        }
        frames.push((wire, candid));
    }
    assert_eq!(account(0, 0), account(0, 1));
    assert_ne!(frames[0].0, frames[1].0);
    assert_ne!(frames[0].1, frames[1].1);
}

#[test]
fn enum_cases_straddle_the_postcard_variant_index_boundary() {
    use super::*;
    let a = enum129::case("v127", enum129::Value::V127(u64::MAX));
    let b = enum129::case("v128", enum129::Value::V128(u64::MAX));
    let payload = ic_wire::native::REQUEST_HEADER_BYTES;
    assert_eq!(a.lanes[0].bytes[payload], 0x7f);
    assert_eq!(&b.lanes[0].bytes[payload..payload + 2], &[0x80, 0x01]);
    let mut invalid = b.lanes[0].bytes.clone();
    invalid[payload] = 0x81; // Variant 129 is not in this 129-variant enum.
    assert!(ic_wire::native::decode_request::<enum129::Value>(METHOD, &invalid, LIMITS).is_err());
}

#[test]
fn equal_total_blob_batches_keep_payload_content_and_tiny_controls() {
    use super::*;
    let cases = cases_selected(None);
    assert_eq!(cases.len(), 58);
    for total in [0, 1, 4096, 65536] {
        for parts in [1, 16, 128] {
            let case = cases
                .iter()
                .find(|case| case.name == format!("blob-batch-{total}-{parts}"))
                .unwrap();
            let input: Vec<Vec<u8>> = serde_json::from_slice(&case.input_json).unwrap();
            assert_eq!(input.len(), parts);
            assert_eq!(input.iter().map(Vec::len).sum::<usize>(), total);
            let wire: Vec<Blob> =
                ic_wire::native::decode_request(METHOD, &case.lanes[0].bytes, LIMITS).unwrap();
            let candid: Vec<Blob> = crate::fixtures::decode_candid(&case.lanes[1].bytes).unwrap();
            assert_eq!(wire, candid);
        }
    }
}
