//! Selected distributions from the pinned legacy hotspot/bulk studies.
//! Destination: one native runner, no copied legacy codec or orchestration.
use super::{Options, pair};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
    num::NonZeroI64,
};

pub const CASES: &[&str] = &[
    "i128-boundaries",
    "u128-boundaries",
    "i8-boundaries",
    "u16-boundaries",
    "u32-boundaries",
    "i64-boundaries",
    "i32-small",
    "i32-negative",
    "u32-small",
    "u64-small",
    "id16",
    "text-ascii",
    "text-unicode",
    "text-escaped",
    "options",
    "results",
    "tuples",
    "map-integer",
    "map-text",
    "map-byte-key",
    "set",
    "deque",
    "linked-list",
    "record-list",
    "principals",
    "accounts",
    "unit-list",
    "nested-list",
    "finite-f32-normal",
    "finite-f32-subnormal",
    "finite-f64-normal",
    "finite-f64-subnormal",
    "bools-true",
    "bools-false",
    "heap-ascending",
    "heap-descending",
    "i16-small",
    "i16-wide",
    "i16-negative",
    "i16-mixed",
    "i32-boundaries",
    "u64-boundaries",
    "u64-wide",
    "id32",
    "bytes",
    "mixed",
    "nested",
    "bools",
    "finite-f32",
    "finite-f64",
    "nonzero-i64",
    "heap-u64",
];

