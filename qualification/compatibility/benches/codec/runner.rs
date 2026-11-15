// Native hotspot investigation. No production codec or timing hooks live here.
mod fixtures;
mod suppliers;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
#[cfg(not(feature = "bench-allocations"))]
use std::time::{Duration, Instant};
use std::{fmt::Debug, hint::black_box};
use suppliers::{Candidate, Reference, Supplier};

#[cfg(feature = "bench-allocations")]
#[global_allocator]
static GLOBAL: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;

const OPERATIONS: &[&str] = &[
    "preflight",
    "encode-owned",
    "encode-reuse",
    "encode-slice",
    "decode",
    "decode-borrowed-text",
    "decode-borrowed-bytes",
    "invalid-first",
    "invalid-middle",
    "invalid-last",
    "bad-header",
    "truncated",
    "short-header",
    "wrong-method",
    "wrong-direction",
    "trailing",
    "loop",
];
#[derive(Debug)]
struct Options {
    case: String,
    representation: String,
    supplier: String,
    operation: String,
    mode: String,
    sizes: Vec<usize>,
    samples: usize,
    batch_ms: u64,
    seconds: u64,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            case: "all".into(),
            representation: "both".into(),
            supplier: "both".into(),
            operation: "all".into(),
            mode: "measure".into(),
            sizes: vec![0, 1, 128, 4096],
            samples: 9,
            batch_ms: 2,
            seconds: 10,
        }
    }
}
impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self::default();
        let mut args = args.into_iter();
        while let Some(key) = args.next() {
            if key == "--bench" {
                continue;
            }
            if key == "--list" {
                options.mode = "list".into();
                continue;
            }
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {key}"))?;
            let number = || {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid number for {key}"))
            };
            match key.as_str() {
                "--case" => options.case = value,
                "--representation" => options.representation = value,
                "--supplier" => options.supplier = value,
                "--operation" => options.operation = value,
                "--mode" => options.mode = value,
                "--size" if value == "boundaries" => {
                    options.sizes =
                        vec![0, 1, 2, 7, 8, 15, 16, 63, 64, 127, 128, 129, 255, 256, 4096]
                }
                "--size" if value == "all" => options.sizes = Self::default().sizes,
                "--size" => {
                    options.sizes =
                        vec![usize::try_from(number()?).map_err(|_| "size does not fit usize")?]
                }
                "--samples" => {
                    options.samples =
                        usize::try_from(number()?).map_err(|_| "sample count does not fit usize")?
                }
                "--batch-ms" => options.batch_ms = number()?,
                "--seconds" => options.seconds = number()?,
                _ => return Err(format!("unknown option {key}")),
            }
        }
        if options.case != "all" && !fixtures::CASES.contains(&options.case.as_str()) {
            return Err("unknown case; use --list".into());
        }
        if options.case != "all"
            && options.representation == "bulk"
            && !fixtures::supports_bulk(&options.case)
        {
            return Err("this case has only an ordinary representation".into());
        }
        if options.operation != "all" && !OPERATIONS.contains(&options.operation.as_str()) {
            return Err("unknown operation; use --list".into());
        }
        if !["ordinary", "bulk", "both"].contains(&options.representation.as_str())
            || !["candidate", "reference", "both"].contains(&options.supplier.as_str())
            || !["measure", "profile", "check", "list"].contains(&options.mode.as_str())
        {
            return Err("invalid representation, supplier or mode".into());
        }
        if options.sizes.iter().any(|n| *n > 1_048_576)
            || !(3..=51).contains(&options.samples)
            || !(1..=100).contains(&options.batch_ms)
            || !(1..=120).contains(&options.seconds)
        {
            return Err(
                "size <= 1048576, samples 3..51, batch-ms 1..100 and seconds 1..120 required"
                    .into(),
            );
        }
        if options.operation != "all"
            && !fixtures::CASES.iter().any(|case| {
                (options.case == "all" || options.case == *case)
                    && options.sizes.iter().any(|size| {
                        [false, true].into_iter().any(|bulk| {
                            (!bulk || fixtures::supports_bulk(case))
                                && (options.representation == "both"
                                    || options.representation
                                        == if bulk { "bulk" } else { "ordinary" })
                                && supports_operation(case, bulk, *size, &options.operation)
                        })
                    })
            })
        {
            return Err("operation has no applicable selected fixtures".into());
        }
        if options.mode == "profile"
            && (options.case == "all"
                || options.operation == "all"
                || options.representation == "both"
                || options.supplier == "both"
                || options.sizes.len() != 1
                || cfg!(feature = "bench-allocations"))
        {
            return Err("profiling needs one case, representation, supplier, size and operation, without allocator instrumentation".into());
        }
        Ok(options)
    }
}

