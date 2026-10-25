//! Rendering for consumer-owned Serde metadata.
use super::{GenerationError, MAX_SAFE_INTEGER};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
    vec::Vec,
};
use serde_json::{Value, json};

pub(super) fn identifier(name: &str) -> Result<(), GenerationError> {
    // These names appear as executable declarations, unlike escaped field names.
    if name.is_empty()
        || !name
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
        || [
            "as",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "enum",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "new",
            "null",
            "return",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
            "implements",
            "interface",
            "let",
            "package",
            "private",
            "protected",
            "public",
            "static",
            "yield",
            "await",
            "any",
            "boolean",
            "bigint",
            "never",
            "number",
            "object",
            "string",
            "symbol",
            "undefined",
            "unknown",
            "eval",
            "arguments",
        ]
        .contains(&name)
    {
        return Err(GenerationError::Schema);
    }
    Ok(())
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, GenerationError> {
    value[key].as_str().ok_or(GenerationError::Schema)
}

pub(super) fn ty(
    value: &Value,
    types: &BTreeMap<String, Value>,
    depth: usize,
    remaining: &mut usize,
) -> Result<String, GenerationError> {
    if depth > 64 || *remaining == 0 {
        return Err(GenerationError::Complexity);
    }
    *remaining -= 1;
    let kind = text(value, "k")?;
    if value.get("check").is_some_and(|check| !check.is_string()) {
        return Err(GenerationError::Schema);
    }
    if matches!(kind, "bytes" | "text" | "list" | "array" | "packed-i16") {
        let bound = |key| -> Result<Option<u64>, GenerationError> {
            value
                .get(key)
                .map(|n| {
                    let n = n.as_u64().ok_or(GenerationError::Schema)?;
                    if n > MAX_SAFE_INTEGER {
                        return Err(GenerationError::Complexity);
                    }
                    Ok(n)
                })
                .transpose()
        };
        let maximum = bound("max")?;
        if matches!(kind, "array" | "packed-i16") && maximum.is_none() {
            return Err(GenerationError::Schema);
        }
        if kind == "packed-i16" && maximum.is_some_and(|n| n > MAX_SAFE_INTEGER / 2) {
            return Err(GenerationError::Complexity);
        }
        if kind == "bytes"
            && bound("min")?
                .zip(maximum)
                .is_some_and(|(min, max)| min > max)
        {
            return Err(GenerationError::Schema);
        }
    }
    if matches!(kind, "u" | "i" | "float") {
        let bound = |key| -> Result<Option<f64>, GenerationError> {
            value
                .get(key)
                .map(|bound| {
                    let n = bound
                        .as_f64()
                        .filter(|n| n.is_finite())
                        .ok_or(GenerationError::Schema)?;
                    if kind != "float" {
                        if n.fract() != 0.0 {
                            return Err(GenerationError::Schema);
                        }
                        // Runtime converts numeric integer bounds to BigInt. Reject
                        // values that JavaScript would round before that conversion.
                        if n.abs() > MAX_SAFE_INTEGER as f64 {
                            return Err(GenerationError::Complexity);
                        }
                    }
                    Ok(n)
                })
                .transpose()
        };
        if bound("min")?
            .zip(bound("max")?)
            .is_some_and(|(min, max)| min > max)
        {
            return Err(GenerationError::Schema);
        }
    }
    let mut child = |node| ty(node, types, depth + 1, remaining);
    Ok(match kind {
        "ref" => {
            let name = text(value, "name")?;
            if !types.contains_key(name) {
                return Err(GenerationError::Schema);
            }
            name.into()
        }
        "unit" => "null".into(),
        "bool" => "boolean".into(),
        "text" => "string".into(),
        // Qualify built-ins so consumer aliases cannot capture field/frame types.
        "bytes" => "globalThis.Uint8Array".into(),
        "packed-i16" => "globalThis.Int16Array".into(),
        "packed" => format!(
            "globalThis.{}",
            match text(value, "word")? {
                "i8" => "Int8Array",
                "i16" => "Int16Array",
                "u16" => "Uint16Array",
                "i32" => "Int32Array",
                "u32" => "Uint32Array",
                "i64" => "BigInt64Array",
                "u64" => "BigUint64Array",
                "hash32" => "Uint8Array",
                _ => return Err(GenerationError::Schema),
            }
        ),
        "u" | "i" => {
            let bits = value["bits"].as_u64().ok_or(GenerationError::Schema)?;
            if ![8, 16, 32, 64, 128].contains(&bits) {
                return Err(GenerationError::Schema);
            }
            if bits > 32 { "bigint" } else { "number" }.into()
        }
        "float" => {
            if !matches!(value["bits"].as_u64(), Some(32 | 64)) {
                return Err(GenerationError::Schema);
            }
            "number".into()
        }
        "newtype" => child(&value["item"])?,
        "list" | "array" => format!("globalThis.ReadonlyArray<{}>", child(&value["item"])?),
        "tuple" => format!(
            "readonly [{}]",
            value["items"]
                .as_array()
                .ok_or(GenerationError::Schema)?
                .iter()
                .map(&mut child)
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        ),
        "option" => format!("null | {{ readonly some: {} }}", child(&value["item"])?),
        "result" => format!(
            "{{ readonly ok: {} }} | {{ readonly err: {} }}",
            child(&value["ok"])?,
            child(&value["err"])?
        ),
        "record" | "enum" => {
            let is_enum = value["k"] == "enum";
            let fields = value[if is_enum { "variants" } else { "fields" }]
                .as_array()
                .ok_or(GenerationError::Schema)?;
            if is_enum && fields.is_empty() {
                return Err(GenerationError::Schema);
            }
            let mut rendered = Vec::new();
            let mut names = BTreeSet::new();
            let mut ids = BTreeSet::new();
            for field in fields {
                let name = text(field, "name")?;
                if !names.insert(name) {
                    return Err(GenerationError::Schema);
                }
                if is_enum {
                    let id = field["id"]
                        .as_u64()
                        .filter(|id| *id <= u64::from(u32::MAX))
                        .ok_or(GenerationError::Schema)?;
                    if !ids.insert(id) {
                        return Err(GenerationError::Schema);
                    }
                }
                let item = child(&field["value"])?;
                rendered.push(if is_enum {
                    format!(
                        "{{ readonly tag: {}; readonly value: {item} }}",
                        json!(name)
                    )
                } else {
                    format!("readonly {}: {item}", json!(name))
                });
            }
            if is_enum {
                rendered.join(" | ")
            } else {
                format!("{{ {} }}", rendered.join("; "))
            }
        }
        _ => return Err(GenerationError::Schema),
    })
}

// Transparent aliases must reach a concrete node. Recursive records, lists and
// enums remain valid; only references/newtypes disappear from TypeScript types.
pub(super) fn validate_aliases(types: &BTreeMap<String, Value>) -> Result<(), GenerationError> {
    let mut checked = BTreeSet::new();
    for (name, mut node) in types {
        if checked.contains(name.as_str()) {
            continue;
        }
        let mut chain = BTreeSet::from([name.as_str()]);
        loop {
            match text(node, "k")? {
                "ref" => {
                    let name = text(node, "name")?;
                    if checked.contains(name) {
                        break;
                    }
                    if !chain.insert(name) {
                        return Err(GenerationError::Schema);
                    }
                    node = types.get(name).ok_or(GenerationError::Schema)?;
                }
                "newtype" => node = &node["item"],
                _ => break,
            }
        }
        checked.extend(chain);
    }
    Ok(())
}

/// Generate a module from the native metadata grammar consumed by [`super::RUNTIME`].
///
/// [`super::native_schema`] describes supported native Serde declarations.
/// Consumers own endpoint identities, resource limits and custom scalar
/// checks. `runtime_module` selects their `createCodec` adapter (for example,
/// `./native-runtime.js`); it is JSON-escaped. Metadata is trusted configuration,
/// not untrusted input. Names, references, type shapes and generation complexity
/// are checked here; this does not admit native values or infer their bounds.
/// Each method carries `name`, `mode` (`query`/`update`) and the runtime's method
/// fields. Frame allowances must fit the header; node/depth allowances must be
/// positive. Invalid minimums return [`GenerationError::Limits`]. Named references
/// permit recursive values, but alias-only cycles return [`GenerationError::Schema`].
/// Collection bounds must be non-negative integers within JavaScript's exact
/// integer range. Arrays require `max`; byte `min` cannot exceed `max`.
/// Integer bounds must also be integral and within JavaScript's exact integer
/// range; use a named consumer check for wider bigint constraints. Numeric bounds
/// cannot be inverted. A `check` annotation must be a string naming a consumer
/// validator. Other custom scalar constraints remain consumer-owned.
pub fn generate_registry(
    types: &BTreeMap<String, Value>,
    methods: &[Value],
    runtime_module: &str,
) -> Result<String, GenerationError> {
    let mut remaining = 65536;
    let mut source = format!(
        "// Generated from IC Wire metadata; edit the source contracts.\nimport {{ createCodec as $createCodec }} from {};\n",
        json!(runtime_module)
    );
    for (name, node) in types {
        identifier(name)?;
        let rendered = ty(node, types, 0, &mut remaining)?;
        source.push_str(&format!("export type {name} = {rendered};\n"));
    }
    validate_aliases(types)?;
    // `$` is outside consumer identifiers, so helpers cannot capture exports.
    // Computed __proto__ keys preserve the own properties JSON.parse previously
    // produced. JSON escapes quotes inside strings, so this matches keys only.
    // Registry meaning and wire schemas stay unchanged; no consumer migration.
    // Serialize the borrowed registry directly instead of constructing a second
    // Value tree. JSON bytes, validation and consumer contracts are unchanged.
    let registry = serde_json::to_string(types)
        .map_err(|_| GenerationError::Schema)?
        .replace("\"__proto__\":", "[\"__proto__\"]:");
    source.push_str(&format!("const $types = {registry};\n"));
    let mut names = BTreeSet::new();
    for method in methods {
        let name = text(method, "name")?;
        identifier(name)?;
        if types.contains_key(name)
            || !names.insert(name)
            || !matches!(text(method, "mode")?, "query" | "update")
            || text(method, "id")?.parse::<u64>().is_err()
        {
            return Err(GenerationError::Schema);
        }
        super::limits::validate(method)?;
        let request = ty(&method["request"], types, 0, &mut remaining)?;
        let response = ty(&method["response"], types, 0, &mut remaining)?;
        source.push_str(&format!(
            "export const {name} = (() => {{ const codec = $createCodec({method}, $types); return {{ methodName: {name_json}, mode: {mode} as const, encodeRequest: (value: {request}): globalThis.Uint8Array => codec.encodeRequest(value), decodeRequest: (bytes: globalThis.Uint8Array): {request} => codec.decodeRequest(bytes) as {request}, encodeResponse: (value: {response}): globalThis.Uint8Array => codec.encodeResponse(value), decodeResponse: (bytes: globalThis.Uint8Array): {response} => codec.decodeResponse(bytes) as {response} }}; }})();\n",
            name_json = json!(name), mode = method["mode"],
        ));
    }
    Ok(source)
}
