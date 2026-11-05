//! Host schemas for canonical Platform Types; Platform owns Serde bytes.
// Transferred from Platform commit 421edbc. Core owns this sole schema trait;
// neither a facade wrapper nor runtime selection is needed for these schemas.
use crate::typescript::{GenerationError, SchemaContext, Wire};

impl Wire for ic_platform_types::Principal {
    fn native_schema(context: &mut SchemaContext) -> Result<serde_json::Value, GenerationError> {
        context.describe::<crate::Bytes<{ Self::MAX_LENGTH_IN_BYTES }>>()
    }
}

impl Wire for ic_platform_types::Account {
    fn native_schema(context: &mut SchemaContext) -> Result<serde_json::Value, GenerationError> {
        // Types 0.17 -> 0.18 changes Some from 01 + 32 tuple bytes to
        // 01 20 + 32 bytes through serde_bytes; None remains 00. Describe exact
        // 32-byte buffers, preserving owner/subaccount order and raw None/zero.
        // Regenerate Uint8Array clients, coordinate fresh contract keys and
        // requalify all budgets. Retain original retry/hash bytes; no old reader.
        Ok(serde_json::json!({
            "k": "record",
            "fields": [
                {"name": "owner", "value": context.describe::<ic_platform_types::Principal>()?},
                {"name": "subaccount", "value": context.describe::<Option<crate::ByteArray<32>>>()?}
            ]
        }))
    }
}
