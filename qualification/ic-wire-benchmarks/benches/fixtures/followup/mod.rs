//! Missing boundary/Serde shapes and opt-in PRIVATE bulk-field research.
use super::*;
use std::collections::{BTreeMap, HashMap, HashSet};
mod words;

// Equality checks happen in setup, outside native/IC counters. This permits
// exact IEEE-bit validation without wrapping Candid's primitive element types
// (which would disable its concrete-type bulk fast path).
fn measured_lane<T: Debug + 'static>(
    name: &'static str,
    value: T,
    encode: impl Fn(&T) -> Vec<u8> + 'static,
    decode: impl Fn(&[u8]) -> Option<T> + 'static,
    equal: impl Fn(&T, &T) -> bool,
) -> Lane {
    let bytes = encode(&value);
    assert!(
        decode(&bytes).is_some_and(|decoded| equal(&decoded, &value)),
        "{name}: semantic/bit roundtrip"
    );
    Lane {
        name,
        bytes,
        encode: Box::new(move || encode(black_box(&value))),
        decode: Box::new(move |bytes| {
            let result = decode(black_box(bytes));
            let ok = result.is_some();
            drop(black_box(result));
            ok
        }),
        reuse: None,
    }
}
fn paired_values<W: Serialize + DeserializeOwned + PartialEq + Debug + 'static>(
    name: String,
    wire: W,
    #[cfg(feature = "candid")] candid: impl Serialize
    + DeserializeOwned
    + Compared
    + PartialEq
    + Debug
    + 'static,
    input_json: Vec<u8>,
) -> Case {
    #[cfg(not(feature = "wire"))]
    let _ = wire;
    Case {
        name,
        input_json,
        lanes: vec![
            #[cfg(feature = "wire")]
            measured_lane(
                "wire-native",
                wire,
                |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
                PartialEq::eq,
            ),
            #[cfg(feature = "candid")]
            measured_lane(
                "candid",
                candid,
                |v| candid::encode_one(v).unwrap(),
                |b| crate::fixtures::decode_candid(b).ok(),
                PartialEq::eq,
            ),
        ],
    }
}
macro_rules! paired {
    ($name:expr, $wire:expr, $candid:expr, $input:expr $(,)?) => {
        paired_values(
            $name,
            $wire,
            #[cfg(feature = "candid")]
            $candid,
            $input,
        )
    };
}
mod containers;
#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("candid"))]
struct ExternalAccountBytes {
    owner: candid::Principal,
    // Keep the standard Option visitor and IDL opt vec nat8 layout.
    subaccount: Option<ExternalBytes32>,
}
// The facade's Wasm Candid selection does not enable Candid's optional
// serde_bytes implementations. Mirror Blob's explicit bulk primitive locally,
// so native and Wasm controls use identical fixed-byte Serde/Candid paths.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, serde::Deserialize)]
#[serde(transparent)]
struct ExternalBytes32(#[serde(with = "serde_bytes")] [u8; 32]);
#[cfg(feature = "candid")]
impl CandidType for ExternalBytes32 {
    fn _ty() -> candid::types::Type {
        Vec::<u8>::ty()
    }
    fn idl_serialize<S: candid::types::Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_blob(&self.0)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("candid"))]
