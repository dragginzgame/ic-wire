use crate::{Error, Limits, Wire};
use alloc::vec::Vec;
use core::marker::PhantomData;
use postcard::ser_flavors::Flavor;

const HEADER: usize = 11;

/// One consumer-owned endpoint contract. IDs must be unique across incompatible
/// layouts and methods. A source rename alone must not change its identity.
pub trait Method {
    /// Typed request, including any consumer-owned proof/envelope.
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
struct Output {
    bytes: Vec<u8>,
    limit: usize,
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
struct Compare<'a> {
    remaining: &'a [u8],
    allowance: usize,
    equal: bool,
}
impl Flavor for Compare<'_> {
    type Output = bool;
    fn try_push(&mut self, byte: u8) -> postcard::Result<()> {
        self.try_extend(&[byte])
    }
    fn try_extend(&mut self, bytes: &[u8]) -> postcard::Result<()> {
        if bytes.len() > self.allowance {
            return Err(postcard::Error::SerializeBufferFull);
        }
        self.allowance -= bytes.len();
        if let Some((prefix, rest)) = self.remaining.split_at_checked(bytes.len()) {
            self.equal &= prefix == bytes;
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
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| Error::Allocation)?;
    bytes.extend_from_slice(b"IW");
    bytes.extend_from_slice(&id.to_le_bytes());
    bytes.push(direction);
    postcard::serialize_with_flavor(value, Output { bytes, limit: size })
        .map_err(|_| Error::Malformed)
}

fn decode<T: Wire>(id: u64, direction: u8, limits: Limits, bytes: &[u8]) -> Result<T, Error> {
    let size = admit::<T>(limits)?;
    if bytes.len() > size || bytes.len() > limits.bytes {
        return Err(Error::Limit);
    }
    if bytes.len() < HEADER {
        return Err(Error::Malformed);
    }
    if &bytes[..2] != b"IW" || bytes[2..10] != id.to_le_bytes() || bytes[10] != direction {
        return Err(Error::Identity);
    }
    let (value, rest) =
        postcard::take_from_bytes::<T>(&bytes[HEADER..]).map_err(|_| Error::Malformed)?;
    if !rest.is_empty() {
        return Err(Error::Trailing);
    }
    let equal = postcard::serialize_with_flavor(
        &value,
        Compare {
            remaining: &bytes[HEADER..],
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
pub fn decode_request<M: Method>(bytes: &[u8]) -> Result<M::Request, Error> {
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
pub fn decode_response<M: Method>(bytes: &[u8]) -> Result<M::Response, Error> {
    decode(M::ID, 1, M::RESPONSE_LIMITS, bytes)
}
