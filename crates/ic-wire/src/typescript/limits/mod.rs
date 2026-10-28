//! Project policy values, without claiming equivalent traversal or JS heap bounds.
use super::{GenerationError, MAX_SAFE_INTEGER};
use crate::Limits;
use serde_json::{Value, json};

/// Project independent Rust request/response allowances into method metadata.
///
/// The returned object contains the six byte/node/depth properties consumed by
/// generate_registry. Merge it with consumer-owned name, mode, id and schemas.
/// Allocation is deliberately not projected: Rust admission does not bound the
/// JavaScript heap. Equal numbers do not imply equal traversal charges; qualify
/// both runtimes against the consumer corpus. Bytes include the envelope.
pub fn method_limits(request: Limits, response: Limits) -> Result<Value, GenerationError> {
    let value = json!({
        "requestBytes":request.bytes, "requestNodes":request.nodes, "requestDepth":request.depth,
        "responseBytes":response.bytes, "responseNodes":response.nodes, "responseDepth":response.depth
    });
    validate(&value)?;
    Ok(value)
}

/// Project payload-only byte/node/depth policy; zero bytes can admit unit.
/// Allocation remains native-only, as with [`method_limits`].
pub fn payload_limits(limits: Limits) -> Result<Value, GenerationError> {
    let value = json!({"bytes":limits.bytes,"nodes":limits.nodes,"depth":limits.depth});
    validate_payload(&value)?;
    Ok(value)
}

pub(super) fn validate_payload(payload: &Value) -> Result<(), GenerationError> {
    for (key, minimum) in [("bytes", 0), ("nodes", 1), ("depth", 1)] {
        allowance(&payload[key], minimum)?;
    }
    Ok(())
}

fn allowance(value: &Value, minimum: u64) -> Result<(), GenerationError> {
    let n = value.as_u64().ok_or(GenerationError::Complexity)?;
    if n > MAX_SAFE_INTEGER {
        return Err(GenerationError::Complexity);
    }
    if n < minimum {
        return Err(GenerationError::Limits);
    }
    Ok(())
}

pub(super) fn validate(method: &Value) -> Result<(), GenerationError> {
    for key in [
        "requestBytes",
        "responseBytes",
        "requestNodes",
        "responseNodes",
        "requestDepth",
        "responseDepth",
    ] {
        let minimum = if key.ends_with("Bytes") {
            crate::contract::HEADER as u64
        } else {
            1
        };
        allowance(&method[key], minimum)?;
    }
    Ok(())
}
