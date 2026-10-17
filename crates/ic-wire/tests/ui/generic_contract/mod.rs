#[derive(ic_wire::Wire)]
struct Invalid<T> {
    #[wire(id = 1)]
    value: T,
}
