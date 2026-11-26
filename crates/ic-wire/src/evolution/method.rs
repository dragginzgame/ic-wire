//! Typed request and reply framing over the evolutionary codec.

use crate::{Direction, Encoding, Error, Limits, method_id};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// A value with typed encoding and projection through the evolutionary codec.
///
/// Decoded values may borrow from the frame or payload for `'de`.
pub trait Evolving<'de>: Sized {
    /// Encode one complete value into reusable output. Scratch is temporary
    /// storage for preparing evolutionary envelopes.
    ///
    /// # Errors
    /// Returns an encoding or admission error and leaves output empty.
    fn encode_evolving(
        &self,
        output: &mut Vec<u8>,
        encoding: Encoding,
        limits: Limits,
        scratch: &mut [u8],
    ) -> Result<(), Error>;

    /// Decode and project one complete frame or payload from the given input.
    ///
    /// # Errors
    /// Returns a framing, decoding, incompatibility, or admission error.
    fn decode_evolving(input: &'de [u8], encoding: Encoding, limits: Limits)
    -> Result<Self, Error>;
}

/// Bind an export name, direction limits and Rust DTOs to Wire frames.
///
/// An application can infer the lifetimes of borrowing DTOs locally with a type
/// such as `Method<Request<'_>, Reply<'_>>`.
pub struct Method<Request, Reply> {
    name: &'static str,
    id: u64,
    request_limits: Limits,
    reply_limits: Limits,
    types: PhantomData<fn() -> (Request, Reply)>,
}

impl<Request, Reply> Method<Request, Reply> {
    /// Define the Wire method for the exact exported name.
    ///
    /// A `const` descriptor computes the routing ID at compile time. The
    /// configured byte limits include the frame header in each direction.
    #[must_use]
    pub const fn new(name: &'static str, request_limits: Limits, reply_limits: Limits) -> Self {
        Self {
            name,
            id: method_id(name),
            request_limits,
            reply_limits,
            types: PhantomData,
        }
    }

    /// Get the exact exported name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Get the routing ID derived from the exported name.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Get the request limits.
    #[must_use]
    pub const fn request_limits(&self) -> Limits {
        self.request_limits
    }

    /// Get the reply limits.
    #[must_use]
    pub const fn reply_limits(&self) -> Limits {
        self.reply_limits
    }

