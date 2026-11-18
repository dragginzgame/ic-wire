const LIMITS: wire::Limits = wire::Limits {
    bytes: 65536,
    allocation: 4194304,
    nodes: 65536,
    depth: 32,
};
fn encode(value: &Value) -> Vec<u8> {
    wire::encode(value, wire::Encoding::Payload, LIMITS).unwrap()
}
fn decode(bytes: &[u8]) -> Value {
    wire::decode(bytes, wire::Encoding::Payload, LIMITS).unwrap()
}
