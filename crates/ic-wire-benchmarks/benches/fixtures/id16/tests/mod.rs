#[test]
fn flat_candid_control_checks_id_and_cursor_widths() {
    use super::*;
    for n in [0, 1, 15, 16, 17, 32] {
        let bytes = ic_platform::candid::contract::encode_one(Blob(vec![0; n])).unwrap();
        assert_eq!(decode_flat(&bytes).is_some(), n % 16 == 0);
    }
    let value = ExternalFilter {
        ids: Blob(vec![0; 16]),
        cursor: Some(Blob(vec![0; 15])),
        include_archived: false,
        limit: 1,
    };
    assert!(decode_filter(&ic_platform::candid::contract::encode_one(value).unwrap()).is_none());
}
