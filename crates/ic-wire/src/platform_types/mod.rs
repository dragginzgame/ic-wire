//! Metadata for Platform's owned identities; Platform's Serde implementations own bytes.
// Transferred from IC Platform 421edbc's principal/account wire_schema modules.
// Only ownership changes: Principal remains bounded bytes (max 29), and Account
// keeps owner then Option<[u8; 32]>. No bytes, budgets or acceptance rules change.
// The stable identity owner replaces the moving runtime facade; Platform 0.15
// reexports these exact types. Schemas/Serde bytes and admission stay unchanged.
// Consumers replace Platform's wire-schema feature with Wire's platform-types.

// Types 0.1 -> 0.15 changes Rust identity only. Consumers align suppliers on
// this core minor; schema shapes, Serde bytes, budgets and refusals stay unchanged.

use crate::typescript::{GenerationError, SchemaContext, Wire};

impl Wire for ic_platform_types::Principal {
    fn native_schema(context: &mut SchemaContext) -> Result<serde_json::Value, GenerationError> {
        context.describe::<crate::Bytes<{ Self::MAX_LENGTH_IN_BYTES }>>()
    }
}

impl Wire for ic_platform_types::Account {
    fn native_schema(context: &mut SchemaContext) -> Result<serde_json::Value, GenerationError> {
        Ok(serde_json::json!({
            "k": "record",
            "fields": [
                {"name": "owner", "value": context.describe::<ic_platform_types::Principal>()?},
                {"name": "subaccount", "value": context.describe::<Option<ic_platform_types::Subaccount>>()?}
            ]
        }))
    }
}
