//! Qualification of public packed fields against ordinary vectors and Candid.
use super::*;
#[cfg(not(feature = "bulk-baseline"))]
use ic_wire::Packed;

#[cfg(feature = "bulk-baseline")]
pub(super) type Field<T> = Vec<T>;
#[cfg(not(feature = "bulk-baseline"))]
pub(super) type Field<T> = Packed<T>;
pub(super) fn field<T>(values: Vec<T>) -> Field<T> {
    #[cfg(feature = "bulk-baseline")]
    {
        values
    }
    #[cfg(not(feature = "bulk-baseline"))]
    {
        Packed::new(values)
    }
}

fn paired<W, C>(name: String, candid: C, wire: impl FnOnce(&C) -> W) -> Case
where
    W: Serialize + DeserializeOwned + PartialEq + Debug + 'static,
    C: Serialize + DeserializeOwned + Compared + PartialEq + Debug + 'static,
{
    let input_json = serde_json::to_vec(&candid).unwrap();
    #[cfg(feature = "wire")]
    let wire = wire(&candid);
    #[cfg(not(feature = "wire"))]
    let _ = wire;
    let lanes = vec![
        #[cfg(feature = "wire")]
        lane(
            "wire-bounded",
            Rc::new(wire),
            |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
            |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
        ),
        #[cfg(feature = "candid")]
        lane(
            "candid",
            Rc::new(candid),
            |v| ic_platform::candid::contract::encode_one(v).unwrap(),
            |b| ic_platform::candid::decode_exactly_one(b, CANDID_LIMITS).ok(),
        ),
    ];
    Case {
        name,
        input_json,
        lanes,
    }
}

#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
struct WireManifest {
    object: u128,
    size: u64,
    content_type: String,
    hashes: Field<[u8; 32]>,
}
pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let mut out = Vec::new();
    let mut index = 0;
    macro_rules! push {
        ($value:expr) => {{
            if selected.is_none_or(|s| s == index) {
                out.push($value);
            }
            index += 1;
        }};
    }
    macro_rules! numeric {
        ($($ty:ty),*) => {$(
            for n in [0, 1, 128, 1024, 16384] {
                for wide in [false, true] {
                    push!({ let values: Vec<$ty> = (0..n).map(|i| if wide { <$ty>::MAX } else { (i % 64) as $ty }).collect();
                    paired(format!("{}-{}-{n}", stringify!($ty), if wide { "wide" } else { "small" }),
                        values, |values| field(values.clone())) });
                }
            }
        )*};
    }
    numeric!(i16, u16, i32, u32, i64, u64);
    for n in [0, 1, 32, 1024] {
        let hashes = || (0..n).map(|i| [i as u8; 32]).collect::<Vec<_>>();
        push!(paired(format!("hashes-{n}"), hashes(), |values| field(
            values.clone()
        )));
        push!(paired(
            format!("manifest-{n}"),
            Manifest {
                object: u128::MAX,
                size: n as u64 * 1_048_576,
                content_type: "application/octet-stream".into(),
                hashes: hashes(),
            },
            |value| WireManifest {
                object: value.object,
                size: value.size,
                content_type: value.content_type.clone(),
                hashes: field(value.hashes.clone()),
            }
        ));
    }
    assert!(selected.is_none_or(|s| s < index));
    out
}
