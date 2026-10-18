#[derive(ic_wire::Wire)]
enum Recursive {
    #[wire(id = 0)]
    Next(Box<Recursive>),
    #[wire(id = 1)]
    End,
}

const _: ic_wire::Shape = <Recursive as ic_wire::Wire>::SHAPE;
