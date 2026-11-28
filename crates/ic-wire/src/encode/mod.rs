//! Shared preflight and destination-specific bounded Postcard output.

mod admitted;

use crate::{Direction, Encoding, Error, Limits, framing::HEADER_BYTES, meter::Budget};
use alloc::vec::Vec;
use postcard::ser_flavors::Flavor;
use serde::Serialize;

// Dispatch once per pass so framing, reservation and error handling are shared
// across root types. Field serialization stays statically dispatched.
type PayloadWriter<'a> = dyn Fn(&Budget, Output<'_>) -> Result<usize, Error> + 'a;

fn payload_writer<T: Serialize + ?Sized>(
    value: &T,
) -> impl Fn(&Budget, Output<'_>) -> Result<usize, Error> + '_ {
    move |budget, output| {
        postcard::serialize_with_flavor(
            &admitted::Bounded {
                value,
                budget,
                depth: 1,
            },
            output,
        )
        .map_err(|_| budget.error())
    }
}

fn preflight(
    write: &PayloadWriter<'_>,
    encoding: Encoding,
    limits: Limits,
) -> Result<(Option<[u8; HEADER_BYTES]>, usize), Error> {
    let header = frame_header(encoding);
    let prefix = if header.is_some() { HEADER_BYTES } else { 0 };
    let payload_limits = limits.without_prefix(prefix)?;
    let budget = Budget::new(payload_limits);
    let size = write(&budget, Output::Count(0))?;
    if size > payload_limits.bytes {
        return Err(Error::Limit(crate::LimitKind::Bytes));
    }

    // The byte allowance was reduced by prefix before sizing, so addition fits.
    Ok((header, size + prefix))
}

pub(super) fn frame_header(encoding: Encoding) -> Option<[u8; HEADER_BYTES]> {
    match encoding {
        Encoding::Payload => None,
        Encoding::Frame { method, direction } => {
            let mut header = [0; HEADER_BYTES];
            header[..2].copy_from_slice(b"IW");
            header[2..10].copy_from_slice(&method.to_le_bytes());
            header[10] = match direction {
                Direction::Request => 0,
                Direction::Response => 1,
            };
            Some(header)
        }
    }
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
    preflight(&payload_writer(value), encoding, limits).map(|(_, size)| size)
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
    encode_into_with(&payload_writer(value), encoding, limits, output)?;
    Ok(output)
}

#[inline(never)]
fn encode_into_with(
    write: &PayloadWriter<'_>,
    encoding: Encoding,
    limits: Limits,
    output: &mut Vec<u8>,
) -> Result<(), Error> {
    output.clear();
    let (header, size) = preflight(write, encoding, limits)?;
    write_vec_with(write, &Budget::new(limits), header, size, output)
}

pub(super) fn write_projected_vec<T: Serialize>(
    value: &T,
    budget: &Budget,
    header: Option<[u8; HEADER_BYTES]>,
    byte_limit: usize,
    output: &mut Vec<u8>,
) -> Result<(), Error> {
    write_projected_vec_with(&payload_writer(value), budget, header, byte_limit, output)
}

#[inline(never)]
fn write_projected_vec_with(
    write: &PayloadWriter<'_>,
    budget: &Budget,
    header: Option<[u8; HEADER_BYTES]>,
    byte_limit: usize,
    output: &mut Vec<u8>,
) -> Result<(), Error> {
    let size = write(budget, Output::Count(0))?;
    budget.check()?;
    if size > byte_limit {
        return Err(Error::Limit(crate::LimitKind::Bytes));
    }
    let prefix = if header.is_some() { HEADER_BYTES } else { 0 };
    let len = size + prefix;
    write_vec_with(write, budget, header, len, output)?;
    if output.len() != len {
        return Err(Error::Malformed);
    }
    Ok(())
}

fn write_vec_with(
    write: &PayloadWriter<'_>,
    budget: &Budget,
    header: Option<[u8; HEADER_BYTES]>,
    size: usize,
    output: &mut Vec<u8>,
) -> Result<(), Error> {
    output
        .try_reserve_exact(size)
        .map_err(|_| Error::Allocation)?;
    let mut bytes = core::mem::take(output);
    if let Some(header) = header {
        bytes.extend_from_slice(&header);
    }
    write(
        budget,
        Output::Buffer {
            bytes: &mut bytes,
            limit: size,
        },
    )?;
    *output = bytes;
    Ok(())
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
    let (header, size) = preflight(&payload_writer(value), encoding, limits)?;
    let output = output
        .get_mut(..size)
        .ok_or(Error::Limit(crate::LimitKind::Output))?;
    let prefix = if let Some(header) = header {
        output[..HEADER_BYTES].copy_from_slice(&header);

        HEADER_BYTES
    } else {
        0
    };
    let written = prefix + write_payload(value, &Budget::new(limits), 1, &mut output[prefix..])?;

    Ok(&output[..written])
}

pub(super) fn write_payload<T: Serialize + ?Sized>(
    value: &T,
    budget: &Budget,
    depth: usize,
    output: &mut [u8],
) -> Result<usize, Error> {
    postcard::to_slice(
        &admitted::Bounded {
            value,
            budget,
            depth,
        },
        output,
    )
    .map(|payload| payload.len())
    .map_err(|_| budget.error())
}

pub(super) fn write_scratch<T: Serialize + ?Sized>(
    value: &T,
    budget: &Budget,
    depth: usize,
    output: &mut [u8],
    capacity_kind: crate::LimitKind,
) -> Result<usize, Error> {
    write_bounded_slice::<T, true>(value, budget, depth, output, capacity_kind)
}

pub(super) fn write_bounded_slice<T: Serialize + ?Sized, const CHARGE_SCRATCH: bool>(
    value: &T,
    budget: &Budget,
    depth: usize,
    output: &mut [u8],
    capacity_kind: crate::LimitKind,
) -> Result<usize, Error> {
    let initial_len = output.len();
    let result = postcard::serialize_with_flavor(
        &admitted::Bounded {
            value,
            budget,
            depth,
        },
        BoundedSlice::<CHARGE_SCRATCH> {
            output,
            budget,
            capacity_kind,
            initial_len,
        },
    );
    budget.check()?;
    result.map_err(|_| Error::Malformed)
}

// Final output uses the meter's configured byte reservation. Additional
// staging writes debit the remaining allocation allowance.
struct BoundedSlice<'a, 'budget, const CHARGE_SCRATCH: bool> {
    output: &'a mut [u8],
    budget: &'budget Budget,
    capacity_kind: crate::LimitKind,
    initial_len: usize,
}

impl<const CHARGE_SCRATCH: bool> BoundedSlice<'_, '_, CHARGE_SCRATCH> {
    #[inline]
    fn take(&mut self, len: usize) -> postcard::Result<&mut [u8]> {
        let Some((head, rest)) = core::mem::take(&mut self.output).split_at_mut_checked(len) else {
            self.budget.refuse_with(self.capacity_kind);
            return Err(postcard::Error::SerializeBufferFull);
        };
        if CHARGE_SCRATCH && !self.budget.charge_buffer(len) {
            return Err(postcard::Error::SerializeBufferFull);
        }
        self.output = rest;
        Ok(head)
    }
}

impl<const CHARGE_SCRATCH: bool> Flavor for BoundedSlice<'_, '_, CHARGE_SCRATCH> {
    type Output = usize;

    #[inline]
    fn try_push(&mut self, byte: u8) -> postcard::Result<()> {
        self.take(1)?[0] = byte;
        Ok(())
    }

    #[inline]
    fn try_extend(&mut self, bytes: &[u8]) -> postcard::Result<()> {
        self.take(bytes.len())?.copy_from_slice(bytes);
        Ok(())
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self.initial_len - self.output.len())
    }
}

