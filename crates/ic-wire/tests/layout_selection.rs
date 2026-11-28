//! A single sparse contract for owned output, caller buffers and generated clients.
use ic_wire::{
    Direction, Encoding, Error, Limits, Wire, decode, encode, encode_into, encode_to_slice,
    encoded_len,
};
const LIMITS: Limits = Limits {
    bytes: 64,
    allocation: 65536,
    nodes: 128,
    depth: 16,
};

#[derive(Debug, PartialEq, Wire)]
struct Child {
    value: u8,
}
#[derive(Debug, PartialEq, Wire)]
pub(crate) enum Choice {
    None,
    Item(Child),
    Named {
        child: Child,
        #[wire(id = 7)]
        note: Option<u8>,
    },
    Pair(u8, Child),
}
#[derive(Debug, PartialEq, Wire)]
struct Root {
    choice: Choice,
}

#[test]
fn every_output_projects_the_same_sparse_record_and_enum_payloads() {
    let encoding = Encoding::Frame {
        method: 7,
        direction: Direction::Request,
    };
    let header = [73, 87, 7, 0, 0, 0, 0, 0, 0, 0, 0];
    for (choice, payload) in [
        (Choice::None, vec![0, 0]),
        (Choice::Item(Child { value: 42 }), vec![1, 42, 0, 0]),
        (
            Choice::Named {
                child: Child { value: 42 },
                note: Some(5),
            },
            vec![2, 42, 0, 3, 7, 1, 5, 0],
        ),
        (
            Choice::Pair(7, Child { value: 42 }),
            vec![3, 7, 42, 0, 0, 0],
        ),
    ] {
        let root = Root { choice };
        let frame = [header.as_slice(), payload.as_slice()].concat();
        assert_eq!(encoded_len(&root, encoding, LIMITS), Ok(frame.len()));
        assert_eq!(encode(&root, encoding, LIMITS).unwrap(), frame);
        assert_eq!(
            encode_into(&root, encoding, LIMITS, &mut Vec::new()).unwrap(),
            frame
        );
        assert_eq!(
            encode_to_slice(&root, encoding, LIMITS, &mut [0; 64]).unwrap(),
            frame
        );
        assert_eq!(decode::<Root>(&frame, encoding, LIMITS), Ok(root));
    }
    assert_eq!(
        decode::<Root>(&[1, 42], Encoding::Payload, LIMITS),
        Err(Error::Malformed)
    );
}

#[cfg(feature = "typescript")]
#[test]
fn generated_clients_follow_nested_enum_payloads() {
    use ic_wire::typescript::{PayloadLimits, ValidatedRegistry};
    let graph = ValidatedRegistry::from_type::<Root>("Root", Default::default()).unwrap();
    graph
        .payload_module(
            "codec",
            "Root",
            "./runtime.ts",
            PayloadLimits {
                bytes: 64,
                nodes: 128,
                depth: 16,
            },
            &Default::default(),
            |_| {},
        )
        .unwrap();
}

#[test]
fn generic_enum_payloads_can_borrow_from_the_input() {
    use ic_wire::{Message, TextRef};
    #[derive(Debug, PartialEq, Wire)]
    enum Entry<'a, T, const N: usize> {
        Found { label: TextRef<'a, N>, value: T },
    }
    let entry = Message(Entry::Found {
        label: TextRef::<16>::new("item").unwrap(),
        value: 7u8,
    });
    let bytes = encode(&entry, Encoding::Payload, LIMITS).unwrap();
    let decoded: Message<Entry<'_, u8, 16>> = decode(&bytes, Encoding::Payload, LIMITS).unwrap();
    assert_eq!(decoded, entry);
    let Entry::Found { label, .. } = decoded.0;
    assert_eq!(label.as_str().as_ptr(), bytes[2..].as_ptr());
}

#[test]
fn public_unit_enum_has_a_borrow_independent_payload() {
    #[derive(Debug, PartialEq, Wire)]
    pub enum Status {
        Ready,
        Paused,
    }
    let message = ic_wire::Message(Status::Paused);
    let bytes = encode(&message, Encoding::Payload, LIMITS).unwrap();
    assert_eq!(bytes, [1, 0]);
    assert_eq!(
        decode::<ic_wire::Message<Status>>(&bytes, Encoding::Payload, LIMITS).unwrap(),
        message
    );
}
