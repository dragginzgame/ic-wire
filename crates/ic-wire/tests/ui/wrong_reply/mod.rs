fn invalid() {
    let _ = ic_wire::native::encode_response::<Result<u64, u8>>(7, &7_u64, crate::support::LIMITS);
}
