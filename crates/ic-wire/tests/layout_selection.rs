//! Public layout selection for the same bounded frame and logical field.
#![cfg(feature = "derive")]

use ic_wire::{Direction, Encoding, Error, Limits, Wire, evolution};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Wire)]
struct Stored {
    value: u8,
}

#[derive(Debug, PartialEq, Wire)]
#[wire(sparse)]
struct Endpoint {
    value: u8,
}

#[test]
fn positional_data_and_sparse_messages_have_distinct_explicit_contracts() {
    let limits = Limits {
        bytes: 64,
        allocation: 4096,
        nodes: 64,
        depth: 8,
    };
    let frame = Encoding::Frame {
        method: 7,
        direction: Direction::Request,
    };
    let fixed = [73, 87, 7, 0, 0, 0, 0, 0, 0, 0, 0, 42];
    let empty = [fixed.as_slice(), &[0]].concat();
    let added = [fixed.as_slice(), &[3, 1, 1, 9]].concat();
    let stored = Stored { value: 42 };
    let endpoint = Endpoint { value: 42 };

    assert_eq!(
        ic_wire::encoded_len(&stored, frame, limits),
        Ok(fixed.len())
    );
    assert_eq!(ic_wire::encode(&stored, frame, limits).unwrap(), fixed);
    assert_eq!(ic_wire::decode::<Stored>(&fixed, frame, limits), Ok(stored));
    assert_eq!(
        ic_wire::decode::<Stored>(&empty, frame, limits),
        Err(Error::Trailing)
    );
    assert_eq!(evolution::encode(&endpoint, frame, limits), Ok(empty));
    assert_eq!(
        evolution::decode::<Endpoint>(&added, frame, limits),
        Ok(endpoint)
    );
    assert_eq!(
        evolution::decode::<Endpoint>(&fixed, frame, limits),
        Err(Error::Malformed)
    );
}

#[cfg(feature = "typescript")]
#[test]
fn derive_registers_the_chosen_layout() {
    use ic_wire::typescript::ValidatedRegistry;
    for (metadata, kind) in [
        (
            ValidatedRegistry::from_type::<Stored>("Root", Default::default()),
            "record",
        ),
        (
            ValidatedRegistry::from_type::<Endpoint>("Root", Default::default()),
            "extension-record",
        ),
    ] {
        let json: serde_json::Value =
            serde_json::from_str(&metadata.unwrap().metadata_json().unwrap()).unwrap();
        assert_eq!(json["Root"]["k"], kind);
    }
}

#[cfg(feature = "typescript")]
#[test]
fn client_generation_rejects_sparse_children_inside_positional_serde() {
    use ic_wire::typescript::{GenerationError, PayloadLimits, ValidatedRegistry};

    #[derive(Serialize, Deserialize, Wire)]
    #[wire(sparse)]
    struct Child {
        number: u8,
    }

    #[derive(Wire)]
    struct Record {
        _child: Option<Child>,
    }

    #[derive(Wire)]
    struct Wrapper(#[allow(dead_code)] Child);

    for graph in [
        ValidatedRegistry::from_type::<Record>("Root", Default::default()),
        ValidatedRegistry::from_type::<Wrapper>("Root", Default::default()),
    ] {
        assert_eq!(
            graph.unwrap().payload_module(
                "codec",
                "Root",
                "./runtime.ts",
                PayloadLimits {
                    bytes: 32,
                    nodes: 32,
                    depth: 8
                },
                &Default::default(),
                |_| {}
            ),
            Err(GenerationError::Schema)
        );
    }
}
