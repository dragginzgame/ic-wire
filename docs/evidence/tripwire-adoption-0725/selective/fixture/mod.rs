//! Research only: one known Postcard shape, unchanged frames, no production API.
#![cfg(any(target_family = "wasm", test))]
use ic_wire::{Bytes, BytesRef, Error, Limits, native};
use serde::{Deserialize, Serialize};

const ID: u64 = 77;
const MAX: usize = 1_048_576;
const LIMITS: Limits = Limits {
    bytes: MAX + 128,
    allocation: 16 * MAX,
    nodes: 64,
    depth: 16,
};

#[derive(Serialize, Deserialize)]
struct Owned {
    mode: u8,
    nonce: u64,
    body: Bytes<MAX>,
}
#[derive(Serialize, Deserialize)]
struct Borrowed<'a> {
    mode: u8,
    nonce: u64,
    #[serde(borrow)]
    body: BytesRef<'a, MAX>,
}

fn packet(size: usize, mode: u8, nonce: u64) -> Vec<u8> {
    let body: Vec<_> = (0..size)
        .map(|i| (i as u8).wrapping_mul(17).wrapping_add(nonce as u8))
        .collect();
    native::encode_request(
        ID,
        &Borrowed {
            mode,
            nonce,
            body: BytesRef::new(&body).unwrap(),
        },
        LIMITS,
    )
    .unwrap()
}

#[derive(Debug)]
struct Window {
    mode: u8,
    nonce: u64,
    start: usize,
    length: usize,
}

fn locate(length: usize, copy: impl Fn(usize, &mut [u8])) -> Result<Window, Error> {
    if length > LIMITS.bytes {
        return Err(Error::Limit);
    }
    if length < 11 {
        return Err(Error::Malformed);
    }
    // Research restriction: this known DTO is u8, u64, then one byte field.
    // Use Wire's strict parser for its header/scalars/length, not a second varint
    // implementation. At most 11 + 1 + 10 + 5 bytes identify the blob range.
    // Tuple metadata and the complete DTO fit the fixed nonbinding traversal
    // allowances above. This is NOT general admission for arbitrary Limits.
    let mut prefix = [0; 27];
    let available = length.min(prefix.len());
    copy(0, &mut prefix[..available]);
    native::decode_request::<()>(ID, &prefix[..11], LIMITS)?;
    for end in 14..=available {
        if let Ok((mode, nonce, size)) =
            native::decode_request::<(u8, u64, u32)>(ID, &prefix[..end], LIMITS)
        {
            let size = size as usize;
            let stop = end.checked_add(size).ok_or(Error::Malformed)?;
            if stop > length || size > MAX {
                return Err(Error::Malformed);
            }
            if stop < length {
                return Err(Error::Trailing);
            }
            // Every body byte is valid. Validate its complete range and the
            // exact end before business work, even when no body bytes are read.
            return Ok(Window {
                mode,
                nonce,
                start: end,
                length: size,
            });
        }
    }
    Err(Error::Malformed)
}

fn consume(mode: u8, nonce: u64, bytes: &[u8]) -> [u64; 3] {
    let count = match mode {
        0 => bytes.len(),
        1 => bytes.len().min(256),
        2 => 0,
        _ => panic!("mode"),
    };
    [
        nonce,
        count as u64,
        bytes[..count]
            .iter()
            .fold(nonce, |sum, b| sum.wrapping_add(u64::from(*b))),
    ]
}

fn selected(value: &Window, copy: impl Fn(usize, &mut [u8])) -> [u64; 3] {
    let count = match value.mode {
        0 => value.length,
        1 => value.length.min(256),
        2 => 0,
        _ => panic!("mode"),
    };
    let mut scratch = [0; 4096];
    let mut sum = value.nonce;
    for start in (0..count).step_by(scratch.len()) {
        let take = (count - start).min(scratch.len());
        copy(value.start + start, &mut scratch[..take]);
        sum = scratch[..take]
            .iter()
            .fold(sum, |sum, b| sum.wrapping_add(u64::from(*b)));
    }
    [value.nonce, count as u64, sum]
}

fn direct_owned(value: &Window, copy: impl Fn(usize, &mut [u8])) -> [u64; 3] {
    // Available stable safe range-copy control. Platform currently has no safe
    // Vec-returning range helper that avoids initialization; do not hide this
    // zeroing cost or import unsafe allocation machinery into Wire.
    let mut body = vec![0; value.length];
    copy(value.start, &mut body);
    consume(value.mode, value.nonce, &body)
}

#[cfg(target_family = "wasm")]
fn copy(offset: usize, bytes: &mut [u8]) {
    ic_platform::ic0::msg_arg_data_copy(bytes, offset);
}
#[cfg(target_family = "wasm")]
fn reply(bytes: &[u8]) {
    ic_platform::ic0::msg_reply_data_append(bytes);
    ic_platform::ic0::msg_reply();
}

#[cfg(target_family = "wasm")]
#[allow(unsafe_code)]
#[unsafe(export_name = "canister_update consume")]
/// Research raw export: Platform executor integration is not qualified here.
pub extern "C" fn entry() {
    let length = ic_platform::ic0::msg_arg_data_size();
    assert!(length <= LIMITS.bytes);
    let result = if cfg!(feature = "selective-input") {
        selected(&locate(length, copy).unwrap(), copy)
    } else if cfg!(feature = "direct-owned-input") {
        direct_owned(&locate(length, copy).unwrap(), copy)
    } else {
        let input = ic_platform::cdk::api::msg_arg_data();
        if cfg!(feature = "borrowed-input") {
            let value: Borrowed<'_> = native::decode_request(ID, &input, LIMITS).unwrap();
            consume(value.mode, value.nonce, value.body.as_slice())
        } else {
            let value: Owned = native::decode_request(ID, &input, LIMITS).unwrap();
            drop(input);
            consume(value.mode, value.nonce, value.body.as_slice())
        }
    };
    let mut output = [0; 24];
    for (i, value) in result.iter().enumerate() {
        output[i * 8..i * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    reply(&output);
}

#[cfg(target_family = "wasm")]
#[allow(unsafe_code)]
#[unsafe(export_name = "canister_update encode")]
/// Same current Wire encoder remains reachable in all modules.
pub extern "C" fn encode_entry() {
    assert_eq!(ic_platform::ic0::msg_arg_data_size(), 13);
    let mut recipe = [0; 13];
    copy(0, &mut recipe);
    let size = u32::from_le_bytes(recipe[..4].try_into().unwrap()) as usize;
    let nonce = u64::from_le_bytes(recipe[4..12].try_into().unwrap());
    assert!(size <= MAX);
    reply(&packet(size, recipe[12], nonce));
}

#[cfg(test)]
mod tests;
