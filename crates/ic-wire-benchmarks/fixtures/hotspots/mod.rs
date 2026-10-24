//! Ordinary-list hot spots, with empty/singleton and byte-oriented controls.
use super::*;

#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct MixedBlob<T> {
    id: u64,
    payload: Blob,
    samples: T,
    labels: Vec<String>,
    cursor: Option<String>,
}

pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let mut out = Vec::new();
    let mut index = 0;
    macro_rules! push {
        ($case:expr) => {{
            if selected.is_none_or(|s| s == index) {
                out.push($case);
            }
            index += 1;
        }};
    }
    for n in [0, 1, 128, 16384] {
        for distribution in ["small", "wide", "negative", "mixed"] {
            push!({
                let choices = [i16::MIN, -65, -1, 0, 64, i16::MAX];
                let values = (0..n)
                    .map(|i| match distribution {
                        "small" => (i % 64) as i16,
                        "wide" => i16::MAX,
                        "negative" => -1 - (i % 64) as i16,
                        _ => choices[i % choices.len()],
                    })
                    .collect::<Vec<_>>();
                case(&format!("i16-{distribution}-{n}"), values)
            });
        }
        macro_rules! wide {
            ($($ty:ty),*) => {$(push!(case(&format!("{}-wide-{n}", stringify!($ty)), vec![<$ty>::MAX; n]));)*};
        }
        wide!(u16, i32, u32, i64, u64, i8);
    }
    for n in [
        0, 1, 7, 8, 9, 63, 64, 65, 127, 128, 129, 191, 192, 193, 255, 256, 257, 4095, 4096, 4097,
        1_048_576,
    ] {
        push!(case(&format!("generic-bytes-{n}"), data(n)));
        push!(case(&format!("blob-{n}"), Blob(data(n))));
    }
    for n in [0, 1, 64, 4096, 1_048_576] {
        push!({
            let value = MixedBlob {
                id: 16_384,
                payload: Blob(data(n)),
                samples: vec![i16::MIN; n.min(4096)],
                labels: vec!["blob".into(), "application/octet-stream".into()],
                cursor: (n != 0).then(|| "next-page".into()),
            };
            let input_json = serde_json::to_vec(&value).unwrap();
            #[cfg(feature = "wire")]
            let wire = MixedBlob {
                id: value.id,
                payload: Blob(value.payload.0.clone()),
                samples: ic_wire::Packed::new(value.samples.clone()),
                labels: value.labels.clone(),
                cursor: value.cursor.clone(),
            };
            Case {
                name: format!("mixed-blob-{n}"),
                input_json,
                lanes: vec![
                    #[cfg(feature = "wire")]
                    lane(
                        "wire-native",
                        Rc::new(wire),
                        |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                        |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
                    ),
                    #[cfg(feature = "candid")]
                    lane(
                        "candid",
                        Rc::new(value),
                        |v| ic_platform::candid::contract::encode_one(v).unwrap(),
                        |b| ic_platform::candid::decode_exactly_one(b, CANDID_LIMITS).ok(),
                    ),
                ],
            }
        });
    }
    assert!(selected.is_none_or(|s| s < index), "unknown hotspot case");
    out
}
