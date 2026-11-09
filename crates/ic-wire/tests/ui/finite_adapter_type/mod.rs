#[derive(ic_wire::Wire)]
struct WrongFloatAdapter {
    #[serde(with = "ic_wire::serde_packed::finite_f32")]
    values: ic_wire::__private::Vec<ic_wire::FiniteF32>,
    #[serde(with = "ic_wire::serde_packed::finite_f64")]
    other: ic_wire::__private::Vec<f32>,
}
