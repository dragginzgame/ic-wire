#[cfg(test)]
mod tests;

use crate::{Error, Limits, Wire};
use alloc::vec::Vec;
use core::{cell::Cell, marker::PhantomData};
use postcard::ser_flavors::Flavor;

pub(crate) const HEADER: usize = 11;

/// One consumer-owned endpoint contract. IDs must be unique across incompatible
/// layouts and methods. A source rename alone must not change its identity.
pub trait Method {
    /// Typed request, including any consumer-owned proof/envelope.
    /// Borrowed fields can use lifetimes on the implementing method marker.
    type Request: Wire;
    /// Typed response, typically `Result<Success, ApplicationError>`.
    type Response: Wire;
    /// Stable contract identity assigned by the consumer.
    const ID: u64;
    /// Exact exported IC method name; used by typed transport adapters.
    const NAME: &'static str;
    /// Request admission budget.
    const REQUEST_LIMITS: Limits;
    /// Reply admission budget.
    const RESPONSE_LIMITS: Limits;
}

macro_rules! retained {
    ($name:ident) => {
        /// Validated, immutable bytes bound to a method and direction. Retain
        /// these for retries; the codec does not decide when retry is safe.
        pub struct $name<M: Method> {
            bytes: Vec<u8>,
            method: PhantomData<fn() -> M>,
        }
        impl<M: Method> $name<M> {
            /// Borrow the exact encoding for transport or hashing.
            pub fn as_bytes(&self) -> &[u8] {
                &self.bytes
            }
            /// Transfer ownership to a transport or persistence owner.
            pub fn into_bytes(self) -> Vec<u8> {
                self.bytes
            }
        }
    };
}
retained!(RequestBytes);
retained!(ResponseBytes);

pub(crate) fn admit<T: Wire>(limits: Limits) -> Result<usize, Error> {
    let shape = T::SHAPE;
    if [shape.bytes, shape.heap, shape.nodes, shape.depth].contains(&usize::MAX) {
        return Err(Error::SchemaLimit);
    }
    let size = shape.bytes.checked_add(HEADER).ok_or(Error::SchemaLimit)?;
    let allocation = shape.heap.checked_add(size).ok_or(Error::SchemaLimit)?;
    if size > limits.bytes
        || allocation > limits.allocation
        || shape.nodes > limits.nodes
        || shape.depth > limits.depth
    {
        return Err(Error::SchemaLimit);
    }
    Ok(size)
}

// Reserve fallibly once, then initialize only emitted bytes. Capacity is not
// the schema limit: allocators may return more capacity than requested.
pub(crate) struct Output {
    pub(crate) bytes: Vec<u8>,
    pub(crate) limit: usize,
}
impl Flavor for Output {
    type Output = Vec<u8>;
    fn try_push(&mut self, byte: u8) -> postcard::Result<()> {
        if self.bytes.len() == self.limit {
            return Err(postcard::Error::SerializeBufferFull);
        }
        self.bytes.push(byte);
        Ok(())
    }
    fn try_extend(&mut self, bytes: &[u8]) -> postcard::Result<()> {
        if bytes.len() > self.limit - self.bytes.len() {
            return Err(postcard::Error::SerializeBufferFull);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self.bytes)
    }
}

// Finite encoding visits the value once. Grow only for emitted bytes, keeping
// allocation requests within the schema ceiling even when Vec has spare capacity.
struct GrowingOutput<'a> {
    output: Output,
    allocation_failed: &'a Cell<bool>,
}
impl GrowingOutput<'_> {
    #[cold]
    #[inline(never)]
    fn reserve(&mut self, additional: usize) -> postcard::Result<()> {
        let bytes = &mut self.output.bytes;
        if additional > self.output.limit - bytes.len() {
            return Err(postcard::Error::SerializeBufferFull);
        }
        let required = bytes.len() + additional;
        if required > bytes.capacity() {
            // Once a message is substantial relative to its ceiling, reserve
            // the remainder to avoid repeated copies of large batch payloads.
            let capacity = if self.output.limit <= 128 || required > self.output.limit / 4 {
                self.output.limit
            } else {
                required
                    .max(bytes.capacity().saturating_mul(2).max(64))
                    .min(self.output.limit)
            };
            bytes
                .try_reserve_exact(capacity - bytes.len())
                .map_err(|_| {
                    self.allocation_failed.set(true);
                    postcard::Error::SerializeBufferFull
                })?;
        }
        Ok(())
    }
}
impl Flavor for GrowingOutput<'_> {
    type Output = Vec<u8>;
    #[inline(always)]
    fn try_push(&mut self, byte: u8) -> postcard::Result<()> {
        if self.output.bytes.len() == self.output.limit {
            return Err(postcard::Error::SerializeBufferFull);
        }
        if self.output.bytes.len() == self.output.bytes.capacity() {
            self.reserve(1)?;
        }
        self.output.bytes.push(byte);
        Ok(())
    }
    #[inline(always)]
    fn try_extend(&mut self, bytes: &[u8]) -> postcard::Result<()> {
        if bytes.len() > self.output.limit - self.output.bytes.len() {
            return Err(postcard::Error::SerializeBufferFull);
        }
        if bytes.len() > self.output.bytes.capacity() - self.output.bytes.len() {
            self.reserve(bytes.len())?;
        }
        self.output.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self.output.bytes)
    }
}

