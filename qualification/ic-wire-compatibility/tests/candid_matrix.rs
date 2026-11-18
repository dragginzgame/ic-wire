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
    checked_wire_len(value, encoding, MATRIX_LIMITS)
}

const MATRIX_LIMITS: Limits = Limits {
    bytes: 65_536,
    allocation: 4_194_304,
    nodes: 65_536,
    depth: 32,
};

fn checked_wire_len<T: Serialize + DeserializeOwned + PartialEq + Debug>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
) -> usize {
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

#[derive(CandidType, Debug, Deserialize, PartialEq, Serialize)]
enum Tree {
    Leaf(i32),
    Branch(Vec<Tree>),
}

#[test]
fn structured_payloads() {
    use std::collections::BTreeMap;
    compare("tree/empty", &Tree::Branch(vec![]));
    compare(
        "tree/deep",
        &(0..6).fold(Tree::Leaf(-1), |child, _| Tree::Branch(vec![child])),
    );
    compare(
        "tree/wide",
        &Tree::Branch((0..64).map(Tree::Leaf).collect()),
    );
    for len in [0, 1, 127, 128] {
        let map: BTreeMap<_, _> = (0..len)
            .map(|i: u32| (format!("key-{i}"), i.checked_sub(1)))
            .collect();
        compare(&format!("map/text-option/{len}"), &map);
        for (name, stride) in [("small", 0), ("sparse-wide", 16), ("wide", 1)] {
            let mut index = 0_usize;
            let ragged: Vec<Vec<u16>> = (0..len)
                .map(|row| {
                    (0..row % 3)
                        .map(|_| {
                            let wide = stride != 0 && index.is_multiple_of(stride);
                            index += 1;
                            if wide { u16::MAX } else { 1 }
                        })
                        .collect()
                })
                .collect();
            compare(&format!("nested/ragged/{name}/{len}"), &ragged);
        }
    }
    compare(
        "result/tuple",
        &Result::<_, String>::Ok((true, 128_u32, "温度".to_owned())),
    );
    compare(
        "result/error",
        &Result::<(bool, u32, String), _>::Err("拒否".to_owned()),
    );
}

#[test]
fn float_bits() {
    // Compare bits so signed zeros and NaN payloads remain distinguishable.
    for bits in [
        0,
        0x8000_0000,
        1,
        0x7f7f_ffff,
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_1234,
    ] {
        let value = f32::from_bits(bits);
        let limits = Limits {
            bytes: 64,
            allocation: 4096,
            nodes: 32,
            depth: 8,
        };
        let wire = wire::encode(&value, Encoding::Payload, limits).unwrap();
        let message = candid::encode_one(value).unwrap();
        assert_eq!(
            wire::decode::<f32>(&wire, Encoding::Payload, limits)
                .unwrap()
                .to_bits(),
            bits
        );
        assert_eq!(candid::decode_one::<f32>(&message).unwrap().to_bits(), bits);
        println!("f32/{bits:08x}\t{}\t{}", wire.len(), message.len());
    }
}

#[test]
fn platform_identities() {
    use ic_platform_types::{Account, Principal};

    #[derive(CandidType, Deserialize)]
    struct CandidAccount {
        owner: candid::Principal,
        subaccount: Option<[u8; 32]>,
    }

    let limits = Limits {
        bytes: 128,
        allocation: 8192,
        nodes: 128,
        depth: 16,
    };
    println!("case\twire_payload_bytes\tcandid_message_bytes");
    for (name, owner) in [
        ("management", Principal::management_canister()),
        ("anonymous", Principal::anonymous()),
        ("max-length", Principal::from_slice(&[7; 29])),
    ] {
        let candid_owner = candid::Principal::from_slice(owner.as_slice());
        let message = candid::encode_one(candid_owner).unwrap();
        assert_eq!(
            candid::decode_one::<candid::Principal>(&message)
                .unwrap()
                .as_slice(),
            owner.as_slice()
        );
        let size = wire_bytes(&owner, Encoding::Payload);
        println!("principal/{name}\t{size}\t{}", message.len());
        for (kind, subaccount) in [
            ("absent", None),
            ("zero", Some([0; 32])),
            ("nonzero", Some([17; 32])),
        ] {
            let account = Account { owner, subaccount };
            let bytes = wire::encode(&account, Encoding::Payload, limits).unwrap();
            let actual: Account = wire::decode(&bytes, Encoding::Payload, limits).unwrap();
            assert_eq!((actual.owner, actual.subaccount), (owner, subaccount));
            let message = candid::encode_one(CandidAccount {
                owner: candid_owner,
                subaccount,
            })
            .unwrap();
            let decoded: CandidAccount = candid::decode_one(&message).unwrap();
            assert_eq!(
                (decoded.owner.as_slice(), decoded.subaccount),
                (owner.as_slice(), subaccount)
            );
            println!("account/{name}/{kind}\t{}\t{}", bytes.len(), message.len());
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BulkFlags(#[serde(with = "wire::serde_packed::bools")] Vec<bool>);

#[test]
fn fixed_width_fields() {
    println!("case\twire_payload_bytes\tcandid_message_bytes\tbulk_wire_bytes");
    for len in [0, 1, 3, 4, 7, 8, 127, 128, 511, 512, 1024] {
        let flags: Vec<_> = (0..len).map(|i| i % 3 == 0).collect();
        let (ordinary, candid) = lengths(&flags, Encoding::Payload);
        let bulk = wire_bytes(&BulkFlags(flags), Encoding::Payload);
        assert_eq!(ordinary, bulk, "one byte per flag in both representations");
        println!("flags/{len}\t{ordinary}\t{candid}\t{bulk}");
        hash_list::<16>(len);
        hash_list::<32>(len);
    }
}

fn hash_list<const WIDTH: usize>(len: usize)
where
    [u8; WIDTH]: CandidType + Debug + Serialize + DeserializeOwned + wire::PackedElement,
{
    let hashes: Vec<[u8; WIDTH]> = (0..len)
        .map(|i| std::array::from_fn(|j| (i + j).to_le_bytes()[0]))
        .collect();
    let limits = Limits {
        nodes: 524_288,
        ..MATRIX_LIMITS
    };
    let ordinary = checked_wire_len(&hashes, Encoding::Payload, limits);
    let message = candid::encode_one(&hashes).unwrap();
    assert_eq!(
        candid::decode_one::<Vec<[u8; WIDTH]>>(&message).unwrap(),
        hashes
    );
    let candid = message.len();
    if WIDTH == 32 && len == 511 {
        assert_eq!(
            wire::encode(&hashes, Encoding::Payload, MATRIX_LIMITS),
            Err(wire::Error::Limit)
        );
    }
    let packed = wire::Packed::<_, 1024>::new(hashes).unwrap();
    let bulk = checked_wire_len(&packed, Encoding::Payload, limits);
    if WIDTH == 32 && len == 511 {
        assert_eq!(wire_bytes(&packed, Encoding::Payload), bulk);
    }
    println!("hash{WIDTH}/{len}\t{ordinary}\t{candid}\t{bulk}");
}
