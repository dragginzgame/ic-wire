//! Shared host-only producer identities. Verifiers retain their own checks.
use serde_json::Value;
use sha2::{Digest, Sha256};
pub fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
pub fn with_fixtures(mut identity: Value) -> Value {
    let fields = identity.as_object_mut().unwrap();
    fields.insert(
        "candid_decoder".into(),
        "upstream-exact-one-no-platform-nesting-prescan".into(),
    );
    macro_rules! sources {
        ($($key:literal => $source:literal),* $(,)?) => {
            $(fields.insert($key.into(), hash(include_bytes!($source)).into());)*
        };
    }
    sources! {
        "fixture_sha256" => "../fixtures/mod.rs",
        "hotspots_fixture_sha256" => "../fixtures/hotspots/mod.rs",
        "signed32_fixture_sha256" => "../fixtures/signed32/mod.rs",
        "bulk_fixture_sha256" => "../fixtures/bulk/mod.rs",
        "packed_i16_fixture_sha256" => "../fixtures/packed_i16/mod.rs",
        "toko_fixture_sha256" => "../fixtures/toko/mod.rs",
        "derive_fixture_sha256" => "../fixtures/scale/mod.rs",
        "byte_keys_fixture_sha256" => "../fixtures/byte_keys/mod.rs",
        "id16_fixture_sha256" => "../fixtures/id16/mod.rs",
        "packed_source_sha256" => "../../../../crates/ic-wire/src/packed/mod.rs",
        "cost_smoke_fixture_sha256" => "../fixtures/cost_smoke/mod.rs",
        "shapes_fixture_sha256" => "../fixtures/shapes/mod.rs",
        "followup_fixture_sha256" => "../fixtures/followup/mod.rs",
        "container_fixture_sha256" => "../fixtures/followup/containers/mod.rs",
        "container_transport_sha256" => "../fixtures/followup/containers/transport/mod.rs",
        "container_adapter_sha256" => "../fixtures/followup/containers/transport/adapters/mod.rs",
        "followup_words_sha256" => "../fixtures/followup/words/mod.rs",
        "evidence_helper_sha256" => "mod.rs",
        "public_bools_adapter_sha256" => "../../../../crates/ic-wire/src/serde_packed/bools/mod.rs",
        "public_finite_adapter_sha256" => "../../../../crates/ic-wire/src/serde_packed/finite/mod.rs",
        "public_finite_f32_sha256" => "../../../../crates/ic-wire/src/serde_packed/finite_f32/mod.rs",
        "public_finite_f64_sha256" => "../../../../crates/ic-wire/src/serde_packed/finite_f64/mod.rs",
        "public_nonzero_adapter_sha256" => "../../../../crates/ic-wire/src/serde_packed/nonzero_i64/mod.rs",
        "public_heap_adapter_sha256" => "../../../../crates/ic-wire/src/serde_packed/heap_u64/mod.rs",
        "operations_sha256" => "../operations/mod.rs",
    }
    identity
}
