#![cfg_attr(any(feature = "tripwire_owned", feature = "tripwire_selective"), feature(allocator_ext))]
use serde::{Deserialize, Serialize};
pub const MAX: usize = 1048576;
pub const ID: u64 = 77;
#[cfg(any(feature = "wire_owned", feature = "wire_borrowed"))]
pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: MAX + 128,
    allocation: 16 * MAX,
    nodes: 64,
    depth: 16,
};
#[cfg(any(feature = "wire_owned", feature = "wire_borrowed"))]
#[derive(Serialize, Deserialize)]
struct Owned {
    mode: u8,
    nonce: u64,
    body: ic_wire::Bytes<MAX>,
}
#[cfg(any(feature = "wire_owned", feature = "wire_borrowed"))]
#[derive(Serialize, Deserialize)]
struct Borrowed<'a> {
    mode: u8,
    nonce: u64,
    #[serde(borrow)]
    body: ic_wire::BytesRef<'a, MAX>,
}
#[cfg(feature = "candid")]
use ic_platform::candid::contract as candid;
#[cfg(feature = "candid")]
#[derive(candid::CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
struct CandidRequest {
    mode: u8,
    nonce: u64,
    #[serde(with = "serde_bytes")]
    body: Vec<u8>,
}
#[cfg(any(feature = "tripwire_owned", feature = "tripwire_selective"))]
#[cdk_rsx_tripwire::message]
pub struct TripRequest {
    mode: u8,
    nonce: u64,
    body: Bytes<1048576>,
}
pub fn body(size: usize, nonce: u64) -> Vec<u8> {
    (0..size)
        .map(|i| (i as u8).wrapping_mul(17).wrapping_add(nonce as u8))
        .collect()
}
pub fn packet(lane: &str, size: usize, mode: u8, nonce: u64) -> Vec<u8> {
    let bytes = body(size, nonce);
    match lane {
        #[cfg(any(feature = "wire_owned", feature = "wire_borrowed"))]
        "wire_owned" | "wire_borrowed" => ic_wire::native::encode_request(
            ID,
            &Borrowed {
                mode,
                nonce,
                body: ic_wire::BytesRef::new(&bytes).unwrap(),
            },
            LIMITS,
        )
        .unwrap(),
        #[cfg(feature = "candid")]
        "candid" => candid::encode_one(CandidRequest {
            mode,
            nonce,
            body: bytes.into(),
        })
        .unwrap(),
        #[cfg(any(feature = "tripwire_owned", feature = "tripwire_selective"))]
        "tripwire_owned" | "tripwire_selective" => TripRequestArgs {
            mode,
            nonce,
            body: &bytes,
        }
        .encode()
        .unwrap(),
        _ => panic!("unknown lane"),
    }
}
fn consume(mode: u8, nonce: u64, bytes: &[u8]) -> [u64; 3] {
    let n = match mode {
        0 => bytes.len(),
        1 => bytes.len().min(256),
        2 => 0,
        _ => panic!("mode"),
    };
    [
        nonce,
        n as u64,
        bytes[..n]
            .iter()
            .fold(nonce, |sum, b| sum.wrapping_add(u64::from(*b))),
    ]
}
#[cfg(target_family = "wasm")]
fn lane() -> &'static str {
    if cfg!(feature = "wire_owned") {
        "wire_owned"
    } else if cfg!(feature = "wire_borrowed") {
        "wire_borrowed"
    } else if cfg!(feature = "candid") {
        "candid"
    } else if cfg!(feature = "tripwire_owned") {
        "tripwire_owned"
    } else {
        "tripwire_selective"
    }
}
#[cfg(target_family = "wasm")]
fn reply(bytes: &[u8]) {
    ic_platform::ic0::msg_reply_data_append(bytes);
    ic_platform::ic0::msg_reply();
}
#[cfg(target_family = "wasm")]
#[unsafe(export_name = "canister_update consume")]
pub extern "C" fn entry() {
    #[cfg(any(feature = "wire_owned", feature = "wire_borrowed", feature = "candid"))]
    let input = {
        let n = ic_platform::ic0::msg_arg_data_size();
        assert!(n <= MAX + 128);
        ic_platform::cdk::api::msg_arg_data()
    };
    #[cfg(feature = "wire_owned")]
    let result = {
        let v: Owned = ic_wire::native::decode_request(ID, &input, LIMITS).unwrap();
        drop(input);
        consume(v.mode, v.nonce, v.body.as_slice())
    };
    #[cfg(feature = "wire_borrowed")]
    let result = {
        let v: Borrowed<'_> = ic_wire::native::decode_request(ID, &input, LIMITS).unwrap();
        consume(v.mode, v.nonce, v.body.as_slice())
    };
    #[cfg(feature = "candid")]
    let result = {
        let v: CandidRequest = ic_platform::candid::decode_one(
            &input,
            ic_platform::candid::DecodeLimits {
                reply_bytes: MAX + 128,
                decoding_quota: 128 * MAX,
                skipping_quota: 128 * MAX,
                type_entries: 4096,
                header_bytes: 65536,
            },
        )
        .unwrap();
        drop(input);
        consume(v.mode, v.nonce, &v.body)
    };
    #[cfg(any(feature = "tripwire_owned", feature = "tripwire_selective"))]
    let result = TripRequest::with_current(MAX + 128, |v| {
        let mode = v.mode();
        let nonce = v.nonce();
        #[cfg(feature = "tripwire_owned")]
        {
            let body = v.body().unwrap().read_in(std::alloc::Global).unwrap();
            consume(mode, nonce, &body)
        }
        #[cfg(feature = "tripwire_selective")]
        {
            if mode == 2 {
                return [nonce, 0, nonce];
            }
            let body = v.body().unwrap();
            let n = match mode {
                0 => body.len(),
                1 => body.len().min(256),
                _ => panic!("mode"),
            };
            let mut scratch = [0; 4096];
            let mut sum = nonce;
            for at in (0..n).step_by(scratch.len()) {
                let end = (at + scratch.len()).min(n);
                let part = body.read_range_into(at..end, &mut scratch).unwrap();
                sum = part.iter().fold(sum, |s, b| s.wrapping_add(u64::from(*b)));
            }
            [nonce, n as u64, sum]
        }
    })
    .unwrap();
    let mut bytes = [0; 24];
    for (i, v) in result.iter().enumerate() {
        bytes[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
    }
    reply(&bytes);
}
#[cfg(target_family = "wasm")]
#[unsafe(export_name = "canister_update encode")]
pub extern "C" fn encode_entry() {
    assert_eq!(ic_platform::ic0::msg_arg_data_size(), 13);
    let mut args = [0; 13];
    ic_platform::ic0::msg_arg_data_copy(&mut args, 0);
    let size = u32::from_le_bytes(args[..4].try_into().unwrap()) as usize;
    let nonce = u64::from_le_bytes(args[4..12].try_into().unwrap());
    assert!(size <= MAX);
    reply(&packet(lane(), size, args[12], nonce));
}
