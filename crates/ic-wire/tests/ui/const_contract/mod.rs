#[derive(ic_wire::Wire)]
struct Invalid<const N: usize> {
    #[wire(id = 1)]
    value: ic_wire::Bytes<N>,
}
