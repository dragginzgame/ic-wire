//! Matched values encoded as Wire payloads and single-argument Candid messages.
use candid::CandidType;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use wire::{Encoding, Limits};

#[derive(CandidType, Debug, Deserialize, PartialEq, Serialize)]
enum Outcome {
    Pending,
    Accepted(u64),
    Rejected(String),
}

#[derive(CandidType, Debug, Deserialize, PartialEq, Serialize)]
struct Update {
    revision: u64,
    label: String,
    samples: Vec<i16>,
    outcome: Option<Outcome>,
}

fn compare<T>(case: &str, value: &T)
where
    T: CandidType + Debug + DeserializeOwned + PartialEq + Serialize,
{
    let (wire, candid) = lengths(value, Encoding::Payload);
    println!("{case}\t{wire}\t{candid}");
}

#[test]
fn matched_payloads() {
    println!("case\twire_payload_bytes\tcandid_message_bytes");
    for n in [0, 127, 128, 16_383, 16_384, u64::MAX] {
        compare(&format!("u64/{n}"), &n);
    }
    for n in [i64::MIN, -65, -64, -1, 0, 63, 64, i64::MAX] {
        compare(&format!("i64/{n}"), &n);
    }
    compare("bool", &true);
    compare("float64", &1.25_f64);
    compare("text/empty", &String::new());
    compare("text/utf8", &String::from("温度🌡"));
    for len in [0, 1, 127, 128, 4096] {
        let bytes: Vec<u8> = (0..=255).cycle().take(len).collect();
        compare(&format!("blob/{len}"), &serde_bytes::ByteBuf::from(bytes));
        compare(&format!("u32-list/{len}"), &vec![128_u32; len]);
        compare(&format!("u32-max-list/{len}"), &vec![u32::MAX; len]);
    }
    for (name, outcome) in [
        ("absent", None),
        ("pending", Some(Outcome::Pending)),
        ("accepted", Some(Outcome::Accepted(128))),
        ("rejected", Some(Outcome::Rejected(String::from("invalid")))),
    ] {
        compare(
            name,
            &Update {
                revision: 129,
                label: String::from("sensor"),
                samples: vec![i16::MIN, 0, i16::MAX],
                outcome,
            },
        );
    }
}

/// Return the delivered lengths after checking the values on the receiving side.
fn lengths<T>(value: &T, encoding: Encoding) -> (usize, usize)
where
    T: CandidType + Debug + DeserializeOwned + PartialEq + Serialize,
{
    let ordinary = wire_bytes(value, encoding);
    let message = candid::encode_one(value).unwrap();
    assert_eq!(&candid::decode_one::<T>(&message).unwrap(), value);
    (ordinary, message.len())
}

fn wire_bytes<T: Serialize + DeserializeOwned + PartialEq + Debug>(
    value: &T,
    encoding: Encoding,
) -> usize {
    let limits = Limits {
        bytes: 65_536,
        allocation: 4_194_304,
        nodes: 65_536,
        depth: 32,
    };
    let payload = wire::encode(value, encoding, limits).unwrap();
    assert_eq!(
        &wire::decode::<T>(&payload, encoding, limits).unwrap(),
        value
    );
    payload.len()
}

fn numeric_rows<T>(name: &str, small: T, wide: T)
where
    T: wire::PackedElement + CandidType + DeserializeOwned + Serialize,
{
    for len in [1, 127, 128, 4096] {
        for (distribution, stride) in [
            ("small", 0),
            ("sparse-wide", 16),
            ("half-wide", 2),
            ("wide", 1),
        ] {
            let words: Vec<T> = (0..len)
                .map(|i| {
                    if stride != 0 && i % stride == 0 {
                        wide
                    } else {
                        small
                    }
                })
                .collect();
            let (ordinary, candid) = lengths(&words, Encoding::Payload);
            let packed = wire_bytes(
                &wire::Packed::<T, 4096>::new(words).unwrap(),
                Encoding::Payload,
            );
            println!("numeric/{name}/{distribution}/{len}\t{ordinary}\t{candid}\t{packed}");
        }
    }
}

#[test]
fn numeric_distributions() {
    println!("case\twire_payload_bytes\tcandid_message_bytes\tpacked_wire_bytes");
    numeric_rows("i16", -1_i16, i16::MIN);
    numeric_rows("u16", 1_u16, u16::MAX);
    numeric_rows("i32", -1_i32, i32::MIN);
    numeric_rows("u32", 1_u32, u32::MAX);
    numeric_rows("i64", -1_i64, i64::MIN);
    numeric_rows("u64", 1_u64, u64::MAX);
}

fn batch_row<T: CandidType + Debug + DeserializeOwned + PartialEq + Serialize>(
    name: &str,
    items: impl Iterator<Item = T>,
) {
    let values: Vec<T> = items.collect();
    let frame = Encoding::Frame {
        method: 7,
        direction: wire::Direction::Request,
    };
    let mut individual = (0, 0);
    for value in &values {
        let (wire, candid) = lengths(value, frame);
        individual.0 += wire;
        individual.1 += candid;
    }
    let (wire, candid) = lengths(&values, frame);
    println!(
        "batch/{name}/{}\t{}\t{}\t{}\t{}",
        values.len(),
        individual.0,
        wire,
        individual.1,
        candid
    );
}

#[test]
fn message_granularity() {
    println!(
        "case\tindividual_wire_frames\tbatched_wire_frame\tindividual_candid_messages\tbatched_candid_message"
    );
    for len in [1, 16, 128] {
        batch_row("counter", 0_u32..len);
        batch_row(
            "event",
            (0..len).map(|i| Update {
                revision: u64::from(i),
                label: String::from("sensor"),
                samples: vec![-1, 1, 2],
                outcome: Some(Outcome::Accepted(u64::from(i))),
            }),
        );
    }
}