// Count with the same admitted byte ceiling as encoding, without output storage.
struct Size {
    bytes: usize,
    limit: usize,
}
impl Flavor for Size {
    type Output = usize;
    fn try_push(&mut self, _: u8) -> postcard::Result<()> {
        self.try_extend(&[0])
    }
    fn try_extend(&mut self, bytes: &[u8]) -> postcard::Result<()> {
        if bytes.len() > self.limit - self.bytes {
            return Err(postcard::Error::SerializeBufferFull);
        }
        self.bytes += bytes.len();
        Ok(())
    }
    fn finalize(self) -> postcard::Result<usize> {
        Ok(self.bytes)
    }
}

fn encoded_len<T: Wire>(limits: Limits, value: &T) -> Result<usize, Error> {
    let limit = admit::<T>(limits)?;
    postcard::serialize_with_flavor(
        value,
        Size {
            bytes: HEADER,
            limit,
        },
    )
    .map_err(|_| Error::Malformed)
}

// Compare canonical serialization directly with the admitted payload. Continue
// after a mismatch so serializer errors still retain their Malformed mapping.
pub(crate) struct Compare<'a> {
    pub(crate) remaining: &'a [u8],
    pub(crate) allowance: usize,
    pub(crate) equal: bool,
}
impl Flavor for Compare<'_> {
    type Output = bool;
    #[inline(always)]
    fn try_push(&mut self, byte: u8) -> postcard::Result<()> {
        if self.allowance == 0 {
            return Err(postcard::Error::SerializeBufferFull);
        }
        self.allowance -= 1;
        if let Some((first, rest)) = self.remaining.split_first() {
            self.equal &= *first == byte;
            self.remaining = rest;
        } else {
            self.equal = false;
        }
        Ok(())
    }
    fn try_extend(&mut self, bytes: &[u8]) -> postcard::Result<()> {
        if let [byte] = bytes {
            return self.try_push(*byte);
        }
        if bytes.len() > self.allowance {
            return Err(postcard::Error::SerializeBufferFull);
        }
        self.allowance -= bytes.len();
        if let Some((prefix, rest)) = self.remaining.split_at_checked(bytes.len()) {
            // Wasm's bytewise memcmp is expensive for bulk byte/text payloads.
            // Exact unaligned word loads preserve canonical equality; no hash or
            // scratch allocation is involved. Small scalar writes stay simple.
            self.equal &= if cfg!(any(target_arch = "wasm32", test))
                && core::ptr::eq(prefix.as_ptr(), bytes.as_ptr())
            {
                // Borrowed decoding can serialize the exact input region.
                // Its length and allowance were checked above; equality needs no scan.
                true
            } else if cfg!(any(target_arch = "wasm32", test)) && bytes.len() >= 8 {
                if bytes.len() >= 64 {
                    let (left, left_tail) = prefix.as_chunks::<16>();
                    let (right, right_tail) = bytes.as_chunks::<16>();
                    left.iter().zip(right).fold(0_u128, |difference, (a, b)| {
                        difference | (u128::from_ne_bytes(*a) ^ u128::from_ne_bytes(*b))
                    }) == 0
                        && left_tail == right_tail
                } else {
                    let (left, left_tail) = prefix.as_chunks::<8>();
                    let (right, right_tail) = bytes.as_chunks::<8>();
                    left.iter().zip(right).fold(0_u64, |difference, (a, b)| {
                        difference | (u64::from_ne_bytes(*a) ^ u64::from_ne_bytes(*b))
                    }) == 0
                        && left_tail == right_tail
                }
            } else {
                prefix == bytes
            };
            self.remaining = rest;
        } else {
            self.equal = false;
            self.remaining = &[];
        }
        Ok(())
    }
    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self.equal && self.remaining.is_empty())
    }
}

