#[derive(ic_wire::Wire)]
struct Invalid {
    #[wire(id = 1)]
    a: u64,
    #[wire(id = 1)]
    b: bool,
}
