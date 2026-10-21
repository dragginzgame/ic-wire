//! Isolated native layout/canonical verification probe; see docs/protocol-study.md.
use ic_wire::{Bytes, BytesRef, Limits, List, Method, Wire};
use serde::Deserialize;
use std::{hint::black_box, marker::PhantomData, time::Instant};

struct Contract<T, const ID: u64>(PhantomData<T>);
impl<T: Wire, const ID: u64> Method for Contract<T, ID> {
    type Request = T;
    type Response = T;
    const ID: u64 = ID;
    const NAME: &'static str = "probe";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 2 * 1024 * 1024,
        allocation: 8 * 1024 * 1024,
        nodes: 4096,
        depth: 32,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}

// Experimental layout cut: tag 16384 becomes 1 (80 80 01 -> 01).
// They represent the same selected unit value, but use distinct method IDs;
// consumers must update both endpoints and cannot replay old frames as new ones.
#[derive(Wire)]
enum WideTag {
    #[wire(id = 16384)]
    Value,
}
#[derive(Wire)]
enum SmallTag {
    #[wire(id = 1)]
    Value,
}

fn sample(name: &str, stage: &str, size: usize, mut operation: impl FnMut() -> usize) {
    for _ in 0..100 {
        black_box(operation());
    }
    let mut repetitions = 1;
    loop {
        let start = Instant::now();
        for _ in 0..repetitions {
            black_box(operation());
        }
        if start.elapsed().as_millis() >= 10 || repetitions >= 65536 {
            break;
        }
        repetitions *= 2;
    }
    for batch in 0..11 {
        let start = Instant::now();
        for _ in 0..repetitions {
            black_box(operation());
        }
        println!(
            "{name},{stage},{size},{batch},{repetitions},{:.3}",
            start.elapsed().as_nanos() as f64 / f64::from(repetitions)
        );
    }
}

fn measure<'de, T: Wire + Deserialize<'de>, const ID: u64>(
    name: &str,
    value: &T,
    frame: &'de [u8],
) {
    let raw: T = postcard::from_bytes(&frame[11..]).unwrap();
    assert_eq!(
        ic_wire::encode_request::<Contract<T, ID>>(&raw)
            .unwrap()
            .as_bytes(),
        frame
    );
    let full = ic_wire::decode_request::<Contract<T, ID>>(frame).unwrap();
    assert_eq!(
        ic_wire::encode_request::<Contract<T, ID>>(&full)
            .unwrap()
            .as_bytes(),
        frame
    );
    assert!(ic_wire::probe_canonical(value, &frame[11..]));
    sample(name, "encode", frame.len(), || {
        black_box(ic_wire::encode_request::<Contract<T, ID>>(black_box(value)).unwrap())
            .as_bytes()
            .len()
    });
    sample(name, "raw_decode", frame.len(), || {
        black_box(postcard::from_bytes::<T>(black_box(&frame[11..])).unwrap());
        frame.len()
    });
    sample(name, "full_decode", frame.len(), || {
        black_box(ic_wire::decode_request::<Contract<T, ID>>(black_box(frame)).unwrap());
        frame.len()
    });
    // Exact production comparator, exposed only in the disposable benchmark build.
    sample(name, "canonical_only", frame.len(), || {
        usize::from(ic_wire::probe_canonical(
            black_box(value),
            black_box(&frame[11..]),
        ))
    });
}

fn run<T: Wire + for<'de> Deserialize<'de>, const ID: u64>(name: &str, value: &T, dir: &str) {
    let frame = ic_wire::encode_request::<Contract<T, ID>>(value).unwrap();
    std::fs::write(format!("{dir}/{name}.bin"), frame.as_bytes()).unwrap();
    measure::<T, ID>(name, value, frame.as_bytes());
}

fn main() {
    let directory = std::env::args()
        .nth(1)
        .expect("existing frame output directory");
    // Experimental cut: a variable-length key constrained by convention to 32
    // bytes becomes an exact [u8;32], dropping prefix 20. Other lengths cease to
    // be valid. New contract ID required; no automatic conversion/retry migration.
    run::<_, 1>(
        "key_bytes",
        &Bytes::<32>::new(vec![42; 32]).unwrap(),
        &directory,
    );
    run::<_, 2>("key_fixed", &[42_u8; 32], &directory);
    run::<_, 3>("tag_wide", &WideTag::Value, &directory);
    run::<_, 4>("tag_small", &SmallTag::Value, &directory);
    // Experimental cut: remove presence byte 01 only when the field is mandatory.
    // None has no representation in the new schema; this is not an equivalent API.
    run::<_, 5>("key_optional", &Some([42_u8; 32]), &directory);
    run::<_, 6>("key_required", &[42_u8; 32], &directory);
    run::<_, 7>(
        "integers",
        &List::<u32, 1024>::new(vec![u32::MAX; 1024]).unwrap(),
        &directory,
    );
    for size in [0, 32, 4096, 1048576] {
        let owned = Bytes::<1048576>::new(vec![42; size]).unwrap();
        run::<_, 8>(&format!("blob_owned_{size}"), &owned, &directory);
        let frame = ic_wire::encode_request::<Contract<Bytes<1048576>, 8>>(&owned).unwrap();
        let borrowed =
            ic_wire::decode_request::<Contract<BytesRef<'_, 1048576>, 8>>(frame.as_bytes())
                .unwrap();
        measure::<_, 8>(
            &format!("blob_borrowed_{size}"),
            &borrowed,
            frame.as_bytes(),
        );
    }
    // Nonminimal zero: raw Postcard accepts it, production must reject it.
    let mut frame = ic_wire::encode_request::<Contract<u32, 9>>(&0)
        .unwrap()
        .into_bytes();
    frame.splice(11.., [128, 0]);
    assert_eq!(postcard::from_bytes::<u32>(&frame[11..]).unwrap(), 0);
    assert_eq!(
        ic_wire::decode_request::<Contract<u32, 9>>(&frame).unwrap_err(),
        ic_wire::Error::Noncanonical
    );
}
