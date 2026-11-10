#[test]
fn mixed_change_tags_and_truncations_do_not_poison_next_decode() {
    use super::*;
    type Value = Change<ic_wire::ByteBuf>;
    for (value, bytes) in [
        (Value::Missing, vec![0]),
        (Value::Delete(128), vec![1, 128, 1]),
        (
            Value::Replace {
                revision: 128,
                payload: ic_wire::ByteBuf::from(vec![7, 9]),
            },
            vec![2, 128, 1, 2, 7, 9],
        ),
    ] {
        assert_eq!(ic_wire::native::encode(&value, LIMITS).unwrap(), bytes);
        for end in 0..bytes.len() {
            assert!(ic_wire::native::decode::<Value>(&bytes[..end], LIMITS).is_err());
        }
        for tag in [3, 127] {
            let mut invalid = bytes.clone();
            invalid[0] = tag;
            assert!(ic_wire::native::decode::<Value>(&invalid, LIMITS).is_err());
            assert_eq!(
                ic_wire::native::decode::<Value>(&bytes, LIMITS).unwrap(),
                value
            );
        }
    }
}
