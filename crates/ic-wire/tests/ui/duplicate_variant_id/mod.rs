#[derive(ic_wire::Wire)]
enum Invalid {
    #[wire(id = 1)]
    A,
    #[wire(id = 1)]
    B,
}
