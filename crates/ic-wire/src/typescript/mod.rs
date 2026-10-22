//! Host-side TypeScript projection from the same schema used by Rust derives.
use crate::{Method, Wire, schema::Schema};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use serde_json::{Value, json};

mod metadata;
pub use metadata::generate_registry;

// JavaScript's safe-integer ceiling for generated numeric bounds.
const MAX_SAFE_INTEGER: u64 = (1_u64 << 53) - 1;

/// Shared bounded Postcard implementation. Write beside generated method modules
/// as `wire-runtime.ts`; it imports no SDK and owns no transport/auth policy.
pub const RUNTIME: &str = include_str!("runtime.ts");

/// A contract cannot be faithfully projected into a TypeScript module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationError {
    /// The method's static resource admission failed.
    Limits,
    /// Missing/custom metadata, duplicate identities, or unsupported widths.
    Schema,
    /// Metadata is too large, too deep, or outside exact JavaScript integer range.
    Complexity,
}

fn describe(
    schema: &Schema,
    depth: usize,
    remaining: &mut usize,
) -> Result<Value, GenerationError> {
    if depth > 64 || *remaining == 0 {
        return Err(GenerationError::Complexity);
    }
    *remaining -= 1;
    // Every variant needs a child node; refuse before allocating/sorting its index.
    if let Schema::Enum(variants) = schema
        && variants.len() > *remaining
    {
        return Err(GenerationError::Complexity);
    }
    let mut child = |schema| describe(schema, depth + 1, remaining);
    Ok(match schema {
        Schema::Opaque => return Err(GenerationError::Schema),
        Schema::Unit => json!({"k":"unit"}),
        Schema::Bool => json!({"k":"bool"}),
        Schema::Float(bits) => json!({"k":"float","bits":bits}),
        Schema::Unsigned(bits) | Schema::Signed(bits) => {
            json!({"k":if matches!(schema, Schema::Signed(_)) {"i"} else {"u"},"bits":bits})
        }
        Schema::Bytes(max) | Schema::Text(max) => {
            exact(*max)?;
            json!({"k":if matches!(schema, Schema::Bytes(_)) {"bytes"} else {"text"},"max":max})
        }
        Schema::List(value, max) | Schema::Array(value, max) => {
            exact(*max)?;
            json!({"k":if matches!(schema, Schema::List(..)) {"list"} else {"array"},"max":max,"item":child(value)?})
        }
        Schema::Option(value) => json!({"k":"option","item":child(value)?}),
        Schema::Result(ok, err) => json!({"k":"result","ok":child(ok)?,"err":child(err)?}),
        Schema::Record(fields) => {
            let mut nodes = Vec::new();
            for (index, field) in fields.iter().enumerate() {
                // Name uniqueness is checked by the shared metadata validator.
                if index > 0 && fields[index - 1].id >= field.id {
                    return Err(GenerationError::Schema);
                }
                nodes.push(json!({"name":field.name,"id":field.id,"value":child(field.value)?}));
            }
            json!({"k":"record","fields":nodes})
        }
        Schema::Enum(variants) => {
            let mut variants: Vec<_> = variants.iter().collect();
            variants.sort_by_key(|v| v.id);
            let mut nodes = Vec::new();
            // Empty enums and duplicate names/tags are rejected by metadata::ty.
            for variant in variants {
                nodes.push(
                    json!({"name":variant.name,"id":variant.id,"value":child(variant.value)?}),
                );
            }
            json!({"k":"enum","variants":nodes})
        }
    })
}

fn exact(value: usize) -> Result<(), GenerationError> {
    if value as u128 > u128::from(MAX_SAFE_INTEGER) {
        Err(GenerationError::Complexity)
    } else {
        Ok(())
    }
}

/// Generate one typed method module. Write RUNTIME as `wire-runtime.ts` in the
/// same directory. Source field/variant names are JSON-escaped, never interpolated
/// as executable identifiers. Custom metadata is trusted like custom Wire impls.
pub fn generate<M: Method>() -> Result<String, GenerationError> {
    let request_bytes = crate::contract::admit::<M::Request>(M::REQUEST_LIMITS)
        .map_err(|_| GenerationError::Limits)?;
    let response_bytes = crate::contract::admit::<M::Response>(M::RESPONSE_LIMITS)
        .map_err(|_| GenerationError::Limits)?;
    for n in [
        request_bytes,
        response_bytes,
        M::REQUEST_LIMITS.nodes,
        M::REQUEST_LIMITS.depth,
        M::RESPONSE_LIMITS.nodes,
        M::RESPONSE_LIMITS.depth,
    ] {
        exact(n)?;
    }
    let request = describe(&M::Request::SCHEMA, 0, &mut 4096)?;
    let request_ty = metadata::ty(&request, &Default::default(), 0, &mut 4096)?;
    let response = describe(&M::Response::SCHEMA, 0, &mut 4096)?;
    let response_ty = metadata::ty(&response, &Default::default(), 0, &mut 4096)?;
    let metadata = json!({"id":M::ID.to_string(),"request":request,"response":response,"requestBytes":request_bytes,"responseBytes":response_bytes,"requestNodes":M::REQUEST_LIMITS.nodes,"responseNodes":M::RESPONSE_LIMITS.nodes,"requestDepth":M::REQUEST_LIMITS.depth,"responseDepth":M::RESPONSE_LIMITS.depth});
    Ok(format!(
        "// Generated from IC Wire schema; edit the Rust contract, not this file.\nimport {{ createCodec }} from './wire-runtime.js';\nexport type Request = {request_ty};\nexport type Response = {response_ty};\nexport const methodName = {};\nconst codec = createCodec({metadata});\nexport const encodeRequest = (value: Request): Uint8Array => codec.encodeRequest(value);\nexport const decodeRequest = (bytes: Uint8Array): Request => codec.decodeRequest(bytes) as Request;\nexport const encodeResponse = (value: Response): Uint8Array => codec.encodeResponse(value);\nexport const decodeResponse = (bytes: Uint8Array): Response => codec.decodeResponse(bytes) as Response;\n",
        json!(M::NAME)
    ))
}
