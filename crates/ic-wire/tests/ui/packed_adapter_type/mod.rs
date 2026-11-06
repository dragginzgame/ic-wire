#[derive(ic_wire::__private::serde::Serialize, ic_wire::Wire)]
#[serde(crate = "ic_wire::__private::serde")]
struct WrongPackedAdapter {
    #[serde(with = "ic_wire::serde_packed::nonzero_i64")]
    value: [core::num::NonZeroI64; 1],
}
