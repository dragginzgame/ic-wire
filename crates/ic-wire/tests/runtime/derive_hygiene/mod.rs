//! Contract type names must not be captured by generated Serde parameters.
use ic_wire::Wire;

type D = u8;
#[allow(non_camel_case_types)]
type D_ = u16;
type A = u8;
#[allow(non_camel_case_types)]
type A_ = u16;
#[allow(non_camel_case_types)]
type __IcWireVisitorForChoice = u32;

#[derive(Debug, PartialEq, Wire)]
struct Record {
    #[wire(id = 2)]
    wide: D_,
    #[wire(id = 1)]
    small: D,
}

#[derive(Debug, PartialEq, Wire)]
enum Choice {
    #[wire(id = 1)]
    Small(A),
    #[wire(id = 2)]
    Wide(A_),
    #[wire(id = 3)]
    Visitor(__IcWireVisitorForChoice),
}

#[test]
fn generated_names_preserve_contract_types_and_bytes() {
    let record = Record {
        small: 7,
        wide: 128,
    };
    let bytes = postcard::to_allocvec(&record).unwrap();
    assert_eq!(bytes, [7, 128, 1]);
    assert_eq!(postcard::from_bytes::<Record>(&bytes).unwrap(), record);
    for (value, bytes) in [
        (Choice::Small(7), vec![1, 7]),
        (Choice::Wide(128), vec![2, 128, 1]),
        (Choice::Visitor(16384), vec![3, 128, 128, 1]),
    ] {
        assert_eq!(postcard::to_allocvec(&value).unwrap(), bytes);
        assert_eq!(postcard::from_bytes::<Choice>(&bytes).unwrap(), value);
    }
}

mod result_names {
    use ic_wire::Wire;

    struct Ok;
    struct Err;

    #[derive(Debug, PartialEq, Wire)]
    struct Record {
        #[wire(id = 1)]
        value: u8,
    }

    #[derive(Debug, PartialEq, Wire)]
    enum Choice {
        #[wire(id = 1)]
        Empty,
        #[wire(id = 2)]
        Value(u8),
    }

    #[test]
    fn result_constructors_ignore_consumer_names() {
        let _consumer_names = (Ok, Err);
        let record = Record { value: 7 };
        assert_eq!(postcard::to_allocvec(&record).unwrap(), [7]);
        assert_eq!(postcard::from_bytes::<Record>(&[7]).unwrap(), record);
        for (value, bytes) in [(Choice::Empty, vec![1]), (Choice::Value(7), vec![2, 7])] {
            assert_eq!(postcard::to_allocvec(&value).unwrap(), bytes);
            assert_eq!(postcard::from_bytes::<Choice>(&bytes).unwrap(), value);
        }
        assert!(postcard::from_bytes::<Choice>(&[3]).is_err());
    }
}

mod no_prelude {
    #![no_implicit_prelude]

    // This consumer alias must not narrow the generated enum discriminant.
    #[allow(non_camel_case_types)]
    type u32 = ::core::primitive::u8;

    #[derive(::ic_wire::Wire)]
    struct Record {
        #[wire(id = 1)]
        value: u32,
    }

    #[derive(::ic_wire::Wire)]
    enum Choice {
        #[wire(id = 300)]
        Empty,
        #[wire(id = 301)]
        Value(Record),
    }

    #[test]
    fn generated_paths_ignore_prelude_and_primitive_shadowing() {
        let bytes = ::postcard::to_allocvec(&Choice::Value(Record { value: 7 })).unwrap();
        ::std::assert_eq!(bytes, [173, 2, 7]);
        match ::postcard::from_bytes::<Choice>(&bytes).unwrap() {
            Choice::Value(value) => ::std::assert_eq!(value.value, 7),
            Choice::Empty => ::std::panic!("wrong variant"),
        }
        ::std::assert_eq!(::postcard::to_allocvec(&Choice::Empty).unwrap(), [172, 2]);
        ::std::assert!(::postcard::from_bytes::<Choice>(&[174, 2]).is_err());
    }
}

mod constant_names {
    #![allow(dead_code, non_upper_case_globals)]

    // Ordinary consumer constants must not become generated binding patterns.
    const value: u8 = 0;
    const tag: u32 = 0;
    const serializer: u8 = 0;
    const deserializer: u8 = 0;
    const seq: u8 = 0;
    const f: u8 = 0;
    const LENGTH: usize = 2;

    #[derive(Debug, PartialEq, ic_wire::Wire)]
    struct Record {
        #[wire(id = 2)]
        tag: [u8; LENGTH],
        #[wire(id = 1)]
        value: u8,
    }

    #[derive(Debug, PartialEq, ic_wire::Wire)]
    enum Choice {
        #[wire(id = 300)]
        Empty,
        #[wire(id = 301)]
        Value(<Self as Payload>::Item),
    }

    trait Payload {
        type Item;
    }

    impl Payload for Choice {
        type Item = Record;
    }

    #[test]
    fn bindings_ignore_ambient_constants() {
        let record = Record {
            value: 7,
            tag: [8, 9],
        };
        assert_eq!(postcard::to_allocvec(&record).unwrap(), [7, 8, 9]);
        assert_eq!(postcard::from_bytes::<Record>(&[7, 8, 9]).unwrap(), record);
        for (item, bytes) in [
            (Choice::Empty, vec![172, 2]),
            (Choice::Value(record), vec![173, 2, 7, 8, 9]),
        ] {
            assert_eq!(postcard::to_allocvec(&item).unwrap(), bytes);
            assert_eq!(postcard::from_bytes::<Choice>(&bytes).unwrap(), item);
        }
        assert!(postcard::from_bytes::<Choice>(&[174, 2]).is_err());
        assert!(postcard::from_bytes::<Choice>(&[173, 2]).is_err());
    }
}
