//! Fixed-width IDs against equally flattened Candid blobs, with explicit width checks.
use super::*;
type Id = [u8; 16];
#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
struct Filter<I> {
    ids: I,
    cursor: Option<Id>,
    include_archived: bool,
    limit: u16,
}
#[cfg(feature = "candid")]
#[derive(Debug, PartialEq, Serialize, serde::Deserialize, CandidType)]
#[candid_path("ic_platform::candid::contract")]
struct ExternalFilter {
    ids: Blob,
    cursor: Option<Blob>,
    include_archived: bool,
    limit: u16,
}
#[cfg(feature = "candid")]
fn decode_flat(bytes: &[u8]) -> Option<Blob> {
    let value: Blob = ic_platform::candid::decode_exactly_one(bytes, CANDID_LIMITS).ok()?;
    value.0.len().is_multiple_of(16).then_some(value)
}
#[cfg(feature = "candid")]
fn decode_filter(bytes: &[u8]) -> Option<ExternalFilter> {
    let value: ExternalFilter =
        ic_platform::candid::decode_exactly_one(bytes, CANDID_LIMITS).ok()?;
    (value.ids.0.len().is_multiple_of(16) && value.cursor.as_ref().is_none_or(|v| v.0.len() == 16))
        .then_some(value)
}
#[cfg(feature = "wire")]
mod wire {
    use super::*;
    #[cfg(feature = "id16-control")]
    type Ids = Vec<Id>;
    #[cfg(not(feature = "id16-control"))]
    type Ids = ic_wire::Packed<Id>;
    fn owned(ids: Vec<Id>) -> Ids {
        #[cfg(feature = "id16-control")]
        {
            ids
        }
        #[cfg(not(feature = "id16-control"))]
        {
            ic_wire::Packed::new(ids)
        }
    }
    fn raw(ids: Ids) -> Vec<Id> {
        #[cfg(feature = "id16-control")]
        {
            ids
        }
        #[cfg(not(feature = "id16-control"))]
        {
            ids.into_vec()
        }
    }
    fn filter(value: Filter<Vec<Id>>) -> Filter<Ids> {
        Filter {
            ids: owned(value.ids),
            cursor: value.cursor,
            include_archived: value.include_archived,
            limit: value.limit,
        }
    }
    pub(super) fn list(ids: Vec<Id>, copy: bool) -> Lane {
        if copy {
            lane(
                "wire-native",
                Rc::new(ids),
                |v| {
                    #[cfg(feature = "id16-control")]
                    {
                        ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap()
                    }
                    #[cfg(not(feature = "id16-control"))]
                    {
                        ic_wire::native::encode_request(METHOD, &owned(v.clone()), LIMITS).unwrap()
                    }
                },
                |b| {
                    ic_wire::native::decode_request::<Ids>(METHOD, b, LIMITS)
                        .ok()
                        .map(raw)
                },
            )
        } else {
            lane(
                "wire-native",
                Rc::new(owned(ids)),
                |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
            )
        }
    }
    pub(super) fn record(value: Filter<Vec<Id>>, copy: bool) -> Lane {
        if copy {
            lane(
                "wire-native",
                Rc::new(value),
                |v| {
                    #[cfg(feature = "id16-control")]
                    {
                        ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap()
                    }
                    #[cfg(not(feature = "id16-control"))]
                    {
                        ic_wire::native::encode_request(METHOD, &filter(v.clone()), LIMITS).unwrap()
                    }
                },
                |b| {
                    ic_wire::native::decode_request::<Filter<Ids>>(METHOD, b, LIMITS)
                        .ok()
                        .map(|v| Filter {
                            ids: raw(v.ids),
                            cursor: v.cursor,
                            include_archived: v.include_archived,
                            limit: v.limit,
                        })
                },
            )
        } else {
            lane(
                "wire-native",
                Rc::new(filter(value)),
                |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
            )
        }
    }
}
pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let mut out = Vec::new();
    let mut index = 0;
    for n in [0, 1, 8, 128, 4096] {
        for record in [false, true] {
            for copy in [false, true] {
                let i = index;
                index += 1;
                if selected.is_some_and(|s| s != i) {
                    continue;
                }
                let ids = (0..n)
                    .map(|i| {
                        core::array::from_fn(|j| {
                            ((i as u64).wrapping_mul(131).wrapping_add(j as u64 * 17) >> (j % 5))
                                as u8
                        })
                    })
                    .collect::<Vec<Id>>();
                let value = Filter {
                    ids: ids.clone(),
                    cursor: (n > 0).then_some([0x95; 16]),
                    include_archived: false,
                    limit: 128,
                };
                let input_json = if record {
                    serde_json::to_vec(&value).unwrap()
                } else {
                    serde_json::to_vec(&ids).unwrap()
                };
                let name = format!(
                    "id16-{}-{}-{n}",
                    if record { "filter" } else { "list" },
                    if copy { "from-borrowed" } else { "owned" }
                );
                let mut lanes = Vec::new();
                #[cfg(feature = "wire")]
                lanes.push(if record {
                    wire::record(value.clone(), copy)
                } else {
                    wire::list(ids.clone(), copy)
                });
                // Candid gets already-flat storage outside the counter, even for the
                // Wire copy control. Decode validates width but does not rebuild IDs.
                #[cfg(feature = "candid")]
                lanes.push(if record {
                    lane(
                        "candid",
                        Rc::new(ExternalFilter {
                            ids: Blob(ids.into_flattened()),
                            cursor: value.cursor.map(|v| Blob(v.to_vec())),
                            include_archived: false,
                            limit: 128,
                        }),
                        |v| ic_platform::candid::contract::encode_one(v).unwrap(),
                        decode_filter,
                    )
                } else {
                    lane(
                        "candid",
                        Rc::new(Blob(ids.into_flattened())),
                        |v| ic_platform::candid::contract::encode_one(v).unwrap(),
                        decode_flat,
                    )
                });
                out.push(Case {
                    name,
                    input_json,
                    lanes,
                });
            }
        }
    }
    assert!(selected.is_none_or(|s| s < index));
    out
}
#[cfg(all(test, feature = "candid"))]
mod tests;
