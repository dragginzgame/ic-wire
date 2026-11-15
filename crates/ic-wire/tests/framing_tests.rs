//! Literal vectors are independent of the candidate implementation.

use ic_wire::{Error, method_id, validate_request_header};

#[test]
fn method_ids_match_domain_separated_utf8_vectors() {
    // Independently calculated with arbitrary-precision integer arithmetic,
    // reducing modulo 2^64 after each FNV-1a step (see docs/design/framing.md).
    const PING: u64 = method_id("ping");
    assert_eq!(PING, 0xf3a6_2f2d_39b5_7bb6);
    for (name, expected) in [
        ("", 0x3504_91d1_e1e8_17fe),
        ("transfer", 0x3f6c_265a_c5d1_95d5),
        ("Ping", 0x1e33_3039_3b24_3c96),
        ("a\0b", 0xe1f6_2121_f9ef_e2af),
        ("é", 0x6da4_eebe_293f_cbca),
    ] {
        assert_eq!(method_id(name), expected, "{name:?}");
    }
}

#[test]
fn literal_request_headers_are_admitted() {
    for (method, header) in [
        (0, [0x49, 0x57, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        (
            u64::MAX,
            [0x49, 0x57, 255, 255, 255, 255, 255, 255, 255, 255, 0],
        ),
        (
            0x0123_4567_89ab_cdef,
            [
                0x49, 0x57, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, 0,
            ],
        ),
        (
            0xf3a6_2f2d_39b5_7bb6,
            [
                0x49, 0x57, 0xb6, 0x7b, 0xb5, 0x39, 0x2d, 0x2f, 0xa6, 0xf3, 0,
            ],
        ),
    ] {
        assert_eq!(validate_request_header(method, &header), Ok(()));
    }
}

#[test]
fn every_single_byte_header_mutation_is_an_identity_error() {
    let original = [
        0x49, 0x57, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, 0,
    ];
    for index in 0..original.len() {
        for byte in 0..=u8::MAX {
            if byte == original[index] {
                continue;
            }
            let mut damaged = original;
            damaged[index] = byte;
            assert_eq!(
                validate_request_header(0x0123_4567_89ab_cdef, &damaged),
                Err(Error::Identity),
                "offset {index}, byte {byte}"
            );
        }
    }
    assert_eq!(validate_request_header(0, &original), Err(Error::Identity));
}

#[test]
fn error_display_preserves_the_reference_contract() {
    for (error, text) in [
        (Error::Limit, "wire limit"),
        (Error::Allocation, "wire allocation"),
        (Error::Malformed, "wire malformed"),
        (Error::Trailing, "wire trailing"),
        (Error::Identity, "wire identity"),
    ] {
        assert_eq!(error.to_string(), text);
    }
}