struct BorrowRecord {
    id: u64,
    label: String,
    payload: Blob,
}
#[derive(Serialize, serde::Deserialize)]
struct BorrowedRecord<'a> {
    id: u64,
    #[serde(borrow)]
    label: &'a str,
    #[serde(borrow, with = "serde_bytes")]
    payload: &'a [u8],
}
#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("candid"))]
enum ExternalIp {
    V4([u8; 4]),
    V6([u8; 16]),
}
#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("candid"))]
enum ExternalSocket {
    V4([u8; 4], u16),
    V6([u8; 16], u16),
}
fn borrowed_case(n: usize) -> Case {
    let value = BorrowRecord {
        id: 128,
        label: "é水🌍".into(),
        payload: Blob(data(n)),
    };
    let input = serde_json::to_vec(&value).unwrap();
    let case = paired!(
        format!("borrowed-record-{n}"),
        value.clone(),
        value.clone(),
        input
    );
    #[cfg(feature = "wire")]
    let mut case = case;
    #[cfg(feature = "wire")]
    {
        let bytes = case
            .lanes
            .iter()
            .find(|l| l.name == "wire-native")
            .unwrap()
            .bytes
            .clone();
        let view: BorrowedRecord<'_> =
            ic_wire::native::decode_request(METHOD, &bytes, LIMITS).unwrap();
        assert_eq!(view.payload, value.payload.0);
        assert_eq!(view.label, value.label);
        // Same bytes; views borrow an already-owned frame and do not remove ingress
        // copying. Their admission charges differ; later ownership needs its own budget.
        case.lanes.push(Lane {
            name: "wire-borrowed-record",
            bytes,
            encode: Box::new(move || {
                ic_wire::native::encode_request(
                    METHOD,
                    &BorrowedRecord {
                        id: value.id,
                        label: &value.label,
                        payload: &value.payload.0,
                    },
                    LIMITS,
                )
                .unwrap()
            }),
            decode: Box::new(|b| {
                let result =
                    ic_wire::native::decode_request::<BorrowedRecord<'_>>(METHOD, b, LIMITS);
                let ok = result.is_ok();
                drop(black_box(result));
                ok
            }),
            reuse: None,
        });
    }
    #[cfg(not(feature = "wire"))]
    let _ = value;
    case
}
#[derive(Debug, Default)]
struct DirectHash(u64);
impl std::hash::Hasher for DirectHash {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = self.0.wrapping_mul(1099511628211) ^ u64::from(*b);
        }
    }
    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }
}
type HashBuilder = std::hash::BuildHasherDefault<DirectHash>;

