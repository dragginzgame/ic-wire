//! Matched logical values for native, owned-decode performance measurements.
//! Alternate encodings are private controls with different admission guarantees.
#[cfg(any(
    feature = "performance-wire",
    feature = "performance-postcard",
    not(target_arch = "wasm32")
))]
use ic_wire::{Bytes, List, Text, Wire};
#[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
use ic_wire::{Limits, Method};
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
use std::marker::PhantomData;
use std::{fmt::Debug, hint::black_box};

/// Source-defined inputs, deliberately including opposite integer distributions.
pub const CASES: &[&str] = &[
    "scalar-zero",
    "scalar-max",
    "blob-empty",
    "blob-4096",
    "text-utf8",
    "integers-small",
    "integers-wide",
    "page-empty",
    "page-32",
];
/// Names in the report; native means dynamic Serde, not native CPU execution.
pub const LANES: &[&str] = &[
    "wire-finite",
    "wire-native",
    "postcard",
    "bitcode",
    "wincode",
    "postcard-bounded",
    "wire-native-bounded",
];
#[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
const ID: u64 = 0x50455246;
#[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
const LIMITS: Limits = Limits {
    bytes: 16384,
    allocation: 8 * 1024 * 1024,
    nodes: 65536,
    depth: 32,
};
const ITERATIONS: usize = 32;

#[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
struct Binding<T>(PhantomData<T>);
#[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
impl<T: Wire> Method for Binding<T> {
    type Request = T;
    type Response = T;
    const ID: u64 = ID;
    const NAME: &'static str = "performance";
    const REQUEST_LIMITS: Limits = LIMITS;
    const RESPONSE_LIMITS: Limits = LIMITS;
}

#[cfg(any(
    feature = "performance-wire",
    feature = "performance-postcard",
    not(target_arch = "wasm32")
))]
#[derive(Debug, PartialEq, Wire)]
struct FiniteRow {
    #[wire(id = 1)]
    id: u64,
    #[wire(id = 2)]
    name: Text<64>,
    #[wire(id = 3)]
    balance: u64,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "performance-bitcode",
    derive(bitcode::Encode, bitcode::Decode)
)]
#[cfg_attr(
    feature = "performance-wincode",
    derive(wincode::SchemaWrite, wincode::SchemaRead)
)]
struct Row {
    id: u64,
    name: String,
    balance: u64,
}

/// Counter deltas include a batch loop, allocation and destruction, not setup.
#[derive(Debug)]
pub struct Sample {
    /// Encode batch nanoseconds.
    pub encode: u64,
    /// Owned decode batch nanoseconds.
    pub decode: u64,
    /// Dynamic admission/sizing batch; zero for other lanes.
    pub sizing: u64,
    /// Unmodified encoding, including Wire's header where applicable.
    pub bytes: Vec<u8>,
}

fn roundtrip<T: PartialEq + Debug>(
    value: &T,
    encode: impl Fn(&T) -> Vec<u8>,
    decode: impl Fn(&[u8]) -> T,
    sizing: Option<impl Fn(&T) -> usize>,
    clock: &mut impl FnMut() -> u64,
) -> Sample {
    let bytes = encode(value);
    assert_eq!(decode(&bytes), *value);
    let start = clock();
    for _ in 0..ITERATIONS {
        drop(black_box(encode(black_box(value))));
    }
    let encoded = clock() - start;
    let start = clock();
    for _ in 0..ITERATIONS {
        drop(black_box(decode(black_box(&bytes))));
    }
    let decoded = clock() - start;
    let sized = if let Some(sizing) = sizing {
        let start = clock();
        for _ in 0..ITERATIONS {
            black_box(sizing(black_box(value)));
        }
        clock() - start
    } else {
        0
    };
    Sample {
        encode: encoded,
        decode: decoded,
        sizing: sized,
        bytes,
    }
}

/// Number of operations per counter delta; never subtract an estimated overhead.
pub const fn iterations() -> usize {
    ITERATIONS
}

