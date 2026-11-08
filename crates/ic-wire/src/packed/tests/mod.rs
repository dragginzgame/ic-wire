#[test]
fn endian_append_matches_wire_bytes() {
    use super::sealed::Word;
    use alloc::vec::Vec;
    macro_rules! check {
        ($($ty:ty),*) => {$(
            let words = [<$ty>::MIN, 1, <$ty>::MAX];
            let mut bytes = Vec::new();
            for word in words { word.append_le(&mut bytes); }
            let expected: Vec<_> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
            assert_eq!(bytes, expected);
            for (chunk, word) in bytes.chunks_exact(size_of::<$ty>()).zip(words) {
                assert_eq!(<$ty>::read_le(chunk), word);
            }
        )*};
    }
    check!(i8, i16, u16, i32, u32, i64, u64);
    let id = core::array::from_fn::<_, 16, _>(|i| i as u8);
    let mut id_bytes = Vec::new();
    id.append_le(&mut id_bytes);
    assert_eq!(id_bytes, id);
    assert_eq!(<[u8; 16]>::read_le(&id_bytes), id);
    let hash = core::array::from_fn::<_, 32, _>(|i| i as u8);
    let mut bytes = Vec::new();
    hash.append_le(&mut bytes);
    assert_eq!(bytes, hash);
    assert_eq!(<[u8; 32]>::read_le(&bytes), hash);
}