fn encode<T: Wire>(id: u64, direction: u8, limits: Limits, value: &T) -> Result<Vec<u8>, Error> {
    let size = admit::<T>(limits)?;
    let allocation_failed = Cell::new(false);
    let mut output = GrowingOutput {
        output: Output {
            bytes: Vec::new(),
            limit: size,
        },
        allocation_failed: &allocation_failed,
    };
    output
        .try_extend(&header(id, direction))
        .map_err(|_| Error::Allocation)?;
    postcard::serialize_with_flavor(value, output).map_err(|_| {
        if allocation_failed.get() {
            Error::Allocation
        } else {
            Error::Malformed
        }
    })
}

fn decode<'de, T: Wire + serde::Deserialize<'de>>(
    id: u64,
    direction: u8,
    limits: Limits,
    bytes: &'de [u8],
) -> Result<T, Error> {
    let size = admit::<T>(limits)?;
    if bytes.len() > size || bytes.len() > limits.bytes {
        return Err(Error::Limit);
    }
    let payload = payload(id, direction, bytes)?;
    let (value, rest) = postcard::take_from_bytes::<T>(payload).map_err(|_| Error::Malformed)?;
    if !rest.is_empty() {
        return Err(Error::Trailing);
    }
    let equal = postcard::serialize_with_flavor(
        &value,
        Compare {
            remaining: payload,
            allowance: size - HEADER,
            equal: true,
        },
    )
    .map_err(|_| Error::Malformed)?;
    if !equal {
        return Err(Error::Noncanonical);
    }
    Ok(value)
}

/// Encode once, retaining exact request bytes bound to `M`.
pub fn encode_request<M: Method>(value: &M::Request) -> Result<RequestBytes<M>, Error> {
    Ok(RequestBytes {
        bytes: encode(M::ID, 0, M::REQUEST_LIMITS, value)?,
        method: PhantomData,
    })
}
/// Measure the complete request envelope without allocating an output buffer.
/// Uses the same schema admission as encoding. Custom serializers must be
/// deterministic; sizing does not retain bytes for retries.
pub fn encoded_request_len<M: Method>(value: &M::Request) -> Result<usize, Error> {
    encoded_len(M::REQUEST_LIMITS, value)
}
/// Measure the complete response envelope without allocating an output buffer.
/// Consumers can compare this with a delivery allowance before encoding. Schema
/// admission still applies to the entire type, not just this value.
pub fn encoded_response_len<M: Method>(value: &M::Response) -> Result<usize, Error> {
    encoded_len(M::RESPONSE_LIMITS, value)
}
/// Decode a request only after method, direction, canonical and resource checks.
pub fn decode_request<'de, M: Method>(bytes: &'de [u8]) -> Result<M::Request, Error>
where
    M::Request: serde::Deserialize<'de>,
{
    decode(M::ID, 0, M::REQUEST_LIMITS, bytes)
}
/// Encode a reply. Application errors belong inside `M::Response`.
pub fn encode_response<M: Method>(value: &M::Response) -> Result<ResponseBytes<M>, Error> {
    Ok(ResponseBytes {
        bytes: encode(M::ID, 1, M::RESPONSE_LIMITS, value)?,
        method: PhantomData,
    })
}
/// Decode a reply under the selected method's response contract.
pub fn decode_response<'de, M: Method>(bytes: &'de [u8]) -> Result<M::Response, Error>
where
    M::Response: serde::Deserialize<'de>,
{
    decode(M::ID, 1, M::RESPONSE_LIMITS, bytes)
}

pub(crate) fn header(id: u64, direction: u8) -> [u8; HEADER] {
    let mut header = [0; HEADER];
    header[..2].copy_from_slice(b"IW");
    header[2..10].copy_from_slice(&id.to_le_bytes());
    header[10] = direction;
    header
}
pub(crate) fn payload(id: u64, direction: u8, bytes: &[u8]) -> Result<&[u8], Error> {
    if bytes.len() < HEADER {
        return Err(Error::Malformed);
    }
    if &bytes[..2] != b"IW" || bytes[2..10] != id.to_le_bytes() || bytes[10] != direction {
        return Err(Error::Identity);
    }
    Ok(&bytes[HEADER..])
}
