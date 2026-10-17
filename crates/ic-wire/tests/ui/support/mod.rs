use ic_wire::{Limits, Method};
pub struct Read;
impl Method for Read {
    type Request = u64;
    type Response = Result<u64, u8>;
    const ID: u64 = 7;
    const NAME: &'static str = "handler";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 128,
        allocation: 512,
        nodes: 16,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
pub fn adapter(
    bytes: &[u8],
    handler: impl FnOnce(u64) -> Result<u64, u8>,
) -> Result<ic_wire::ResponseBytes<Read>, ic_wire::Error> {
    let request = ic_wire::decode_request::<Read>(bytes)?;
    ic_wire::encode_response::<Read>(&handler(request))
}
pub async fn async_adapter<F: core::future::Future<Output = Result<u64, u8>>>(
    bytes: &[u8],
    handler: impl FnOnce(u64) -> F,
) -> Result<ic_wire::ResponseBytes<Read>, ic_wire::Error> {
    let request = ic_wire::decode_request::<Read>(bytes)?;
    ic_wire::encode_response::<Read>(&handler(request).await)
}