    /// Encode a request frame into reusable output.
    ///
    /// # Errors
    /// Returns an encoding or request admission error.
    pub fn encode_request<'de>(
        &self,
        value: &Request,
        output: &mut Vec<u8>,
        scratch: &mut [u8],
    ) -> Result<(), Error>
    where
        Request: Evolving<'de>,
    {
        value.encode_evolving(
            output,
            self.encoding(Direction::Request),
            self.request_limits,
            scratch,
        )
    }

    /// Decode a request frame for this method.
    ///
    /// # Errors
    /// Returns a framing, decoding, incompatibility, or request admission error.
    pub fn decode_request<'de>(&self, input: &'de [u8]) -> Result<Request, Error>
    where
        Request: Evolving<'de>,
    {
        Request::decode_evolving(
            input,
            self.encoding(Direction::Request),
            self.request_limits,
        )
    }

    /// Encode a reply frame into reusable output.
    ///
    /// # Errors
    /// Returns an encoding or reply admission error.
    pub fn encode_reply<'de>(
        &self,
        value: &Reply,
        output: &mut Vec<u8>,
        scratch: &mut [u8],
    ) -> Result<(), Error>
    where
        Reply: Evolving<'de>,
    {
        value.encode_evolving(
            output,
            self.encoding(Direction::Response),
            self.reply_limits,
            scratch,
        )
    }

    /// Decode a reply frame for this method.
    ///
    /// # Errors
    /// Returns a framing, decoding, incompatibility, or reply admission error.
    pub fn decode_reply<'de>(&self, input: &'de [u8]) -> Result<Reply, Error>
    where
        Reply: Evolving<'de>,
    {
        Reply::decode_evolving(input, self.encoding(Direction::Response), self.reply_limits)
    }

    const fn encoding(&self, direction: Direction) -> Encoding {
        Encoding::Frame {
            method: self.id,
            direction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LimitKind, TextRef};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Request<'a>(#[serde(borrow)] TextRef<'a, 16>);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Reply(u16);

    impl<'de: 'a, 'a> Evolving<'de> for Request<'a> {
        fn encode_evolving(
            &self,
            output: &mut Vec<u8>,
            encoding: Encoding,
            limits: Limits,
            _scratch: &mut [u8],
        ) -> Result<(), Error> {
            crate::encode_into(self, encoding, limits, output).map(|_| ())
        }

        fn decode_evolving(
            input: &'de [u8],
            encoding: Encoding,
            limits: Limits,
        ) -> Result<Self, Error> {
            crate::decode(input, encoding, limits)
        }
    }

    impl<'de> Evolving<'de> for Reply {
        fn encode_evolving(
            &self,
            output: &mut Vec<u8>,
            encoding: Encoding,
            limits: Limits,
            _scratch: &mut [u8],
        ) -> Result<(), Error> {
            crate::encode_into(self, encoding, limits, output).map(|_| ())
        }

        fn decode_evolving(
            input: &'de [u8],
            encoding: Encoding,
            limits: Limits,
        ) -> Result<Self, Error> {
            crate::decode(input, encoding, limits)
        }
    }

    const REQUEST_LIMITS: Limits = Limits {
        bytes: 64,
        allocation: 4096,
        nodes: 64,
        depth: 16,
    };
    const REPLY_LIMITS: Limits = Limits {
        bytes: 12,
        ..REQUEST_LIMITS
    };
    const LOOKUP: Method<Request<'static>, Reply> =
        Method::new("lookup", REQUEST_LIMITS, REPLY_LIMITS);

    #[test]
    fn typed_methods_round_trip_borrowed_requests_and_check_identity() {
        assert_eq!(LOOKUP.name(), "lookup");
        assert_eq!(LOOKUP.id(), method_id("lookup"));
        assert_eq!(LOOKUP.request_limits().bytes, 64);
        assert_eq!(LOOKUP.reply_limits().bytes, 12);

        let value = alloc::string::String::from("hello");
        let mut input = Vec::new();
        let method: Method<Request<'_>, Reply> = LOOKUP;
        method
            .encode_request(&Request(TextRef::new(&value).unwrap()), &mut input, &mut [])
            .unwrap();
        let decoded = method.decode_request(&input).unwrap();
        assert_eq!(decoded.0.as_str(), "hello");
        assert_eq!(decoded.0.as_str().as_bytes().as_ptr(), input[12..].as_ptr());

        let other = Method::<Request<'_>, Reply>::new("other", REQUEST_LIMITS, REPLY_LIMITS);
        assert_eq!(other.decode_request(&input), Err(Error::Identity));
        let mut reply = Vec::new();
        method.encode_reply(&Reply(7), &mut reply, &mut []).unwrap();
        assert_eq!(reply[10], 1);
        assert_eq!(method.decode_reply(&reply), Ok(Reply(7)));
        assert_eq!(method.decode_request(&reply), Err(Error::Identity));
    }

    #[test]
    fn reply_limits_apply_to_both_encoding_and_decoding() {
        let mut reply = alloc::vec![1, 2, 3];
        assert_eq!(
            LOOKUP.encode_reply(&Reply(128), &mut reply, &mut []),
            Err(Error::Limit(LimitKind::Bytes))
        );
        assert!(reply.is_empty());
        let large = crate::encode(
            &Reply(128),
            Encoding::Frame {
                method: LOOKUP.id(),
                direction: Direction::Response,
            },
            REQUEST_LIMITS,
        )
        .unwrap();
        assert_eq!(
            LOOKUP.decode_reply(&large),
            Err(Error::Limit(LimitKind::Bytes))
        );
    }
}
