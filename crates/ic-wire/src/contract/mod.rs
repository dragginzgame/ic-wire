use crate::{Error, Limits, Wire};
use alloc::vec::Vec;
use core::marker::PhantomData;

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

fn admit<T: Wire>(limits: Limits) -> Result<usize, Error> {
    let shape = T::SHAPE;
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

fn encode<T: Wire>(id: u64, direction: u8, limits: Limits, value: &T) -> Result<Vec<u8>, Error> {
    let size = admit::<T>(limits)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| Error::Allocation)?;
    bytes.resize(size, 0);
    bytes[..2].copy_from_slice(b"IW");
    bytes[2..10].copy_from_slice(&id.to_le_bytes());
    bytes[10] = direction;
    let payload = postcard::to_slice(value, &mut bytes[HEADER..])
        .map_err(|_| Error::Malformed)?
        .len();
    bytes.truncate(HEADER + payload);
    Ok(bytes)
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
    if encode(id, direction, limits, &value)? != bytes {
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
