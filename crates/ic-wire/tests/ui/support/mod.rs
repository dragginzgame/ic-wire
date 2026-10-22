pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 128,
    allocation: 512,
    nodes: 64,
    depth: 16,
};
pub fn adapter(
    bytes: &[u8],
    handler: impl FnOnce(u64) -> Result<u64, u8>,
) -> Result<Vec<u8>, ic_wire::Error> {
    let request = ic_wire::native::decode_request(7, bytes, LIMITS)?;
    ic_wire::native::encode_response(7, &handler(request), LIMITS)
}
pub async fn async_adapter<F: core::future::Future<Output = Result<u64, u8>>>(
    bytes: &[u8],
    handler: impl FnOnce(u64) -> F,
) -> Result<Vec<u8>, ic_wire::Error> {
    let request = ic_wire::native::decode_request(7, bytes, LIMITS)?;
    ic_wire::native::encode_response(7, &handler(request).await, LIMITS)
}