fn nat_case(n: usize) -> Case {
    let bytes = vec![255; n];
    let magnitude = ic_wire::Bytes::<64>::new(bytes.clone()).unwrap();
    let value = candid::Nat(num_bigint::BigUint::from_bytes_be(&bytes));
    let case = paired!(
        format!("bounded-nat-{n}"),
        magnitude,
        value.clone(),
        serde_json::to_vec(&bytes).unwrap()
    );
    #[cfg(feature = "wire")]
    let mut case = case;
    #[cfg(feature = "wire")]
    {
        let frame = case
            .lanes
            .iter()
            .find(|l| l.name == "wire-native")
            .unwrap()
            .bytes
            .clone();
        // A separately labelled APPLICATION conversion, not a second codec:
        // Wire itself only sees bounded magnitude bytes, never a Candid DTO.
        case.lanes.push(Lane {
            name: "wire-with-nat-conversion",
            bytes: frame,
            encode: Box::new(move || {
                let magnitude = crate::unsigned_magnitude::from_nat(&value).unwrap();
                ic_wire::native::encode_request(METHOD, &magnitude, LIMITS).unwrap()
            }),
            decode: Box::new(|b| {
                let value =
                    ic_wire::native::decode_request::<ic_wire::Bytes<64>>(METHOD, b, LIMITS)
                        .and_then(|v| crate::unsigned_magnitude::to_nat(&v));
                let ok = value.is_ok();
                drop(black_box(value));
                ok
            }),
            reuse: None,
        });
    }
    case
}
pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let mut out = Vec::new();
    let mut index = 0;
    macro_rules! push {
        ($case:expr) => {{
            if selected.is_none_or(|s| s == index) {
                out.push($case);
            }
            index += 1;
        }};
    }
    for n in [0, 1, 29] {
        let bytes = vec![4_u8; n];
        push!(paired!(
            format!("owned-principal-{n}"),
            ic_platform_types::Principal::from_slice(&bytes),
            candid::Principal::from_slice(&bytes),
            serde_json::to_vec(&bytes).unwrap()
        ));
    }
    for state in 0..3 {
        for n in [0, 1, 128] {
            push!({
                let wire = (0..n)
                    .map(|i| shapes::account(i, state))
                    .collect::<Vec<_>>();
                #[cfg(feature = "candid")]
                let candid = wire
                    .iter()
                    .copied()
                    .map(shapes::external)
                    .collect::<Vec<_>>();
                let input = serde_json::to_vec(&wire).unwrap();
                paired!(format!("account-plain-{state}-{n}"), wire, candid, input)
            });
            push!({
                let wire = (0..n)
                    .map(|i| shapes::account(i, state))
                    .collect::<Vec<_>>();
                #[cfg(feature = "candid")]
                let candid = wire
                    .iter()
                    .map(|a| ExternalAccountBytes {
                        owner: candid::Principal::from_slice(a.owner.as_slice()),
                        subaccount: a.subaccount.map(ExternalBytes32),
                    })
                    .collect::<Vec<_>>();
                let input = serde_json::to_vec(&wire).unwrap();
                paired!(format!("account-bytes-{state}-{n}"), wire, candid, input)
            });
        }
    }
    for n in [0, 1, 16, 128] {
        let values = (0..n)
            .map(|i| {
                let mut key = [0x95; 32];
                key[0] = i as u8;
                (key, Some(Blob(data(32))))
            })
            .collect::<BTreeMap<_, _>>();
        let input = serde_json::to_vec(&values.iter().collect::<Vec<_>>()).unwrap();
        push!({
            let wire = values
                .iter()
                .map(|(k, v)| (ic_wire::ByteArray::new(*k), v.clone()))
                .collect::<BTreeMap<_, _>>();
            paired!(
                format!("hash-key-plain-{n}"),
                wire,
                values.clone(),
                input.clone(),
            )
        });
        push!({
            let wire = values
                .iter()
                .map(|(k, v)| (ic_wire::ByteArray::new(*k), v.clone()))
                .collect::<BTreeMap<_, _>>();
            #[cfg(feature = "candid")]
            let candid = values
                .iter()
                .map(|(k, v)| (ExternalBytes32(*k), v.clone()))
                .collect::<BTreeMap<_, _>>();
            paired!(format!("hash-key-bytes-{n}"), wire, candid, input.clone())
        });
    }
    for n in [0, 1, 128, 16384] {
        push!({
            let values = (0..n).map(|i| u64::MAX - i as u64).collect::<Vec<_>>();
            paired!(
                format!("option-packed-some-{n}"),
                Some(ic_wire::Packed::new(values.clone())),
                Some(values.clone()),
                serde_json::to_vec(&values).unwrap(),
            )
        });
    }
    push!(paired!(
        "option-packed-none".into(),
        None::<ic_wire::Packed<u64>>,
        None::<Vec<u64>>,
        b"null".to_vec()
    ));
    for n in [0, 1, 16, 128] {
        push!({
            let v = (0..n).map(|i| vec![i as u64, u64::MAX]).collect::<Vec<_>>();
            let w = v
                .iter()
                .cloned()
                .map(ic_wire::Packed::new)
                .collect::<Vec<_>>();
            paired!(
                format!("nested-packed-{n}"),
                w,
                v.clone(),
                serde_json::to_vec(&v).unwrap(),
            )
        });
        push!({
            let v = (0..n)
                .map(|i| (i as u32, vec![i as u64, u64::MAX]))
                .collect::<BTreeMap<_, _>>();
            let w = v
                .iter()
                .map(|(k, v)| (*k, ic_wire::Packed::new(v.clone())))
                .collect::<BTreeMap<_, _>>();
            paired!(
                format!("map-packed-{n}"),
                w,
                v.clone(),
                serde_json::to_vec(&v).unwrap(),
            )
        });
        // Qualification-only fixed-hash, collision-free keys. Exact native/Wasm
        // frames must match; do not infer this order for arbitrary HashMaps.
        push!({
            let v = (0..n as u64)
                .map(|i| (i, Some(Blob(data(16)))))
                .collect::<HashMap<_, _, HashBuilder>>();
            case_with_input_json(
                &format!("hash-map-{n}"),
                v,
                serde_json::to_vec(&(0..n).collect::<Vec<_>>()).unwrap(),
            )
        });
        push!({
            let v = (0..n as u64).collect::<HashSet<_, HashBuilder>>();
            case_with_input_json(
                &format!("hash-set-{n}"),
                v,
                serde_json::to_vec(&(0..n).collect::<Vec<_>>()).unwrap(),
            )
        });
    }
    for n in [0, 1, 16, 32, 64] {
        push!(nat_case(n));
    }
    for (name, n) in [("zero", 0_u32), ("boundary", 128), ("max32", u32::MAX)] {
        push!(case(&format!("usize-{name}"), n as usize));
        push!(case(&format!("isize-{name}"), (n as i32) as isize));
        push!(case(
            &format!("system-time-{name}"),
            std::time::UNIX_EPOCH + std::time::Duration::new(u64::from(n), 999_999_999)
        ));
    }
    for n in [1_u16, 128, u16::MAX] {
        push!(paired!(
            format!("nonzero-u16-{n}"),
            std::num::NonZeroU16::new(n).unwrap(),
            n,
            serde_json::to_vec(&n).unwrap()
        ));
    }
    for (start, end) in [(0_u32, u32::MAX), (5, 2)] {
        push!(paired!(
            format!("range-{start}-{end}"),
            start..end,
            (start, end),
            serde_json::to_vec(&(start, end)).unwrap()
        ));
    }
    let v4 = [192, 0, 2, 1];
    push!(paired!(
        "ipv4".into(),
        std::net::Ipv4Addr::from(v4),
        v4,
        serde_json::to_vec(&v4).unwrap()
    ));
    let v6 = [0x2001_u16, 0xdb8, 0, 0, 0, 0, 0, 1];
    push!(paired!(
        "ipv6".into(),
        std::net::Ipv6Addr::from(v6),
        v6,
        serde_json::to_vec(&v6).unwrap()
    ));
    for n in [0, 1, 65536] {
        push!(borrowed_case(n));
    }
    let ip4 = std::net::Ipv4Addr::from(v4);
    let ip6 = std::net::Ipv6Addr::from(v6);
    push!(paired!(
        "ip-address-v4".into(),
        std::net::IpAddr::V4(ip4),
        ExternalIp::V4(v4),
        serde_json::to_vec(&v4).unwrap()
    ));
    push!(paired!(
        "ip-address-v6".into(),
        std::net::IpAddr::V6(ip6),
        ExternalIp::V6(ip6.octets()),
        serde_json::to_vec(&v6).unwrap()
    ));
    push!(paired!(
        "socket-v4".into(),
        std::net::SocketAddr::new(ip4.into(), 65535),
        ExternalSocket::V4(v4, 65535),
        serde_json::to_vec(&(v4, 65535_u16)).unwrap()
    ));
    push!(paired!(
        "socket-v6".into(),
        std::net::SocketAddr::new(ip6.into(), 65535),
        ExternalSocket::V6(ip6.octets(), 65535),
        serde_json::to_vec(&(v6, 65535_u16)).unwrap()
    ));
    push!(paired!(
        "range-inclusive".into(),
        std::ops::RangeInclusive::new(i64::MIN, i64::MAX),
        (i64::MIN, i64::MAX),
        serde_json::to_vec(&(i64::MIN, i64::MAX)).unwrap()
    ));
    for n in [0, 1, 128, 16384, 1_048_576] {
        push!(words::bool_case(
            format!("bool-words-{n}"),
            (0..n).map(|i| i % 2 == 0).collect()
        ));
    }
    for n in [0, 1, 128, 16384, 262144] {
        push!(words::case(
            format!("f32-bits-{n}"),
            (0..n)
                .map(|i| f32::from_bits(
                    [
                        0, 0x80000000, 1, 0x7f7fffff, 0x7f800000, 0xff800000, 0x7fc00001,
                        0x7f800001
                    ][i % 8]
                ))
                .collect()
        ));
    }
    for n in [0, 1, 128, 16384, 131072] {
        push!(words::case(
            format!("f64-bits-{n}"),
            (0..n)
                .map(|i| f64::from_bits(
                    [
                        0,
                        0x8000000000000000,
                        1,
                        0x7fefffffffffffff,
                        0x7ff0000000000000,
                        0xfff0000000000000,
                        0x7ff8000000000001,
                        0x7ff0000000000001
                    ][i % 8]
                ))
                .collect()
        ));
    }
    // Explicit public finite adapters, separate from the all-IEEE research cases.
    // New corpus identity: downstream case indices must come from this source.
    for n in [0, 1, 128, 16384, 262144] {
        push!(words::finite_f32_case(
            format!("finite-f32-{n}"),
            (0..n)
                .map(|i| [0.0, -0.0, f32::from_bits(1), f32::MAX][i % 4])
                .collect()
        ));
    }
    for n in [0, 1, 128, 16384, 131072] {
        push!(words::finite_f64_case(
            format!("finite-f64-{n}"),
            (0..n)
                .map(|i| [0.0, -0.0, f64::from_bits(1), f64::MAX][i % 4])
                .collect()
        ));
    }
    if selected.is_none_or(|s| s >= index) {
        out.extend(containers::cases_selected(selected.map(|s| s - index)));
    }
    out
}
#[cfg(all(test, feature = "wire", feature = "candid"))]
mod tests;