fn checked<C: Supplier, T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) -> Vec<u8> {
    let bytes = C::encode(value);
    assert_eq!(C::len(value), bytes.len());
    assert_eq!(&C::decode::<T>(&bytes).unwrap(), value);
    let mut reused = Vec::with_capacity(bytes.len());
    C::reuse(value, &mut reused);
    assert_eq!(reused, bytes);
    let mut slice = vec![0; bytes.len()];
    assert_eq!(C::slice(value, &mut slice), bytes.len());
    assert_eq!(slice, bytes);
    assert!(C::decode::<T>(&bytes[..bytes.len() - 1]).is_err());
    for name in [
        "bad-header",
        "short-header",
        "wrong-method",
        "wrong-direction",
        "trailing",
    ] {
        assert!(C::decode::<T>(&malformed(name, &bytes)).is_err(), "{name}");
    }
    bytes
}

fn supports_operation(case: &str, bulk: bool, size: usize, operation: &str) -> bool {
    match operation {
        "decode-borrowed-text" => case.starts_with("text-"),
        "decode-borrowed-bytes" => case == "bytes",
        "invalid-first" | "invalid-middle" | "invalid-last" => {
            bulk && size > 0
                && (case.starts_with("finite-")
                    || case.starts_with("bools")
                    || case == "nonzero-i64")
        }
        _ => true,
    }
}

fn invalid_word(case: &str, size: usize, operation: &str, input: &[u8]) -> Vec<u8> {
    let word = if case.starts_with("finite-f32") {
        f32::INFINITY.to_le_bytes().to_vec()
    } else if case.starts_with("finite-f64") {
        f64::INFINITY.to_le_bytes().to_vec()
    } else if case == "nonzero-i64" {
        vec![0; 8]
    } else {
        vec![2]
    };
    let position = match operation {
        "invalid-first" => 0,
        "invalid-middle" => size / 2,
        _ => size - 1,
    };
    let offset = input.len() - size * word.len() + position * word.len();
    let mut bytes = input.to_vec();
    bytes[offset..offset + word.len()].copy_from_slice(&word);
    bytes
}

fn malformed(name: &str, input: &[u8]) -> Vec<u8> {
    let mut bytes = input.to_vec();
    match name {
        "bad-header" => bytes[0] ^= 1,
        "wrong-method" => bytes[2] ^= 1,
        "wrong-direction" => bytes[10] = 1,
        "short-header" => bytes.truncate(10),
        "trailing" => bytes.push(0),
        _ => unreachable!("malformed operation"),
    }
    bytes
}

