//! Installed archive's optional identity projection uses the published supplier.
use wire::typescript::{SchemaLimits, ValidatedRegistry};
use wire::{Encoding, Limits, decode, encode};
use ic_platform_types::{Account, Principal};

#[test]
fn archive_platform_projection_and_identity() {
    let graph =
        ValidatedRegistry::from_type::<Account>("Account", SchemaLimits::default()).unwrap();
    assert!(graph.metadata_json().unwrap().contains("subaccount"));
    let limits = Limits {
        bytes: 128,
        allocation: 8192,
        nodes: 64,
        depth: 16,
    };
    for subaccount in [None, Some([0; 32])] {
        let value = Account {
            owner: Principal::anonymous(),
            subaccount,
        };
        let bytes = encode(&value, Encoding::Payload, limits).unwrap();
        let decoded: Account = decode(&bytes, Encoding::Payload, limits).unwrap();
        assert_eq!(decoded.owner, value.owner);
        assert_eq!(decoded.subaccount, subaccount);
        assert_eq!(bytes.len(), if subaccount.is_some() { 36 } else { 3 });
    }
}
