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