fn operation<'a, C: Supplier, T: Serialize + DeserializeOwned>(
    name: &str,
    value: &'a T,
    input: &'a [u8],
) -> Box<dyn FnMut() + 'a> {
    match name {
        "preflight" => Box::new(move || {
            black_box(C::len(black_box(value)));
        }),
        "encode-owned" => Box::new(move || {
            black_box(C::encode(black_box(value)));
        }),
        "encode-reuse" => {
            let mut output = Vec::with_capacity(input.len());
            Box::new(move || {
                C::reuse(black_box(value), &mut output);
                black_box(output.as_slice());
            })
        }
        "encode-slice" => {
            let mut output = vec![0; input.len()];
            Box::new(move || {
                let len = C::slice(black_box(value), &mut output);
                black_box(&output[..len]);
            })
        }
        "decode-borrowed-text" => Box::new(move || {
            black_box(C::decode::<&str>(black_box(input)).unwrap());
        }),
        "decode-borrowed-bytes" => Box::new(move || {
            black_box(C::decode::<&serde_bytes::Bytes>(black_box(input)).unwrap());
        }),
        "invalid-first" | "invalid-middle" | "invalid-last" => Box::new(move || {
            black_box(C::decode::<T>(black_box(input)).is_err());
        }),
        "decode" => Box::new(move || {
            black_box(C::decode::<T>(black_box(input)).unwrap());
        }),
        "truncated" => Box::new(move || {
            black_box(C::decode::<T>(black_box(&input[..input.len() - 1])).is_err());
        }),
        "bad-header" | "short-header" | "wrong-method" | "wrong-direction" | "trailing" => {
            let bad = malformed(name, input);
            Box::new(move || {
                black_box(C::decode::<T>(black_box(&bad)).is_err());
            })
        }
        "loop" => Box::new(move || {
            black_box(black_box(input).len());
        }),
        _ => unreachable!("validated operation"),
    }
}

#[cfg(not(feature = "bench-allocations"))]
fn elapsed(run: &mut dyn FnMut(), repetitions: u64) -> Duration {
    let start = Instant::now();
    for _ in 0..repetitions {
        run();
    }
    start.elapsed()
}
#[cfg(not(feature = "bench-allocations"))]
fn calibrate(run: &mut dyn FnMut(), millis: u64) -> u64 {
    for _ in 0..3 {
        run();
    }
    let mut repetitions = 1;
    while elapsed(run, repetitions) < Duration::from_millis(millis) && repetitions < 1_048_576 {
        repetitions *= 2;
    }
    repetitions
}

fn measurements<'a>(
    options: &Options,
    left: &'a mut dyn FnMut(),
    right: &'a mut dyn FnMut(),
) -> Vec<(&'static str, Value)> {
    #[cfg(feature = "bench-allocations")]
    {
        let mut output = Vec::new();
        for (name, run) in [(Candidate::NAME, left), (Reference::NAME, right)] {
            if options.supplier != "both" && options.supplier != name {
                continue;
            }
            run(); // Warm reusable buffers outside the measured allocation region.
            let region = stats_alloc::Region::new(GLOBAL);
            run();
            let stats = region.change();
            output.push((name,json!({"allocations":stats.allocations,"deallocations":stats.deallocations,"reallocations":stats.reallocations,"bytes_allocated":stats.bytes_allocated,"bytes_deallocated":stats.bytes_deallocated,"net_reallocated_bytes":stats.bytes_reallocated})));
        }
        output
    }
    #[cfg(not(feature = "bench-allocations"))]
    {
        if options.mode == "profile" {
            let run = if options.supplier == Candidate::NAME {
                left
            } else {
                right
            };
            let repetitions = calibrate(run, options.batch_ms);
            let start = Instant::now();
            let mut completed = 0_u64;
            while start.elapsed() < Duration::from_secs(options.seconds) {
                elapsed(run, repetitions);
                completed += repetitions;
            }
            let name = if options.supplier == Candidate::NAME {
                Candidate::NAME
            } else {
                Reference::NAME
            };
            return vec![(
                name,
                json!({"profile_iterations":completed,"elapsed_ns":start.elapsed().as_nanos()}),
            )];
        }
        let selected = [
            options.supplier != Reference::NAME,
            options.supplier != Candidate::NAME,
        ];
        let mut repetitions = [0; 2];
        if selected[0] {
            repetitions[0] = calibrate(left, options.batch_ms);
        }
        if selected[1] {
            repetitions[1] = calibrate(right, options.batch_ms);
        }
        let mut samples: [Vec<f64>; 2] = Default::default();
        for round in 0..options.samples {
            for index in if round % 2 == 0 { [0, 1] } else { [1, 0] } {
                if !selected[index] {
                    continue;
                }
                let run: &mut dyn FnMut() = if index == 0 { &mut *left } else { &mut *right };
                samples[index].push(
                    elapsed(run, repetitions[index]).as_nanos() as f64 / repetitions[index] as f64,
                );
            }
        }
        [Candidate::NAME, Reference::NAME].into_iter().enumerate().filter_map(|(i,name)| {
            if !selected[i] { return None; }
            let mut sorted = samples[i].clone(); sorted.sort_by(f64::total_cmp);
            Some((name,json!({"repetitions":repetitions[i],"samples_ns_per_op":samples[i],"min_ns":sorted[0],"median_ns":sorted[sorted.len()/2],"max_ns":sorted[sorted.len()-1]})))
        }).collect()
    }
}

