//! Matched binary-key maps; only the Wire key representation changes.
use super::{Blob, Case, LIMITS, METHOD, Rc, data, lane};

pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let sizes = [0, 1, 16, 128];
    assert!(
        selected.is_none_or(|index| index < sizes.len()),
        "unknown byte-key case"
    );
    sizes
        .into_iter()
        .enumerate()
        .filter_map(|(index, n)| {
            if selected.is_some_and(|s| s != index) {
                return None;
            }
            let value = (0..n)
                .map(|i| {
                    let mut key = [0x95_u8; 32];
                    key[0] = i as u8;
                    (key, (i % 2 == 0).then(|| Blob(data(32))))
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            // Input identity is the same ordered-pair JSON for both representations;
            // the measured codecs serialize actual BTreeMaps, not these JSON pairs.
            let input_json = serde_json::to_vec(&value.iter().collect::<Vec<_>>()).unwrap();
            #[cfg(all(feature = "wire", not(feature = "byte-keys-control")))]
            let wire = value
                .iter()
                .map(|(key, value)| {
                    (
                        ic_wire::ByteArray::new(*key),
                        value.as_ref().map(|v| Blob(v.0.clone())),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            #[cfg(all(feature = "wire", feature = "byte-keys-control"))]
            let wire = value
                .iter()
                .map(|(key, value)| (*key, value.as_ref().map(|v| Blob(v.0.clone()))))
                .collect::<std::collections::BTreeMap<_, _>>();
            Some(Case {
                name: format!("binary-key-map-{n}"),
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
                        |v| candid::encode_one(v).unwrap(),
                        |b| crate::fixtures::decode_candid(b).ok(),
                    ),
                ],
            })
        })
        .collect()
}
