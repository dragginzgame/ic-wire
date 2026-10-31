//! Research transport envelope only: opaque original Wire bytes roundtrip unchanged.
//! This is not a proposed Wire codec, frame or acceptance-rule change.
pub const MAX: usize = 2 * 1024 * 1024;
pub fn compress(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() <= MAX);
    lz4_flex::block::compress_prepend_size(bytes)
}
pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    let prefix: [u8; 4] = bytes.get(..4).ok_or("prefix")?.try_into().unwrap();
    let size = u32::from_le_bytes(prefix) as usize;
    if size > MAX || bytes.len() > MAX + 65536 { return Err("limit"); }
    let mut output = vec![0; size];
    let written = lz4_flex::block::decompress_into(&bytes[4..], &mut output).map_err(|_| "block")?;
    if written != size { return Err("length"); }
    Ok(output)
}
