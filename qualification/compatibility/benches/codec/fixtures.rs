//! Selected distributions from the pinned legacy hotspot/bulk studies.
//! Destination: one native runner, no copied legacy codec or orchestration.
use super::{Options, pair};
use serde::{Deserialize, Serialize};
use std::{collections::BinaryHeap, num::NonZeroI64};

pub const CASES: &[&str] = &[
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
    match name {
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
        "bools" => adapter!(
            (0..size).map(|i| i % 2 == 0).collect::<Vec<_>>(),
            NewBools,
            OldBools
        ),
        "finite-f32" => adapter!(
            (0..size)
                .map(|i| [0.0_f32, -0.0, f32::from_bits(1), 1.25, f32::MAX][i % 5])
                .collect::<Vec<_>>(),
            NewF32,
            OldF32
        ),
        "finite-f64" => adapter!(
            (0..size)
                .map(|i| [0.0_f64, -0.0, f64::from_bits(1), 1.25, f64::MAX][i % 5])
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
        "heap-u64" => {
            let values: BinaryHeap<u64> = (0..size).map(|i| (i % 17) as u64).collect();
            if bulk {
                compare!(NewHeap(values.clone()), OldHeap(values),);
            } else {
                compare!(OrdinaryHeap(values.clone()), OrdinaryHeap(values),);
            }
        }
        _ => unreachable!("validated case"),
    }
}
