#[derive(ic_wire::Wire)]
struct Custom {
    #[serde(with = "custom")]
    value: u8,
}
#[derive(ic_wire::Wire)]
#[serde(untagged)]
enum Untagged {
    Value(u8),
}
#[derive(ic_wire::Wire)]
struct ExplicitId {
    #[wire(id = 0)]
    value: u8,
}
