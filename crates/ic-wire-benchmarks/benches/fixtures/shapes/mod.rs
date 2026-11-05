//! Real owned IC identities and individual contract shapes, not production modes.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

// Platform owns these application identities and intentionally does not implement
// CandidType for them. Only the comparison's external DTO uses Candid identities.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
pub(super) struct ExternalAccount {
    owner: ic_platform::candid::contract::Principal,
    subaccount: Option<[u8; 32]>,
}
pub(super) fn external(value: ic_platform_types::Account) -> ExternalAccount {
    ExternalAccount {
        owner: ic_platform::candid::contract::Principal::from_slice(value.owner.as_slice()),
        subaccount: value.subaccount,
    }
}
pub(super) fn account(i: usize, state: usize) -> ic_platform_types::Account {
    let mut raw = [0x95; 29];
    raw[..8].copy_from_slice(&(i as u64).to_le_bytes());
    ic_platform_types::Account {
        owner: ic_platform_types::Principal::from_slice(&raw),
        subaccount: match state {
            0 => None,
            1 => Some([0; 32]),
            _ => Some([0x95; 32]),
        },
    }
}

// Conversion and input identity are outside measurement. Wire measures actual
// Platform types; Candid measures the matching explicitly external DTOs.
fn identity_case<W>(
    name: &str,
    wire: W,
    #[cfg(feature = "candid")] candid: impl Serialize
    + DeserializeOwned
    + Compared
    + PartialEq
    + Debug
    + 'static,
) -> Case
where
    W: Serialize + DeserializeOwned + PartialEq + Debug + 'static,
{
    let input_json = serde_json::to_vec(&wire).unwrap();
    identity_case_with_input_json(
        name,
        wire,
        #[cfg(feature = "candid")]
        candid,
        input_json,
    )
}
fn identity_case_with_input_json<W>(
    name: &str,
    wire: W,
    #[cfg(feature = "candid")] candid: impl Serialize
    + DeserializeOwned
    + Compared
    + PartialEq
    + Debug
    + 'static,
    input_json: Vec<u8>,
) -> Case
where
    W: Serialize + DeserializeOwned + PartialEq + Debug + 'static,
{
    #[cfg(not(feature = "wire"))]
    let _ = wire;
    Case {
        name: name.into(),
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
                Rc::new(candid),
                |v| ic_platform::candid::contract::encode_one(v).unwrap(),
                |b| ic_platform::candid::decode_exactly_one(b, CANDID_LIMITS).ok(),
            ),
        ],
    }
}

macro_rules! record {
    ($module:ident, [$($field:ident,)*]) => {
        mod $module {
            use super::*;
            #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
            #[cfg_attr(feature = "candid", derive(CandidType))]
            #[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
            struct SparseRecord { $($field: Option<u64>,)* }
            pub(super) fn case(populated: usize) -> Case {
                let mut index = 0;
                let value = SparseRecord { $($field: {
                    let value = if index < populated { Some(u64::MAX - index as u64) } else { None };
                    index += 1;
                    value
                },)* };
                assert_eq!(index, 0 $(+ { let _ = stringify!($field); 1 })*);
                super::case(&format!("{}-{populated}", stringify!($module)), value)
            }
        }
    };
}
// Optional fields vary occupancy without introducing another serializer strategy.
record!(sparse8, [f000, f001, f002, f003, f004, f005, f006, f007,]);
record!(
    sparse32,
    [
        f000, f001, f002, f003, f004, f005, f006, f007, f008, f009, f010, f011, f012, f013, f014,
        f015, f016, f017, f018, f019, f020, f021, f022, f023, f024, f025, f026, f027, f028, f029,
        f030, f031,
    ]
);
record!(
    sparse128,
    [
        f000, f001, f002, f003, f004, f005, f006, f007, f008, f009, f010, f011, f012, f013, f014,
        f015, f016, f017, f018, f019, f020, f021, f022, f023, f024, f025, f026, f027, f028, f029,
        f030, f031, f032, f033, f034, f035, f036, f037, f038, f039, f040, f041, f042, f043, f044,
        f045, f046, f047, f048, f049, f050, f051, f052, f053, f054, f055, f056, f057, f058, f059,
        f060, f061, f062, f063, f064, f065, f066, f067, f068, f069, f070, f071, f072, f073, f074,
        f075, f076, f077, f078, f079, f080, f081, f082, f083, f084, f085, f086, f087, f088, f089,
        f090, f091, f092, f093, f094, f095, f096, f097, f098, f099, f100, f101, f102, f103, f104,
        f105, f106, f107, f108, f109, f110, f111, f112, f113, f114, f115, f116, f117, f118, f119,
        f120, f121, f122, f123, f124, f125, f126, f127,
    ]
);

