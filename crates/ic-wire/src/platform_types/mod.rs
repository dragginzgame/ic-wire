//! Metadata for Platform's owned identities; Platform's Serde implementations own bytes.
// Transferred from IC Platform 421edbc's principal/account wire_schema modules.
// Only ownership changes: Principal remains bounded bytes (max 29), and Account
// keeps owner then Option<[u8; 32]>. No bytes, budgets or acceptance rules change.
// Consumers replace Platform's wire-schema feature with Wire's platform-types.

use crate::typescript::{GenerationError, SchemaContext, Wire};

impl Wire for ic_platform::Principal {
    fn native_schema(context: &mut SchemaContext) -> Result<serde_json::Value, GenerationError> {
        context.describe::<crate::Bytes<{ Self::MAX_LENGTH_IN_BYTES }>>()
    }
}

impl Wire for ic_platform::Account {
    fn native_schema(context: &mut SchemaContext) -> Result<serde_json::Value, GenerationError> {
        Ok(serde_json::json!({
            "k": "record",
            "fields": [
                {"name": "owner", "value": context.describe::<ic_platform::Principal>()?},
                {"name": "subaccount", "value": context.describe::<Option<ic_platform::Subaccount>>()?}
            ]
        }))
    }
}
