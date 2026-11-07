#[derive(ic_wire::Wire)]
struct WrongBoolAdapter {
    #[serde(with = "ic_wire::serde_packed::bools")]
    value: [bool; 1],
}
