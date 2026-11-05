fn invalid() {
    let _ = ic_wire::native::encode_request::<u64>(7, &true, crate::support::LIMITS);
}
