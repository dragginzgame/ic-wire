#[derive(ic_wire::Wire)]
struct Invalid {
    #[wire(id = 1)]
    name: String,
}