// Allocation capacity is not admission: every write is capped at preflight size.
enum Output<'a> {
    Count(usize),
    Buffer {
        bytes: &'a mut Vec<u8>,
        limit: usize,
    },
}

impl Flavor for Output<'_> {
    type Output = usize;

    fn try_push(&mut self, byte: u8) -> postcard::Result<()> {
        match self {
            Self::Count(size) => {
                *size = size
                    .checked_add(1)
                    .ok_or(postcard::Error::SerializeBufferFull)?
            }
            Self::Buffer { bytes, limit } => {
                if bytes.len() >= *limit {
                    return Err(postcard::Error::SerializeBufferFull);
                }
                bytes.push(byte);
            }
        }
        Ok(())
    }

    fn try_extend(&mut self, span: &[u8]) -> postcard::Result<()> {
        if let [byte] = span {
            return self.try_push(*byte);
        }
        match self {
            Self::Count(size) => {
                *size = size
                    .checked_add(span.len())
                    .ok_or(postcard::Error::SerializeBufferFull)?
            }
            Self::Buffer { bytes, limit } => {
                if span.len() > *limit - bytes.len() {
                    return Err(postcard::Error::SerializeBufferFull);
                }
                bytes.extend_from_slice(span);
            }
        }
        Ok(())
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(match self {
            Self::Count(size) => size,
            Self::Buffer { bytes, .. } => bytes.len(),
        })
    }
}
