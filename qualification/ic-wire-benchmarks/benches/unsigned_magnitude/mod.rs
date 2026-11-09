//! External Nat conversion stays in this unpublished consumer qualification.
use candid::Nat;
use ic_wire::{Bytes, Error};
use num_bigint::BigUint;

// Consumer-selected maximum: 64 magnitude bytes, not a Wire-wide integer limit.
type Magnitude = Bytes<64>;

pub(crate) fn from_nat(value: &Nat) -> Result<Magnitude, Error> {
    // Check the external integer before allocating its magnitude buffer.
    if value.0.bits() > 64 * 8 {
        return Err(Error::Limit);
    }
    let bytes = if value.0.bits() == 0 {
        Vec::new()
    } else {
        value.0.to_bytes_be()
    };
    Magnitude::new(bytes)
}
pub(crate) fn to_nat(value: &Magnitude) -> Result<Nat, Error> {
    // Bytes intentionally accepts opaque bytes; numeric canonicality is checked
    // at the consumer boundary, before constructing its arithmetic type.
    if value.as_slice().first() == Some(&0) {
        return Err(Error::Malformed);
    }
    Ok(Nat(BigUint::from_bytes_be(value.as_slice())))
}

#[cfg(all(test, feature = "wire", feature = "candid"))]
mod tests;
