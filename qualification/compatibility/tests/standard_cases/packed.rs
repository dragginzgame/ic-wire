use super::{definition, limits, run_typescript};
use candidate::{Encoding, Packed, decode, encode};
use serde_json::json;

#[test]
fn packed_words_keep_byte_prefixes_and_owned_storage() {
    let mut cases = Vec::new();
    macro_rules! case {
        ($ty:ty,$word:literal,$values:expr,$bytes:expr) => {{
            let value=Packed::<$ty>::new($values);
            let bytes:Vec<u8>=$bytes;
            assert_eq!(encode(&value,Encoding::Payload,limits()).unwrap(),bytes);
            assert_eq!(decode::<Packed<$ty>>(&bytes,Encoding::Payload,limits()).unwrap(),value);
            let old=reference::Packed::<$ty>::new(value.as_slice().to_vec());
            let lim=limits();
            let old_lim=reference::Limits {bytes:lim.bytes,allocation:lim.allocation,nodes:lim.nodes,depth:lim.depth};
            assert_eq!(reference::native::encode(&old,old_lim).unwrap(),bytes);
            for (axis, maximum) in [(0,bytes.len()+1),(1,2048),(2,8),(3,4)] {
                for allowance in 0..=maximum {
                    let mut current=limits();
                    match axis {0=>current.bytes=allowance,1=>current.allocation=allowance,2=>current.nodes=allowance,_=>current.depth=allowance};
                    let prior=reference::Limits {bytes:current.bytes,allocation:current.allocation,nodes:current.nodes,depth:current.depth};
                    let candidate=encode(&value,Encoding::Payload,current).map_err(|e|format!("{e:?}"));
                    let reference=reference::native::encode(&old,prior).map_err(|e|format!("{e:?}"));
                    assert_eq!(candidate,reference,"encode {} axis {axis} allowance {allowance}",$word);
                    let candidate=decode::<Packed<$ty>>(&bytes,Encoding::Payload,current).map(|v|v.into_vec()).map_err(|e|format!("{e:?}"));
                    let reference=reference::native::decode::<reference::Packed<$ty>>(&bytes,prior).map(|v|v.into_vec()).map_err(|e|format!("{e:?}"));
                    assert_eq!(candidate,reference,"decode {} axis {axis} allowance {allowance}",$word);
                }
            }
            let schema=definition::<Packed<$ty>>("Words");
            assert_eq!(schema,json!({"k":"packed","word":$word}));
            cases.push(json!({"schema":schema,"bytes":bytes}));
        }};
    }
    case!(i8, "i8", vec![-128, 127], vec![2, 128, 127]);
    case!(i16, "i16", vec![-1, 128], vec![4, 255, 255, 128, 0]);
    case!(u16, "u16", vec![0, 65535], vec![4, 0, 0, 255, 255]);
    case!(
        i32,
        "i32",
        vec![-1, 128],
        vec![8, 255, 255, 255, 255, 128, 0, 0, 0]
    );
    case!(u32, "u32", vec![0x12345678], vec![4, 120, 86, 52, 18]);
    case!(
        i64,
        "i64",
        vec![-1],
        vec![8, 255, 255, 255, 255, 255, 255, 255, 255]
    );
    case!(
        u64,
        "u64",
        vec![0x0102030405060708],
        vec![8, 8, 7, 6, 5, 4, 3, 2, 1]
    );
    let mut hash16 = vec![16];
    hash16.extend([7; 16]);
    case!([u8; 16], "hash16", vec![[7; 16]], hash16);
    let mut hash32 = vec![32];
    hash32.extend([9; 32]);
    case!([u8; 32], "hash32", vec![[9; 32]], hash32);
    let output = run_typescript(&json!({"cases":cases}), include_str!("packed.mjs"));
    assert_eq!(output, json!(cases.len()));
    assert_eq!(
        decode::<Packed<u32>>(&[3, 0, 0, 0], Encoding::Payload, limits()),
        Err(candidate::Error::Malformed)
    );
    assert_eq!(
        decode::<Packed<[u8; 16]>>(&[1, 0], Encoding::Payload, limits()),
        Err(candidate::Error::Malformed)
    );
}
