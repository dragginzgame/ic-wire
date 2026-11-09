pub fn roundtrip() -> Result<u64, ic_wire::Error> {
    let limits = ic_wire::Limits {
        bytes: 16,
        allocation: 1024,
        nodes: 8,
        depth: 8,
    };
    let bytes = ic_wire::native::encode(&128_u64, limits)?;
    ic_wire::native::decode(&bytes, limits)
}
