//! Host-side TypeScript projection from the same schema used by Rust derives.
use crate::{Method, Wire, schema::Schema};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use serde_json::{Value, json};

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
) -> Result<(Value, String), GenerationError> {
    if depth > 64 || *remaining == 0 {
        return Err(GenerationError::Complexity);
    }
    *remaining -= 1;
    let mut child = |schema| describe(schema, depth + 1, remaining);
    Ok(match schema {
        Schema::Opaque => return Err(GenerationError::Schema),
        Schema::Unit => (json!({"k":"unit"}), "null".into()),
        Schema::Bool => (json!({"k":"bool"}), "boolean".into()),
        Schema::Float(bits) => {
            if ![32, 64].contains(bits) {
                return Err(GenerationError::Schema);
            }
            (json!({"k":"float","bits":bits}), "number".into())
        }
        Schema::Unsigned(bits) | Schema::Signed(bits) => {
            if ![8, 16, 32, 64, 128].contains(bits) {
                return Err(GenerationError::Schema);
            }
            (
                json!({"k":if matches!(schema, Schema::Signed(_)) {"i"} else {"u"},"bits":bits}),
                if *bits <= 32 { "number" } else { "bigint" }.into(),
            )
        }
        Schema::Bytes(max) | Schema::Text(max) => {
            exact(*max)?;
            (
                json!({"k":if matches!(schema, Schema::Bytes(_)) {"bytes"} else {"text"},"max":max}),
                if matches!(schema, Schema::Bytes(_)) {
                    "Uint8Array"
                } else {
                    "string"
                }
                .into(),
            )
        }
        Schema::List(value, max) | Schema::Array(value, max) => {
            exact(*max)?;
            let (node, ty) = child(value)?;
            (
                json!({"k":if matches!(schema, Schema::List(..)) {"list"} else {"array"},"max":max,"item":node}),
                format!("ReadonlyArray<{ty}>"),
            )
        }
        Schema::Option(value) => {
            let (node, ty) = child(value)?;
            (
                json!({"k":"option","item":node}),
                format!("null | {{ readonly some: {ty} }}"),
            )
        }
        Schema::Result(ok, err) => {
            let (ok_node, ok_ty) = child(ok)?;
            let (err_node, err_ty) = child(err)?;
            (
                json!({"k":"result","ok":ok_node,"err":err_node}),
                format!("{{ readonly ok: {ok_ty} }} | {{ readonly err: {err_ty} }}"),
            )
        }
        Schema::Record(fields) => {
            let mut nodes = Vec::new();
            let mut types = Vec::new();
            for (index, field) in fields.iter().enumerate() {
                if fields[..index]
                    .iter()
                    .any(|f| f.name == field.name || f.id >= field.id)
                {
                    return Err(GenerationError::Schema);
                }
                let (node, ty) = child(field.value)?;
                nodes.push(json!({"name":field.name,"id":field.id,"value":node}));
                types.push(format!("readonly {}: {ty}", json!(field.name)));
            }
            (
                json!({"k":"record","fields":nodes}),
                format!("{{ {} }}", types.join("; ")),
            )
        }
        Schema::Enum(variants) => {
            if variants.is_empty() {
                return Err(GenerationError::Schema);
            }
            let mut variants: Vec<_> = variants.iter().collect();
            variants.sort_by_key(|v| v.id);
            let mut nodes = Vec::new();
            let mut types = Vec::new();
            for (index, variant) in variants.iter().enumerate() {
                if variants[..index]
                    .iter()
                    .any(|v| v.name == variant.name || v.id == variant.id)
                {
                    return Err(GenerationError::Schema);
                }
                let (node, ty) = child(variant.value)?;
                nodes.push(json!({"name":variant.name,"id":variant.id,"value":node}));
                types.push(format!(
                    "{{ readonly tag: {}; readonly value: {ty} }}",
                    json!(variant.name)
                ));
            }
            (json!({"k":"enum","variants":nodes}), types.join(" | "))
        }
    })
}

fn exact(value: usize) -> Result<(), GenerationError> {
    if value as u128 > (1_u128 << 53) - 1 {
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
    let (request, request_ty) = describe(&M::Request::SCHEMA, 0, &mut 4096)?;
    let (response, response_ty) = describe(&M::Response::SCHEMA, 0, &mut 4096)?;
    let metadata = json!({"id":M::ID.to_string(),"request":request,"response":response,"requestBytes":request_bytes,"responseBytes":response_bytes,"requestNodes":M::REQUEST_LIMITS.nodes,"responseNodes":M::RESPONSE_LIMITS.nodes,"requestDepth":M::REQUEST_LIMITS.depth,"responseDepth":M::RESPONSE_LIMITS.depth});
    Ok(format!(
        "// Generated from IC Wire schema; edit the Rust contract, not this file.\nimport {{ createCodec }} from './wire-runtime.js';\nexport type Request = {request_ty};\nexport type Response = {response_ty};\nexport const methodName = {};\nconst codec = createCodec({metadata});\nexport const encodeRequest = (value: Request): Uint8Array => codec.encodeRequest(value);\nexport const decodeRequest = (bytes: Uint8Array): Request => codec.decodeRequest(bytes) as Request;\nexport const encodeResponse = (value: Response): Uint8Array => codec.encodeResponse(value);\nexport const decodeResponse = (bytes: Uint8Array): Response => codec.decodeResponse(bytes) as Response;\n",
        json!(M::NAME)
    ))
}
