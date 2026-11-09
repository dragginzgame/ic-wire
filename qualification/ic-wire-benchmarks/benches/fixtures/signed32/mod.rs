//! Zigzag boundaries and std containers; Candid stays an external comparator.
use super::*;
use std::collections::{BTreeSet, VecDeque};

const BOUNDARIES: [i32; 17] = [
    0,
    -1,
    1,
    -64,
    63,
    -65,
    64,
    -8192,
    8191,
    -8193,
    8192,
    -1_048_576,
    1_048_575,
    -1_048_577,
    1_048_576,
    i32::MIN,
    i32::MAX,
];

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
    for value in BOUNDARIES {
        push!(case(&format!("signed32-scalar-{value}"), value));
    }
    for n in [0, 1, 128, 4096] {
        for distribution in ["small", "negative", "wide", "mixed"] {
            push!({
                let values = (0..n)
                    .map(|i| match distribution {
                        "small" => (i % 64) as i32,
                        "negative" => -1 - (i % 64) as i32,
                        "wide" => i32::MAX,
                        _ => BOUNDARIES[i % BOUNDARIES.len()],
                    })
                    .collect::<Vec<_>>();
                case(&format!("signed32-list-{distribution}-{n}"), values)
            });
        }
        push!(case(
            &format!("signed32-deque-{n}"),
            (0..n)
                .map(|i| BOUNDARIES[i % BOUNDARIES.len()])
                .collect::<VecDeque<_>>(),
        ));
        // Unique elements: the name denotes actual set cardinality. Both codecs
        // reconstruct the same sorted set, outside any canonical-byte promise.
        push!(case(
            &format!("signed32-set-{n}"),
            (0..n)
                .map(|i| i as i32 - n as i32 / 2)
                .collect::<BTreeSet<_>>(),
        ));
        push!(case(
            &format!("signed32-tuple-{n}"),
            (
                i32::MIN,
                (n > 0).then_some(64_i32),
                Ok::<i32, String>(-65),
                (0..n)
                    .map(|i| BOUNDARIES[i % BOUNDARIES.len()])
                    .collect::<Vec<_>>(),
            ),
        ));
        push!({
            let values = vec![i32::MAX; n];
            Case {
                name: format!("signed32-packed-wide-{n}"),
                input_json: serde_json::to_vec(&values).unwrap(),
                lanes: vec![
                    #[cfg(feature = "wire")]
                    lane(
                        "wire-native",
                        Rc::new(ic_wire::Packed::new(values.clone())),
                        |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                        |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
                    ),
                    #[cfg(feature = "candid")]
                    lane(
                        "candid",
                        Rc::new(values),
                        |v| candid::encode_one(v).unwrap(),
                        |b| crate::fixtures::decode_candid(b).ok(),
                    ),
                ],
            }
        });
    }
    assert!(selected.is_none_or(|s| s < index), "unknown signed32 case");
    out
}

#[cfg(all(test, feature = "wire", feature = "candid"))]
mod tests;
