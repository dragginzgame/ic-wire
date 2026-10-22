struct Outgoing(u32);
impl ic_wire::__private::serde::Serialize for Outgoing {
    fn serialize<S: ic_wire::__private::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u32(self.0)
    }
}
fn invalid() {
    let _ = ic_wire::native::decode_request::<Outgoing>(9, &[], crate::support::LIMITS);
}
