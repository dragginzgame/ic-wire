//! Explicit resource admission for native Serde payloads in the same Postcard format.
//!
//! Types, IDs, deterministic serialization and domain rules are consumer-owned.
//! This does not implement `Wire` for arbitrary types or infer a finite schema.
//! Custom Serde implementations are trusted: allocations/work outside visited
//! values cannot be constrained here. See the runtime guide for budget semantics.
mod decode;
mod encode;

use crate::{Error, Limits, contract};
use alloc::vec::Vec;
use core::cell::Cell;
use serde::{Serialize, de::DeserializeOwned};

struct Budget {
    nodes: Cell<usize>,
    heap: Cell<usize>,
    depth: usize,
    refused: Cell<bool>,
}
impl Budget {
    fn new(limits: Limits) -> Self {
        Self {
            nodes: Cell::new(limits.nodes),
            heap: Cell::new(limits.allocation.saturating_sub(limits.bytes)),
            depth: limits.depth,
            refused: Cell::new(limits.allocation < limits.bytes),
        }
    }
    fn enter(&self, depth: usize, size: usize) -> bool {
        if self.refused.get() {
            return false;
        }
        let Some(nodes) = self.nodes.get().checked_sub(1) else {
            self.refused.set(true);
            return false;
        };
        if depth > self.depth || !self.bytes(size) {
            self.refused.set(true);
            return false;
        }
        self.nodes.set(nodes);
        true
    }
    fn bytes(&self, bytes: usize) -> bool {
        let Some(cost) = bytes.checked_mul(8) else {
            self.refused.set(true);
            return false;
        };
        let Some(heap) = self.heap.get().checked_sub(cost) else {
            self.refused.set(true);
            return false;
        };
        self.heap.set(heap);
        true
    }
    fn error(&self) -> Error {
        if self.refused.get() {
            Error::Limit
        } else {
            Error::Malformed
        }
    }
}

/// Measure one native payload with no output allocation; `bytes` excludes a frame.
pub fn encoded_len<T: Serialize + ?Sized>(value: &T, limits: Limits) -> Result<usize, Error> {
    let budget = Budget::new(limits);
    let size = postcard::experimental::serialized_size(&encode::Bounded {
        value,
        budget: &budget,
        depth: 1,
    })
    .map_err(|_| budget.error())?;
    if size > limits.bytes {
        return Err(Error::Limit);
    }
    Ok(size)
}

/// Encode an owned or borrowed native payload under explicit traversal allowances.
pub fn encode<T: Serialize + ?Sized>(value: &T, limits: Limits) -> Result<Vec<u8>, Error> {
    encode_prefixed(value, limits, &[])
}
fn encode_prefixed<T: Serialize + ?Sized>(
    value: &T,
    limits: Limits,
    prefix: &[u8],
) -> Result<Vec<u8>, Error> {
    let size = encoded_len(value, payload_limits(limits, prefix.len())?)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size + prefix.len())
        .map_err(|_| Error::Allocation)?;
    bytes.extend_from_slice(prefix);
    let budget = Budget::new(limits);
    postcard::serialize_with_flavor(
        &encode::Bounded {
            value,
            budget: &budget,
            depth: 1,
        },
        contract::Output {
            bytes,
            limit: size + prefix.len(),
        },
    )
    .map_err(|_| budget.error())
}

/// Decode a complete canonical payload; allocation admission precedes visitor calls.
pub fn decode<T: Serialize + DeserializeOwned>(bytes: &[u8], limits: Limits) -> Result<T, Error> {
    if bytes.len() > limits.bytes {
        return Err(Error::Limit);
    }
    let budget = Budget::new(limits);
    let mut input = postcard::Deserializer::from_bytes(bytes);
    let value = T::deserialize(decode::Bounded {
        inner: &mut input,
        budget: &budget,
        depth: 1,
    })
    .map_err(|_| budget.error())?;
    if !input.finalize().map_err(|_| Error::Malformed)?.is_empty() {
        return Err(Error::Trailing);
    }
    let budget = Budget::new(limits);
    let equal = postcard::serialize_with_flavor(
        &encode::Bounded {
            value: &value,
            budget: &budget,
            depth: 1,
        },
        contract::Compare {
            remaining: bytes,
            allowance: limits.bytes,
            equal: true,
        },
    )
    .map_err(|_| budget.error())?;
    if !equal {
        return Err(Error::Noncanonical);
    }
    Ok(value)
}

/// Encode a complete request; `limits.bytes` includes the canonical frame header.
pub fn encode_request<T: Serialize + ?Sized>(
    id: u64,
    value: &T,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    encode_prefixed(value, limits, &contract::header(id, 0))
}
/// Encode a complete response, including borrowed response projections.
pub fn encode_response<T: Serialize + ?Sized>(
    id: u64,
    value: &T,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    encode_prefixed(value, limits, &contract::header(id, 1))
}
/// Measure a request or response frame; both directions have equal header length.
pub fn encoded_frame_len<T: Serialize + ?Sized>(value: &T, limits: Limits) -> Result<usize, Error> {
    encoded_len(value, payload_limits(limits, contract::HEADER)?)
        .map(|size| size + contract::HEADER)
}
/// Check request identity and direction before decoding its bounded native value.
pub fn decode_request<T: Serialize + DeserializeOwned>(
    id: u64,
    bytes: &[u8],
    limits: Limits,
) -> Result<T, Error> {
    decode_frame(id, 0, bytes, limits)
}
/// Check response identity and direction before decoding its bounded native value.
pub fn decode_response<T: Serialize + DeserializeOwned>(
    id: u64,
    bytes: &[u8],
    limits: Limits,
) -> Result<T, Error> {
    decode_frame(id, 1, bytes, limits)
}
fn decode_frame<T: Serialize + DeserializeOwned>(
    id: u64,
    direction: u8,
    bytes: &[u8],
    limits: Limits,
) -> Result<T, Error> {
    if bytes.len() > limits.bytes {
        return Err(Error::Limit);
    }
    let payload = contract::payload(id, direction, bytes)?;
    decode(payload, payload_limits(limits, contract::HEADER)?)
}

fn payload_limits(limits: Limits, prefix: usize) -> Result<Limits, Error> {
    Ok(Limits {
        bytes: limits.bytes.checked_sub(prefix).ok_or(Error::Limit)?,
        allocation: limits.allocation.checked_sub(prefix).ok_or(Error::Limit)?,
        ..limits
    })
}