fn pair<A, B>(options: &Options, case: &str, size: usize, bulk: bool, candidate: &A, reference: &B)
where
    A: Serialize + DeserializeOwned + PartialEq + Debug,
    B: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let input = checked::<Candidate, _>(candidate);
    assert_eq!(
        input,
        checked::<Reference, _>(reference),
        "supplier bytes differ for {case}"
    );
    for &name in OPERATIONS {
        if !supports_operation(case, bulk, size, name)
            || options.operation != "all" && options.operation != name
        {
            continue;
        }
        let bad = name
            .starts_with("invalid-")
            .then(|| invalid_word(case, size, name, &input));
        if let Some(bytes) = &bad {
            assert!(Candidate::decode::<A>(bytes).is_err());
            assert!(Reference::decode::<B>(bytes).is_err());
        }
        if options.mode == "check" {
            continue;
        }
        let operation_input = bad.as_deref().unwrap_or(&input);
        let mut left = operation::<Candidate, _>(name, candidate, operation_input);
        let mut right = operation::<Reference, _>(name, reference, operation_input);
        for (supplier, measurements) in measurements(options, &mut left, &mut right) {
            println!(
                "{}",
                json!({"case":case,"size":size,"representation":if bulk {"bulk"} else {"ordinary"},"supplier":supplier,"operation":name,"frame_bytes":input.len(),"payload_bytes":input.len()-11,"instrumented_allocator":cfg!(feature="bench-allocations"),"measurements":measurements})
            );
        }
    }
}

fn main() -> Result<(), String> {
    let options = Options::parse(std::env::args().skip(1))?;
    if options.mode == "list" {
        println!(
            "{}",
            json!({"cases":fixtures::CASES,"operations":OPERATIONS,"default_sizes":Options::default().sizes,"ordinary_only":fixtures::CASES.iter().filter(|case| !fixtures::supports_bulk(case)).collect::<Vec<_>>(),"limits":{"bytes":suppliers::BYTES,"allocation":suppliers::ALLOCATION,"nodes":suppliers::NODES,"depth":suppliers::DEPTH}})
        );
        return Ok(());
    }
    for &case in fixtures::CASES {
        if options.case != "all" && options.case != case {
            continue;
        }
        for &size in &options.sizes {
            for bulk in [false, true] {
                if bulk && !fixtures::supports_bulk(case)
                    || options.representation == "ordinary" && bulk
                    || options.representation == "bulk" && !bulk
                {
                    continue;
                }
                fixtures::run(&options, case, size, bulk);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_fixtures_agree_on_bytes_and_output_destinations() {
        use super::*;
        let options = Options {
            mode: "check".into(),
            ..Options::default()
        };
        for &case in fixtures::CASES {
            for size in [0, 1, 2, 7, 8, 127, 128, 129] {
                for bulk in [false, true] {
                    if bulk && !fixtures::supports_bulk(case) {
                        continue;
                    }
                    fixtures::run(&options, case, size, bulk);
                }
            }
        }
    }
    #[test]
    fn invalid_selections_fail_instead_of_silently_benchmarking_defaults() {
        use super::*;
        for args in [
            vec!["--case", "missing"],
            vec!["--case", "text-ascii", "--representation", "bulk"],
            vec!["--case", "i16-small", "--operation", "decode-borrowed-text"],
            vec![
                "--case",
                "finite-f32",
                "--size",
                "0",
                "--operation",
                "invalid-first",
            ],
            vec!["--operation", "write-only"],
            vec!["--size", "1048577"],
            vec!["--samples", "0"],
            vec!["--mode", "profile"],
            vec!["--case"],
        ] {
            assert!(Options::parse(args.into_iter().map(String::from)).is_err());
        }
    }
}
