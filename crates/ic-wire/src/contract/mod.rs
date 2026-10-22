#[cfg(test)]
mod tests;

use crate::Error;
use alloc::vec::Vec;
use postcard::ser_flavors::Flavor;

pub(crate) const HEADER: usize = 11;

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
            self.equal &= if core::ptr::eq(prefix.as_ptr(), bytes.as_ptr()) {
                // Borrowed decoding can serialize the exact input region.
                // Its length and allowance were checked above; equality needs no scan.
                // Apply on every host: canonical bytes and rejection rules are unchanged.
                true
            } else if cfg!(any(target_arch = "wasm32", test)) && bytes.len() >= 8 {
                // Process 64 bytes per bulk iteration, then an 8-byte word tail.
                // Previously bulk blocks were 32 bytes. Exact equality,
                // frame bytes, admission and serializer-error precedence are unchanged;
                // no consumer migration is needed. Keep the small-write branch separate
                // to avoid increasing its Wasm instruction cost.
                if bytes.len() >= 64 {
                    equal_bulk(prefix, bytes)
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

// Keep the bulk scan out of scalar serializer paths; equality remains exact.
// The caller has already checked equal slice lengths and the byte allowance.
#[inline(never)]
fn equal_bulk(prefix: &[u8], bytes: &[u8]) -> bool {
    let (left, left_tail) = prefix.as_chunks::<64>();
    let (right, right_tail) = bytes.as_chunks::<64>();
    left.iter().zip(right).fold(0_u128, |difference, (a, b)| {
        let (a, _) = a.as_chunks::<16>();
        let (b, _) = b.as_chunks::<16>();
        difference
            | (u128::from_ne_bytes(a[0]) ^ u128::from_ne_bytes(b[0]))
            | (u128::from_ne_bytes(a[1]) ^ u128::from_ne_bytes(b[1]))
            | (u128::from_ne_bytes(a[2]) ^ u128::from_ne_bytes(b[2]))
            | (u128::from_ne_bytes(a[3]) ^ u128::from_ne_bytes(b[3]))
    }) == 0
        && {
            let (left, left_tail) = left_tail.as_chunks::<8>();
            let (right, right_tail) = right_tail.as_chunks::<8>();
            left.iter().zip(right).fold(0_u64, |difference, (a, b)| {
                difference | (u64::from_ne_bytes(*a) ^ u64::from_ne_bytes(*b))
            }) == 0
                && left_tail == right_tail
        }
}

pub(crate) fn header(id: u64, direction: u8) -> [u8; HEADER] {
    let mut header = [0; HEADER];
    header[..2].copy_from_slice(b"IW");
    header[2..10].copy_from_slice(&id.to_le_bytes());
    header[10] = direction;
    header
}
// Share fixed-header validation across Wasm codec instantiations.
#[cfg_attr(target_arch = "wasm32", inline(never))]
pub(crate) fn payload(id: u64, direction: u8, bytes: &[u8]) -> Result<&[u8], Error> {
    if bytes.len() < HEADER {
        return Err(Error::Malformed);
    }
    if &bytes[..2] != b"IW" || bytes[2..10] != id.to_le_bytes() || bytes[10] != direction {
        return Err(Error::Identity);
    }
    Ok(&bytes[HEADER..])
}