macro_rules! wide_record {
    ($module:ident, [$($field:ident,)*]) => {
        mod $module {
            use super::*;
            #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
            #[cfg_attr(feature = "candid", derive(CandidType))]
            #[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
            struct WideRecord { $($field: u64,)* }
            pub(super) fn case() -> Case {
                super::case(stringify!($module), WideRecord { $($field: u64::MAX,)* })
            }
        }
    };
}
wide_record!(wide8, [f000, f001, f002, f003, f004, f005, f006, f007,]);
wide_record!(
    wide32,
    [
        f000, f001, f002, f003, f004, f005, f006, f007, f008, f009, f010, f011, f012, f013, f014,
        f015, f016, f017, f018, f019, f020, f021, f022, f023, f024, f025, f026, f027, f028, f029,
        f030, f031,
    ]
);
wide_record!(
    wide128,
    [
        f000, f001, f002, f003, f004, f005, f006, f007, f008, f009, f010, f011, f012, f013, f014,
        f015, f016, f017, f018, f019, f020, f021, f022, f023, f024, f025, f026, f027, f028, f029,
        f030, f031, f032, f033, f034, f035, f036, f037, f038, f039, f040, f041, f042, f043, f044,
        f045, f046, f047, f048, f049, f050, f051, f052, f053, f054, f055, f056, f057, f058, f059,
        f060, f061, f062, f063, f064, f065, f066, f067, f068, f069, f070, f071, f072, f073, f074,
        f075, f076, f077, f078, f079, f080, f081, f082, f083, f084, f085, f086, f087, f088, f089,
        f090, f091, f092, f093, f094, f095, f096, f097, f098, f099, f100, f101, f102, f103, f104,
        f105, f106, f107, f108, f109, f110, f111, f112, f113, f114, f115, f116, f117, f118, f119,
        f120, f121, f122, f123, f124, f125, f126, f127,
    ]
);