/// Run the same logical case with an explicitly selected, compiled private lane.
pub fn run_case(lane: u8, case: u8, clock: &mut impl FnMut() -> u64) -> Sample {
    // Expansion keeps concrete codec bounds at each subject without a codec trait.
    macro_rules! subject {
        ($finite:expr, $plain:expr) => {{
            match lane {
                #[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
                0 => roundtrip(
                    &$finite,
                    |v| {
                        ic_wire::encode_request::<Binding<_>>(v)
                            .unwrap()
                            .into_bytes()
                    },
                    |b| ic_wire::decode_request::<Binding<_>>(b).unwrap(),
                    None::<fn(&_) -> usize>,
                    clock,
                ),
                #[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
                1 => roundtrip(
                    &$plain,
                    |v| ic_wire::native::encode_request(ID, v, LIMITS).unwrap(),
                    |b| ic_wire::native::decode_request(ID, b, LIMITS).unwrap(),
                    Some(|v: &_| ic_wire::native::encoded_frame_len(v, LIMITS).unwrap()),
                    clock,
                ),
                #[cfg(feature = "performance-postcard")]
                2 => roundtrip(
                    &$plain,
                    |v| postcard::to_allocvec(v).unwrap(),
                    |b| postcard::from_bytes(b).unwrap(),
                    None::<fn(&_) -> usize>,
                    clock,
                ),
                #[cfg(feature = "performance-postcard")]
                5 => roundtrip(
                    &$finite,
                    |v| postcard::to_allocvec(v).unwrap(),
                    |b| postcard::from_bytes(b).unwrap(),
                    None::<fn(&_) -> usize>,
                    clock,
                ),
                #[cfg(any(feature = "performance-wire", not(target_arch = "wasm32")))]
                6 => roundtrip(
                    &$finite,
                    |v| ic_wire::native::encode_request(ID, v, LIMITS).unwrap(),
                    |b| ic_wire::native::decode_request(ID, b, LIMITS).unwrap(),
                    Some(|v: &_| ic_wire::native::encoded_frame_len(v, LIMITS).unwrap()),
                    clock,
                ),
                #[cfg(feature = "performance-bitcode")]
                3 => roundtrip(
                    &$plain,
                    bitcode::encode,
                    |b| bitcode::decode(b).unwrap(),
                    None::<fn(&_) -> usize>,
                    clock,
                ),
                #[cfg(feature = "performance-wincode")]
                4 => roundtrip(
                    &$plain,
                    |v| wincode::serialize(v).unwrap(),
                    |b| wincode::deserialize_exact(b).unwrap(),
                    None::<fn(&_) -> usize>,
                    clock,
                ),
                _ => panic!("lane not compiled"),
            }
        }};
    }
    match case {
        0 => subject!(0u64, 0u64),
        1 => subject!(u64::MAX, u64::MAX),
        2 => subject!(Bytes::<4096>::new(vec![]).unwrap(), Vec::<u8>::new()),
        3 => subject!(
            Bytes::<4096>::new((0..4096).map(|i| (i % 251) as u8).collect()).unwrap(),
            (0..4096).map(|i| (i % 251) as u8).collect::<Vec<_>>()
        ),
        4 => subject!(
            Text::<1024>::new("é🦀".repeat(128)).unwrap(),
            "é🦀".repeat(128)
        ),
        5 => subject!(
            List::<u64, 256>::new((0..256).map(|i| i % 128).collect()).unwrap(),
            (0..256u64).map(|i| i % 128).collect::<Vec<_>>()
        ),
        6 => subject!(
            List::<u64, 256>::new((0..256).map(|i| u64::MAX - i).collect()).unwrap(),
            (0..256u64).map(|i| u64::MAX - i).collect::<Vec<_>>()
        ),
        7 | 8 => {
            let count = if case == 7 { 0 } else { 32 };
            subject!(
                List::<FiniteRow, 32>::new(
                    (0..count)
                        .map(|id| FiniteRow {
                            id,
                            name: Text::new(format!("row-{id}-é")).unwrap(),
                            balance: u64::MAX - id,
                        })
                        .collect()
                )
                .unwrap(),
                (0..count)
                    .map(|id| Row {
                        id,
                        name: format!("row-{id}-é"),
                        balance: u64::MAX - id,
                    })
                    .collect::<Vec<_>>()
            )
        }
        _ => panic!("unknown case"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_frames_match_payload_controls_for_every_case() {
        for case in 0..CASES.len() as u8 {
            let mut clock = || 0;
            let finite = run_case(0, case, &mut clock);
            let native = run_case(1, case, &mut clock);
            assert_eq!(finite.bytes, native.bytes);
            #[cfg(feature = "performance-postcard")]
            {
                assert_eq!(&finite.bytes[11..], run_case(2, case, &mut clock).bytes);
                assert_eq!(&finite.bytes[11..], run_case(5, case, &mut clock).bytes);
            }
            assert_eq!(finite.bytes, run_case(6, case, &mut clock).bytes);
            for lane in 3..=4 {
                if cfg!(feature = "performance-bitcode") && lane == 3
                    || cfg!(feature = "performance-wincode") && lane == 4
                {
                    run_case(lane, case, &mut clock);
                }
            }
        }
    }
}
