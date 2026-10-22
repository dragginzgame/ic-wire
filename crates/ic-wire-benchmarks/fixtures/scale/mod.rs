//! Nominally distinct records: six fields, sixteen byte-array widths, unique field labels.
//! Each enabled type has a runtime-selectable codec path. No native Wire lane is linked.
use super::*;
macro_rules! record {
    ($module:ident, $field:ident, $width:literal) => {
        mod $module {
            use super::*;
            #[cfg(feature = "wire")]
            #[derive(Debug, PartialEq, ic_wire::Wire)]
            struct WireRecord {
                #[wire(id = 1)] $field: u64,
                #[wire(id = 2)] active: bool,
                #[wire(id = 3)] delta: i64,
                #[wire(id = 4)] optional: Option<u64>,
                #[wire(id = 5)] bytes: [u8; $width],
                #[wire(id = 6)] label: ic_wire::Text<64>,
            }
            #[cfg(feature = "candid")]
            #[derive(Debug, PartialEq, serde::Deserialize, Serialize, CandidType)]
            #[candid_path("ic_platform::candid::contract")]
            struct CandidRecord {
                $field: u64,
                active: bool,
                delta: i64,
                optional: Option<u64>,
                bytes: [u8; $width],
                label: String,
            }
            pub(super) fn build() -> Case {
                let mut lanes = Vec::new();
                let input_json = serde_json::to_vec(&serde_json::json!({
                    stringify!($field): 1_000_000_u64, "active": true, "delta": -65,
                    "optional": 128, "bytes": vec![42_u8; $width], "label": "Representative record"
                })).unwrap();
                #[cfg(feature = "wire")]
                lanes.push(lane("wire-finite", Rc::new(WireRecord {
                    $field: 1_000_000, active: true, delta: -65, optional: Some(128),
                    bytes: [42; $width], label: ic_wire::Text::new("Representative record".into()).unwrap(),
                }), |v| ic_wire::encode_request::<Binding<WireRecord>>(v).unwrap().into_bytes(),
                    |b| ic_wire::decode_request::<Binding<WireRecord>>(b).ok()));
                #[cfg(feature = "candid")]
                lanes.push(lane("candid", Rc::new(CandidRecord {
                    $field: 1_000_000, active: true, delta: -65, optional: Some(128),
                    bytes: [42; $width], label: "Representative record".into(),
                }), |v| ic_platform::candid::contract::encode_one(v).unwrap(),
                    |b| ic_platform::candid::decode_exactly_one(b, CANDID_LIMITS).ok()));
                Case { name: stringify!($module).into(), input_json, lanes }
            }
        }
    };
}
// One roster owns both derive expansion and runtime reachability.
macro_rules! cohort {
    (first: [$($a:ident, $af:ident, $an:literal;)*]
     ten: [$($b:ident, $bf:ident, $bn:literal;)*]
     rest: [$($c:ident, $cf:ident, $cn:literal;)*]) => {
        $(record!($a, $af, $an);)*
        $(#[cfg(not(feature = "derive-scale-1"))] record!($b, $bf, $bn);)*
        $(#[cfg(not(any(feature = "derive-scale-1", feature = "derive-scale-10")))] record!($c, $cf, $cn);)*
        pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
            let builders: &[fn() -> Case] = &[
                $($a::build,)*
                $(#[cfg(not(feature = "derive-scale-1"))] $b::build,)*
                $(#[cfg(not(any(feature = "derive-scale-1", feature = "derive-scale-10")))] $c::build,)*
            ];
            if let Some(index) = selected { vec![builders[index]()] } else { builders.iter().map(|build| build()).collect() }
        }
    };
}
cohort! {
first: [
    record_000, amount_000, 1;
]
ten: [
    record_001, amount_001, 2;
    record_002, amount_002, 3;
    record_003, amount_003, 4;
    record_004, amount_004, 5;
    record_005, amount_005, 6;
    record_006, amount_006, 7;
    record_007, amount_007, 8;
    record_008, amount_008, 9;
    record_009, amount_009, 10;
]
rest: [
    record_010, amount_010, 11;
    record_011, amount_011, 12;
    record_012, amount_012, 13;
    record_013, amount_013, 14;
    record_014, amount_014, 15;
    record_015, amount_015, 16;
    record_016, amount_016, 1;
    record_017, amount_017, 2;
    record_018, amount_018, 3;
    record_019, amount_019, 4;
    record_020, amount_020, 5;
    record_021, amount_021, 6;
    record_022, amount_022, 7;
    record_023, amount_023, 8;
    record_024, amount_024, 9;
    record_025, amount_025, 10;
    record_026, amount_026, 11;
    record_027, amount_027, 12;
    record_028, amount_028, 13;
    record_029, amount_029, 14;
    record_030, amount_030, 15;
    record_031, amount_031, 16;
    record_032, amount_032, 1;
    record_033, amount_033, 2;
    record_034, amount_034, 3;
    record_035, amount_035, 4;
    record_036, amount_036, 5;
    record_037, amount_037, 6;
    record_038, amount_038, 7;
    record_039, amount_039, 8;
    record_040, amount_040, 9;
    record_041, amount_041, 10;
    record_042, amount_042, 11;
    record_043, amount_043, 12;
    record_044, amount_044, 13;
    record_045, amount_045, 14;
    record_046, amount_046, 15;
    record_047, amount_047, 16;
    record_048, amount_048, 1;
    record_049, amount_049, 2;
    record_050, amount_050, 3;
    record_051, amount_051, 4;
    record_052, amount_052, 5;
    record_053, amount_053, 6;
    record_054, amount_054, 7;
    record_055, amount_055, 8;
    record_056, amount_056, 9;
    record_057, amount_057, 10;
    record_058, amount_058, 11;
    record_059, amount_059, 12;
    record_060, amount_060, 13;
    record_061, amount_061, 14;
    record_062, amount_062, 15;
    record_063, amount_063, 16;
    record_064, amount_064, 1;
    record_065, amount_065, 2;
    record_066, amount_066, 3;
    record_067, amount_067, 4;
    record_068, amount_068, 5;
    record_069, amount_069, 6;
    record_070, amount_070, 7;
    record_071, amount_071, 8;
    record_072, amount_072, 9;
    record_073, amount_073, 10;
    record_074, amount_074, 11;
    record_075, amount_075, 12;
    record_076, amount_076, 13;
    record_077, amount_077, 14;
    record_078, amount_078, 15;
    record_079, amount_079, 16;
    record_080, amount_080, 1;
    record_081, amount_081, 2;
    record_082, amount_082, 3;
    record_083, amount_083, 4;
    record_084, amount_084, 5;
    record_085, amount_085, 6;
    record_086, amount_086, 7;
    record_087, amount_087, 8;
    record_088, amount_088, 9;
    record_089, amount_089, 10;
    record_090, amount_090, 11;
    record_091, amount_091, 12;
    record_092, amount_092, 13;
    record_093, amount_093, 14;
    record_094, amount_094, 15;
    record_095, amount_095, 16;
    record_096, amount_096, 1;
    record_097, amount_097, 2;
    record_098, amount_098, 3;
    record_099, amount_099, 4;
]
}