pub fn supports_bulk(name: &str) -> bool {
    !matches!(
        name,
        "i128-boundaries"
            | "u128-boundaries"
            | "text-ascii"
            | "text-unicode"
            | "text-escaped"
            | "options"
            | "results"
            | "tuples"
            | "map-integer"
            | "map-text"
            | "map-byte-key"
            | "set"
            | "deque"
            | "linked-list"
            | "record-list"
            | "principals"
            | "accounts"
            | "unit-list"
            | "nested-list"
    )
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Mixed<T> {
    id: u64,
    payload: serde_bytes::ByteBuf,
    samples: T,
    labels: Vec<String>,
    cursor: Option<String>,
}
fn mixed<T>(size: usize, samples: T) -> Mixed<T> {
    Mixed {
        id: 16384,
        payload: serde_bytes::ByteBuf::from(vec![0xa5; size]),
        samples,
        labels: vec!["blob".into(), "application/octet-stream".into()],
        cursor: Some("next-page".into()),
    }
}

// A small application-shaped request exercises records, enum variants, optional
// fields and strings together. Size controls samples, not the number of records.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Change<T> {
    Remove {
        id: u64,
    },
    Put {
        id: u64,
        revision: Option<u32>,
        samples: T,
    },
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Request<T> {
    cursor: Option<String>,
    changes: Vec<Change<T>>,
}
fn nested<T>(samples: T) -> Request<T> {
    Request {
        cursor: Some("page-128".into()),
        changes: vec![
            Change::Remove { id: 127 },
            Change::Put {
                id: 128,
                revision: Some(16384),
                samples,
            },
        ],
    }
}
macro_rules! domain {
    ($name:ident, $ty:ty, $adapter:literal) => {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct $name(#[serde(with = $adapter)] $ty);
    };
}
domain!(NewBools, Vec<bool>, "candidate::serde_packed::bools");
domain!(OldBools, Vec<bool>, "reference::serde_packed::bools");
domain!(NewF32, Vec<f32>, "candidate::serde_packed::finite_f32");
domain!(OldF32, Vec<f32>, "reference::serde_packed::finite_f32");
domain!(NewF64, Vec<f64>, "candidate::serde_packed::finite_f64");
domain!(OldF64, Vec<f64>, "reference::serde_packed::finite_f64");
domain!(
    NewNonzero,
    Vec<NonZeroI64>,
    "candidate::serde_packed::nonzero_i64"
);
domain!(
    OldNonzero,
    Vec<NonZeroI64>,
    "reference::serde_packed::nonzero_i64"
);
macro_rules! heap {
    ($name:ident, $adapter:literal) => {
        #[derive(Debug, Serialize, Deserialize)]
        struct $name(#[serde(with = $adapter)] BinaryHeap<u64>);
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0.clone().into_sorted_vec() == other.0.clone().into_sorted_vec()
            }
        }
    };
}
heap!(NewHeap, "candidate::serde_packed::heap_u64");
heap!(OldHeap, "reference::serde_packed::heap_u64");
#[derive(Debug, Serialize, Deserialize)]
struct OrdinaryHeap(BinaryHeap<u64>);
impl PartialEq for OrdinaryHeap {
    fn eq(&self, other: &Self) -> bool {
        self.0.clone().into_sorted_vec() == other.0.clone().into_sorted_vec()
    }
}

pub fn run(options: &Options, name: &str, size: usize, bulk: bool) {
    macro_rules! compare {
        ($new:expr, $old:expr $(,)?) => {
            pair(options, name, size, bulk, &$new, &$old)
        };
    }
    macro_rules! words {
        ($value:expr) => {{
            let values = $value;
            if bulk {
                compare!(
                    candidate::Packed::<_>::new(values.clone()).unwrap(),
                    reference::Packed::new(values),
                );
            } else {
                compare!(values.clone(), values);
            }
        }};
    }
    macro_rules! adapter {
        ($value:expr, $new:ident, $old:ident) => {{
            let values = $value;
            if bulk {
                compare!($new(values.clone()), $old(values),);
            } else {
                compare!(values.clone(), values);
            }
        }};
    }
    macro_rules! ordinary {
        ($value:expr) => {{
            let value = $value;
            compare!(value, value);
        }};
    }
    match name {
        "i128-boundaries" => ordinary!(
            (0..size)
                .map(|i| [i128::MIN, -65, -64, -1, 0, 63, 64, i128::MAX][i % 8])
                .collect::<Vec<_>>()
        ),
        "u128-boundaries" => ordinary!(
            (0..size)
                .map(|i| [0_u128, 127, 128, u64::MAX as u128, 1_u128 << 64, u128::MAX][i % 6])
                .collect::<Vec<_>>()
        ),
        "i8-boundaries" => words!(
            (0..size)
                .map(|i| [i8::MIN, -1, 0, i8::MAX][i % 4])
                .collect::<Vec<_>>()
        ),
        "u16-boundaries" => words!(
            (0..size)
                .map(|i| [0_u16, 127, 128, 16383, 16384, u16::MAX][i % 6])
                .collect::<Vec<_>>()
        ),
        "u32-boundaries" => words!(
            (0..size)
                .map(|i| [0_u32, 127, 128, 16383, 16384, 1 << 21, 1 << 28, u32::MAX][i % 8])
                .collect::<Vec<_>>()
        ),
        "i64-boundaries" => words!(
            (0..size)
                .map(|i| [i64::MIN, -8193, -65, -64, -1, 0, 63, 64, 8192, i64::MAX][i % 10])
                .collect::<Vec<_>>()
        ),
        "i32-small" => words!((0..size).map(|i| (i % 64) as i32).collect::<Vec<_>>()),
        "i32-negative" => words!((0..size).map(|i| -1 - (i % 64) as i32).collect::<Vec<_>>()),
        "u32-small" => words!((0..size).map(|i| (i % 128) as u32).collect::<Vec<_>>()),
        "u64-small" => words!((0..size).map(|i| (i % 128) as u64).collect::<Vec<_>>()),
        "id16" => words!(vec![[0x5a_u8; 16]; size]),
        "text-ascii" => ordinary!("a".repeat(size)),
        "text-unicode" => ordinary!("é🌍".repeat(size)),
        "text-escaped" => ordinary!("\"\\\n\0".repeat(size)),
        "options" => ordinary!(
            (0..size)
                .map(|i| if i % 3 == 0 { None } else { Some(i as u64) })
                .collect::<Vec<_>>()
        ),
        "results" => ordinary!(
            (0..size)
                .map(|i| if i % 2 == 0 {
                    Ok(i as u32)
                } else {
                    Err(String::from("failure"))
                })
                .collect::<Vec<Result<u32, String>>>()
        ),
        "tuples" => ordinary!(
            (0..size)
                .map(|i| (i as u64, -(i as i32), i % 2 == 0, [7_u8; 4]))
                .collect::<Vec<_>>()
        ),
        "map-integer" => ordinary!(
            (0..size)
                .map(|i| (i as u64, -(i as i64)))
                .collect::<BTreeMap<_, _>>()
        ),
        "map-text" => ordinary!(
            (0..size)
                .map(|i| (format!("key-{i}"), format!("value-{i}")))
                .collect::<BTreeMap<_, _>>()
        ),
        "map-byte-key" => ordinary!(
            (0..size)
                .map(|i| {
                    let mut key = [0_u8; 32];
                    key[..8].copy_from_slice(&(i as u64).to_le_bytes());
                    (key, i as u64)
                })
                .collect::<BTreeMap<_, _>>()
        ),
        "set" => ordinary!((0..size).map(|i| i as u64).collect::<BTreeSet<_>>()),
        "deque" => ordinary!((0..size).map(|i| i as u64).collect::<VecDeque<_>>()),
        "linked-list" => ordinary!((0..size).map(|i| i as u64).collect::<LinkedList<_>>()),
        "record-list" => ordinary!(
            (0..size)
                .map(|i| mixed(8, vec![i as i16; 4]))
                .collect::<Vec<_>>()
        ),
        "principals" => ordinary!(
            (0..size)
                .map(
                    |i| ic_platform_types::Principal::try_from_slice(&vec![7; [0, 1, 29][i % 3]])
                        .unwrap()
                )
                .collect::<Vec<_>>()
        ),
        "accounts" => ordinary!(
            (0..size)
                .map(|i| ic_platform_types::Account {
                    owner: ic_platform_types::Principal::anonymous(),
                    subaccount: [None, Some([0; 32]), Some([7; 32])][i % 3]
                })
                .collect::<Vec<_>>()
        ),
        "unit-list" => ordinary!(vec![(); size]),
        "nested-list" => ordinary!((0..size).map(|i| vec![i as u32; i % 8]).collect::<Vec<_>>()),
        "i16-small" | "i16-wide" | "i16-negative" | "i16-mixed" => {
            let values: Vec<i16> = (0..size)
                .map(|i| match name {
                    "i16-small" => (i % 64) as i16,
                    "i16-wide" => i16::MAX,
                    "i16-negative" => -1 - (i % 64) as i16,
                    _ => [i16::MIN, -65, -1, 0, 64, i16::MAX][i % 6],
                })
                .collect();
            words!(values);
        }
        "u64-wide" => words!(vec![u64::MAX; size]),
        "i32-boundaries" => words!(
            (0..size)
                .map(|i| [
                    i32::MIN,
                    -8193,
                    -8192,
                    -65,
                    -64,
                    -1,
                    0,
                    63,
                    64,
                    8191,
                    8192,
                    i32::MAX
                ][i % 12])
                .collect::<Vec<_>>()
        ),
        "u64-boundaries" => words!(
            (0..size)
                .map(|i| [
                    0,
                    127,
                    128,
                    16383,
                    16384,
                    u32::MAX as u64,
                    1_u64 << 32,
                    u64::MAX
                ][i % 8])
                .collect::<Vec<_>>()
        ),
        "id32" => words!(vec![[0xa5_u8; 32]; size]),
        "bytes" => {
            let values: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
            if bulk {
                compare!(
                    serde_bytes::ByteBuf::from(values.clone()),
                    serde_bytes::ByteBuf::from(values),
                );
            } else {
                compare!(values.clone(), values);
            }
        }
        "mixed" => {
            let words = vec![i16::MIN; size.min(4096)];
            if bulk {
                compare!(
                    mixed(size, candidate::Packed::<i16>::new(words.clone()).unwrap()),
                    mixed(size, reference::Packed::new(words)),
                );
            } else {
                compare!(mixed(size, words.clone()), mixed(size, words),);
            }
        }
        "nested" => {
            let values: Vec<i32> = (0..size)
                .map(|i| [-65, -64, 63, 64, 8191, 8192][i % 6])
                .collect();
            if bulk {
                compare!(
                    nested(candidate::Packed::<i32>::new(values.clone()).unwrap()),
                    nested(reference::Packed::new(values))
                );
            } else {
                compare!(nested(values.clone()), nested(values));
            }
        }
        "bools" | "bools-true" | "bools-false" => adapter!(
            (0..size)
                .map(|i| match name {
                    "bools-true" => true,
                    "bools-false" => false,
                    _ => i % 2 == 0,
                })
                .collect::<Vec<_>>(),
            NewBools,
            OldBools
        ),
        "finite-f32" | "finite-f32-normal" | "finite-f32-subnormal" => adapter!(
            (0..size)
                .map(|i| if name.ends_with("-normal") {
                    if i % 2 == 0 { 1.25_f32 } else { -1.25 }
                } else if name.ends_with("-subnormal") {
                    f32::from_bits(1)
                } else {
                    [0.0_f32, -0.0, f32::from_bits(1), 1.25, f32::MAX][i % 5]
                })
                .collect::<Vec<_>>(),
            NewF32,
            OldF32
        ),
        "finite-f64" | "finite-f64-normal" | "finite-f64-subnormal" => adapter!(
            (0..size)
                .map(|i| if name.ends_with("-normal") {
                    if i % 2 == 0 { 1.25_f64 } else { -1.25 }
                } else if name.ends_with("-subnormal") {
                    f64::from_bits(1)
                } else {
                    [0.0_f64, -0.0, f64::from_bits(1), 1.25, f64::MAX][i % 5]
                })
                .collect::<Vec<_>>(),
            NewF64,
            OldF64
        ),
        "nonzero-i64" => adapter!(
            (0..size)
                .map(|i| NonZeroI64::new(if i % 2 == 0 { -1 } else { i64::MAX }).unwrap())
                .collect::<Vec<_>>(),
            NewNonzero,
            OldNonzero
        ),
        "heap-u64" | "heap-ascending" | "heap-descending" => {
            let values: BinaryHeap<u64> = (0..size)
                .map(|i| match name {
                    "heap-ascending" => i as u64,
                    "heap-descending" => (size - i) as u64,
                    _ => (i % 17) as u64,
                })
                .collect();
            if bulk {
                compare!(NewHeap(values.clone()), OldHeap(values),);
            } else {
                compare!(OrdinaryHeap(values.clone()), OrdinaryHeap(values),);
            }
        }
        _ => unreachable!("validated case"),
    }
}
