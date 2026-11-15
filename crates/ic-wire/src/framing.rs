use crate::Error;

/// Interpretation of the input byte span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Encoding {
    /// A Postcard payload without a frame header.
    Payload,
    /// An 11-byte Wire header followed by a Postcard payload.
    Frame {
        /// Expected routing identity.
        method: u64,
        /// Expected request or response direction.
        direction: Direction,
    },
}

/// Direction carried by the last header byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// Request, encoded as zero.
    Request,
    /// Response, encoded as one.
    Response,
}

pub(crate) fn validate_header(
    method: u64,
    direction: Direction,
    header: &[u8; 11],
) -> Result<(), Error> {
    let direction = match direction {
        Direction::Request => 0,
        Direction::Response => 1,
    };
    if header[..2] != *b"IW" || header[2..10] != method.to_le_bytes() || header[10] != direction {
        return Err(Error::Identity);
    }
    Ok(())
}

/// Derive a routing ID from the exact exported method name.
///
/// Uses FNV-1a-64 over `ic-wire:method:` followed by the name's UTF-8 bytes,
/// modulo 2^64. No case folding or Unicode normalization is performed.
/// This is not authentication or a schema fingerprint. Callers must reject
/// duplicate IDs; explicit IDs remain valid and need not use this derivation.
#[must_use]
pub const fn method_id(name: &str) -> u64 {
    let parts: [&[u8]; 2] = [b"ic-wire:method:", name.as_bytes()];
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut part = 0;
    while part < parts.len() {
        let mut index = 0;
        while index < parts[part].len() {
            hash = (hash ^ parts[part][index] as u64).wrapping_mul(0x0100_0000_01b3);
            index += 1;
        }
        part += 1;
    }
    hash
}

/// Admit an exact 11-byte request header before copying its payload.
///
/// Requires ASCII `IW`, the expected method in little-endian order, and request
/// direction zero. This allocates nothing and examines no payload bytes.
/// Success does not authenticate the sender or admit payload resources/schema.
///
/// # Errors
///
/// Returns [`Error::Identity`] for any marker, method or direction mismatch.
/// The fixed-size argument excludes truncated headers at this API boundary.
pub fn validate_request_header(method: u64, header: &[u8; 11]) -> Result<(), Error> {
    validate_header(method, Direction::Request, header)
}