macro_rules! variants {
    ($module:ident, [$($variant:ident,)*]) => {
        mod $module {
            use super::*;
            #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
            #[cfg_attr(feature = "candid", derive(CandidType))]
            #[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
            pub(super) enum Value { $($variant(u64),)* }
            pub(super) fn case(variant: &str, value: Value) -> Case {
                super::case(&format!("{}-{variant}", stringify!($module)), value)
            }
        }
    };
}
variants!(enum8, [V000, V001, V002, V003, V004, V005, V006, V007,]);
variants!(
    enum32,
    [
        V000, V001, V002, V003, V004, V005, V006, V007, V008, V009, V010, V011, V012, V013, V014,
        V015, V016, V017, V018, V019, V020, V021, V022, V023, V024, V025, V026, V027, V028, V029,
        V030, V031,
    ]
);
variants!(
    enum129,
    [
        V000, V001, V002, V003, V004, V005, V006, V007, V008, V009, V010, V011, V012, V013, V014,
        V015, V016, V017, V018, V019, V020, V021, V022, V023, V024, V025, V026, V027, V028, V029,
        V030, V031, V032, V033, V034, V035, V036, V037, V038, V039, V040, V041, V042, V043, V044,
        V045, V046, V047, V048, V049, V050, V051, V052, V053, V054, V055, V056, V057, V058, V059,
        V060, V061, V062, V063, V064, V065, V066, V067, V068, V069, V070, V071, V072, V073, V074,
        V075, V076, V077, V078, V079, V080, V081, V082, V083, V084, V085, V086, V087, V088, V089,
        V090, V091, V092, V093, V094, V095, V096, V097, V098, V099, V100, V101, V102, V103, V104,
        V105, V106, V107, V108, V109, V110, V111, V112, V113, V114, V115, V116, V117, V118, V119,
        V120, V121, V122, V123, V124, V125, V126, V127, V128,
    ]
);

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
    for state in 0..3 {
        push!(identity_case(
            &format!("platform-account-{state}"),
            account(0, state),
            #[cfg(feature = "candid")]
            external(account(0, state))
        ));
        for n in [0, 1, 16, 128] {
            push!({
                let wire = (0..n).map(|i| account(i, state)).collect::<Vec<_>>();
                #[cfg(feature = "candid")]
                let candid = wire.iter().copied().map(external).collect::<Vec<_>>();
                identity_case(
                    &format!("platform-account-list-{state}-{n}"),
                    wire,
                    #[cfg(feature = "candid")]
                    candid,
                )
            });
        }
    }
    for n in [0, 1, 16, 128] {
        push!({
            let wire = (0..n)
                .map(|i| (account(i, 0).owner, Some(i as u64)))
                .collect::<BTreeMap<_, _>>();
            // JSON map-key limitations affect identity only, not either codec.
            let input_json = serde_json::to_vec(&wire.iter().collect::<Vec<_>>()).unwrap();
            #[cfg(feature = "candid")]
            let candid = wire
                .iter()
                .map(|(p, v)| {
                    (
                        ic_platform::candid::contract::Principal::from_slice(p.as_slice()),
                        *v,
                    )
                })
                .collect::<BTreeMap<ic_platform::candid::contract::Principal, _>>();
            identity_case_with_input_json(
                &format!("principal-map-{n}"),
                wire,
                #[cfg(feature = "candid")]
                candid,
                input_json,
            )
        });
        push!({
            let wire = (0..n).map(|i| account(i, i % 3)).collect::<BTreeSet<_>>();
            // Both decoders construct trees, with distinct owners so normalized
            // absent/zero equality cannot change cardinality or iteration order.
            #[cfg(feature = "candid")]
            let candid = wire.iter().copied().map(external).collect::<BTreeSet<_>>();
            identity_case(
                &format!("platform-account-set-{n}"),
                wire,
                #[cfg(feature = "candid")]
                candid,
            )
        });
    }
    push!(wide8::case());
    push!(wide32::case());
    push!(wide128::case());
    for populated in [0, 1, 8] {
        push!(sparse8::case(populated));
    }
    for populated in [0, 4, 32] {
        push!(sparse32::case(populated));
    }
    for populated in [0, 16, 128] {
        push!(sparse128::case(populated));
    }
    push!(enum8::case("v000", enum8::Value::V000(0)));
    push!(enum8::case("v007", enum8::Value::V007(u64::MAX)));
    push!(enum32::case("v000", enum32::Value::V000(0)));
    push!(enum32::case("v031", enum32::Value::V031(u64::MAX)));
    push!(enum129::case("v000", enum129::Value::V000(0)));
    push!(enum129::case("v127", enum129::Value::V127(u64::MAX)));
    push!(enum129::case("v128", enum129::Value::V128(u64::MAX)));
    for total in [0, 1, 4096, 65536] {
        for parts in [1, 16, 128] {
            // Equal total payload bytes, with increasingly many zero/tiny fields.
            push!(case(
                &format!("blob-batch-{total}-{parts}"),
                (0..parts)
                    .map(|i| Blob(data(total / parts + usize::from(i < total % parts))))
                    .collect::<Vec<_>>()
            ));
        }
        push!(case(
            &format!("ragged-blob-batch-{total}"),
            vec![
                vec![],
                vec![Blob(data(total / 2))],
                vec![],
                vec![Blob(data(total - total / 2)), Blob(vec![])]
            ]
        ));
    }
    assert!(selected.is_none_or(|s| s < index), "unknown shape case");
    out
}

#[cfg(all(test, feature = "wire", feature = "candid"))]
mod tests;
