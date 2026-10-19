use ic_wire::{Limits, Method, Wire};
struct Outgoing(u32);
impl ic_wire::__private::serde::Serialize for Outgoing {
    fn serialize<S: ic_wire::__private::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u32(self.0)
    }
}
impl Wire for Outgoing {
    const SHAPE: ic_wire::Shape = <u32 as Wire>::SHAPE;
}
struct Send;
impl Method for Send {
    type Request = Outgoing;
    type Response = ();
    const ID: u64 = 9;
    const NAME: &'static str = "send";
    const REQUEST_LIMITS: Limits = super::support::Read::REQUEST_LIMITS;
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
fn invalid() {
    let _ = ic_wire::decode_request::<Send>(&[]);
}
