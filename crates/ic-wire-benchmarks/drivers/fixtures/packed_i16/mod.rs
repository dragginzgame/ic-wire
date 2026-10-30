//! Explicit packed i16 fields compared with ordinary lists and Candid.
use super::*;

const MAX_ITEMS: usize = 16_384;

pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let mut out = Vec::new();
    let mut index = 0;
    for n in [0, 1, 128, 1024, MAX_ITEMS] {
        for distribution in ["small", "wide", "negative", "mixed"] {
            if selected.is_none_or(|s| s == index) {
                let mixed = [i16::MIN, -16385, -65, -64, -1, 0, 63, 64, 16384, i16::MAX];
                let values: Vec<i16> = (0..n)
                    .map(|i| match distribution {
                        "small" => (i % 64) as i16,
                        "wide" => i16::MAX,
                        "negative" => -1 - (i % 64) as i16,
                        _ => mixed[i % mixed.len()],
                    })
                    .collect();
                #[cfg_attr(
                    not(all(feature = "wire", not(feature = "packed-i16-baseline"))),
                    expect(unused_mut, reason = "Only the packed Wire control adds a lane.")
                )]
                let mut case = case(&format!("i16-{distribution}-{n}"), values.clone());
                #[cfg(all(feature = "wire", not(feature = "packed-i16-baseline")))]
                {
                    case.lanes.push(lane(
                        "wire-packed-i16",
                        Rc::new(ic_wire::PackedI16::<MAX_ITEMS>::new(values).unwrap()),
                        |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                        |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
                    ));
                }
                out.push(case);
            }
            index += 1;
        }
    }
    assert!(selected.is_none_or(|s| s < index), "unknown packed case");
    out
}
