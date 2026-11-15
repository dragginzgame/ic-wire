//! Shared preflight and destination-specific bounded Postcard output.

mod admitted;

use crate::{Direction, Encoding, Error, Limits, meter::Budget};
use alloc::vec::Vec;
use postcard::ser_flavors::Flavor;
use serde::Serialize;

fn preflight<T: Serialize + ?Sized>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
) -> Result<(Option<[u8; 11]>, usize), Error> {
    let header = match encoding {
        Encoding::Payload => None,
        Encoding::Frame { method, direction } => {
            let mut header = [0; 11];
            header[..2].copy_from_slice(b"IW");
            header[2..10].copy_from_slice(&method.to_le_bytes());
            header[10] = match direction {
                Direction::Request => 0,
                Direction::Response => 1,
            };
            Some(header)
        }
    };
    let prefix = if header.is_some() { 11 } else { 0 };
    let payload_limits = limits.without_prefix(prefix)?;
    let budget = Budget::new(payload_limits);
    let size = postcard::experimental::serialized_size(&admitted::Bounded {
        value,
        budget: &budget,
        depth: 1,
    })
    .map_err(|_| budget.error())?;
    if size > payload_limits.bytes {
        return Err(Error::Limit);
    }
    // The byte allowance was reduced by prefix before sizing, so addition fits.
    Ok((header, size + prefix))
}

/// Measure the complete selected encoding without allocating codec output.
///
/// Custom Serialize code is trusted and must be deterministic across calls.
/// This does not return a reusable admission token.
///
/// # Errors
/// Returns Limit for exhausted allowances, otherwise Malformed for serializer
/// failure. Framed lengths include the 11-byte header.
pub fn encoded_len<T: Serialize + ?Sized>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
) -> Result<usize, Error> {
    preflight(value, encoding, limits).map(|(_, size)| size)
}

/// Encode into owned storage after a complete sizing/admission pass.
///
/// # Errors
/// Returns the same errors as [`encode_into`]. No partial output is returned.
pub fn encode<T: Serialize + ?Sized>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    let mut output = Vec::new();
    encode_into(value, encoding, limits, &mut output)?;
    Ok(output)
}

/// Encode into reusable storage, returning only the completed prefix.
///
/// Prior contents are cleared, including on error; capacity retention on failure
/// is not promised. Keep retained retry bytes elsewhere. Preflight completes
/// before codec output reservation. Custom serializers must be deterministic;
/// writes remain bounded by preflight length even if a second pass grows.
///
/// # Errors
/// Returns Limit for exhausted allowances, Allocation for failed output
/// reservation, otherwise Malformed for serialization failure.
pub fn encode_into<'out, T: Serialize + ?Sized>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
    output: &'out mut Vec<u8>,
) -> Result<&'out [u8], Error> {
    output.clear();
    let (header, size) = preflight(value, encoding, limits)?;
    output
        .try_reserve_exact(size)
        .map_err(|_| Error::Allocation)?;
    if let Some(header) = header {
        output.extend_from_slice(&header);
    }
    let budget = Budget::new(limits);
    *output = postcard::serialize_with_flavor(
        &admitted::Bounded {
            value,
            budget: &budget,
            depth: 1,
        },
        Output {
            bytes: core::mem::take(output),
            limit: size,
        },
    )
    .map_err(|_| budget.error())?;
    Ok(output)
}

/// Encode into caller storage without allocating codec output.
///
/// A short destination is rejected before writing. Other failures may modify
/// storage; only a successful returned slice may be published. Custom Serialize
/// work is trusted and may allocate independently. Caller capacity does not
/// reduce the configured allocation allowance.
///
/// # Errors
/// Returns Limit for exhausted allowances or a short destination, otherwise
/// Malformed for serialization failure (including growth between passes).
pub fn encode_to_slice<'out, T: Serialize + ?Sized>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
    output: &'out mut [u8],
) -> Result<&'out [u8], Error> {
    let (header, size) = preflight(value, encoding, limits)?;
    let output = output.get_mut(..size).ok_or(Error::Limit)?;
    let prefix = if let Some(header) = header {
        output[..11].copy_from_slice(&header);
        11
    } else {
        0
    };
    let budget = Budget::new(limits);
    let payload = postcard::to_slice(
        &admitted::Bounded {
            value,
            budget: &budget,
            depth: 1,
        },
        &mut output[prefix..],
    )
    .map_err(|_| budget.error())?;
    let written = prefix + payload.len();
    Ok(&output[..written])
}

// Allocation capacity is not admission: every write is capped at preflight size.
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
