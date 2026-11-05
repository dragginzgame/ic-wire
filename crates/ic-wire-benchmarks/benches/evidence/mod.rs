//! Shared host-only producer identities. Verifiers retain their own checks.
use serde_json::Value;
use sha2::{Digest, Sha256};
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn with_fixtures(mut identity: Value) -> Value {
    let fields = identity.as_object_mut().unwrap();
    fields.insert(
        "fixture_sha256".into(),
        hash(include_bytes!("../fixtures/mod.rs")).into(),
    );
    fields.insert(
        "hotspots_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/hotspots/mod.rs")).into(),
    );
    fields.insert(
        "bulk_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/bulk/mod.rs")).into(),
    );
    fields.insert(
        "packed_i16_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/packed_i16/mod.rs")).into(),
    );
    fields.insert(
        "toko_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/toko/mod.rs")).into(),
    );
    fields.insert(
        "derive_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/scale/mod.rs")).into(),
    );
    fields.insert(
        "byte_keys_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/byte_keys/mod.rs")).into(),
    );
    fields.insert(
        "cost_smoke_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/cost_smoke/mod.rs")).into(),
    );
    fields.insert(
        "shapes_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/shapes/mod.rs")).into(),
    );
    fields.insert(
        "followup_fixture_sha256".into(),
        hash(include_bytes!("../fixtures/followup/mod.rs")).into(),
    );
    fields.insert(
        "followup_words_sha256".into(),
        hash(include_bytes!("../fixtures/followup/words/mod.rs")).into(),
    );
    fields.insert(
        "evidence_helper_sha256".into(),
        hash(include_bytes!("mod.rs")).into(),
    );
    fields.insert(
        "operations_sha256".into(),
        hash(include_bytes!("../operations/mod.rs")).into(),
    );
    identity
}
