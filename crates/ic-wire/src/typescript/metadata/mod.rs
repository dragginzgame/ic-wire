//! Rendering shared by finite schemas and consumer-owned native metadata.
use super::GenerationError;
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
    vec::Vec,
};
use serde_json::{Value, json};

fn identifier(name: &str) -> Result<(), GenerationError> {
    // These names appear as executable declarations, unlike escaped field names.
    if name.is_empty()
        || !name
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
        || [
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
            "JSON",
            "types",
            "createCodec",
            "ReadonlyArray",
            "Uint8Array",
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
    let mut child = |node| ty(node, types, depth + 1, remaining);
    Ok(match text(value, "k")? {
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
        "bytes" => "Uint8Array".into(),
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
        "list" | "array" => {
            // Lists may be dynamically bounded; arrays require an exact length.
            if value["k"] == "array" {
                let length = value["max"].as_u64().ok_or(GenerationError::Schema)?;
                if length > (1_u64 << 53) - 1 {
                    return Err(GenerationError::Complexity);
                }
            }
            format!("ReadonlyArray<{}>", child(&value["item"])?)
        }
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
            for (index, field) in fields.iter().enumerate() {
                let name = text(field, "name")?;
                if fields[..index]
                    .iter()
                    .any(|f| f["name"] == name || (is_enum && f["id"] == field["id"]))
                {
                    return Err(GenerationError::Schema);
                }
                if is_enum
                    && field["id"]
                        .as_u64()
                        .is_none_or(|id| id > u64::from(u32::MAX))
                {
                    return Err(GenerationError::Schema);
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
fn validate_aliases(types: &BTreeMap<String, Value>) -> Result<(), GenerationError> {
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
/// Consumers own reflection, endpoint identities, resource limits and custom scalar
/// checks. `runtime_module` selects their `createCodec` adapter (for example,
/// `./native-runtime.js`); it is JSON-escaped. Metadata is trusted configuration,
/// not untrusted input. Names, references, type shapes and generation complexity
/// are checked here; this does not admit native values or infer their bounds.
/// Each method carries `name`, `mode` (`query`/`update`) and the runtime's method
/// fields. Frame allowances must fit the header; node/depth allowances must be
/// positive. Invalid minimums return [`GenerationError::Limits`]. Named references
/// permit recursive values, but alias-only cycles return [`GenerationError::Schema`].
/// Fixed arrays require a non-negative integer `max` within JavaScript's exact
/// integer range. Custom constraints remain consumer/runtime-owned.
pub fn generate_registry(
    types: &BTreeMap<String, Value>,
    methods: &[Value],
    runtime_module: &str,
) -> Result<String, GenerationError> {
    let mut remaining = 65536;
    let mut source = format!(
        "// Generated from IC Wire metadata; edit the source contracts.\nimport {{ createCodec }} from {};\n",
        json!(runtime_module)
    );
    for (name, node) in types {
        identifier(name)?;
        let rendered = ty(node, types, 0, &mut remaining)?;
        source.push_str(&format!("export type {name} = {rendered};\n"));
    }
    validate_aliases(types)?;
    // JSON.parse preserves an own __proto__ entry; an object literal would
    // interpret that valid Serde name as a prototype mutation instead.
    source.push_str(&format!(
        "const types = JSON.parse({});\n",
        json!(json!(types).to_string())
    ));
    for (index, method) in methods.iter().enumerate() {
        let name = text(method, "name")?;
        identifier(name)?;
        if types.contains_key(name)
            || methods[..index].iter().any(|m| m["name"] == name)
            || !matches!(text(method, "mode")?, "query" | "update")
            || text(method, "id")?.parse::<u64>().is_err()
        {
            return Err(GenerationError::Schema);
        }
        for key in [
            "requestBytes",
            "responseBytes",
            "requestNodes",
            "responseNodes",
            "requestDepth",
            "responseDepth",
        ] {
            let n = method[key].as_u64().ok_or(GenerationError::Complexity)?;
            if n > (1_u64 << 53) - 1 {
                return Err(GenerationError::Complexity);
            }
            let minimum = if key.ends_with("Bytes") {
                crate::contract::HEADER as u64
            } else {
                1
            };
            if n < minimum {
                return Err(GenerationError::Limits);
            }
        }
        let request = ty(&method["request"], types, 0, &mut remaining)?;
        let response = ty(&method["response"], types, 0, &mut remaining)?;
        source.push_str(&format!(
            "export const {name} = (() => {{ const codec = createCodec({method}, types); return {{ methodName: {name_json}, mode: {mode} as const, encodeRequest: (value: {request}): Uint8Array => codec.encodeRequest(value), decodeRequest: (bytes: Uint8Array): {request} => codec.decodeRequest(bytes) as {request}, encodeResponse: (value: {response}): Uint8Array => codec.encodeResponse(value), decodeResponse: (bytes: Uint8Array): {response} => codec.decodeResponse(bytes) as {response} }}; }})();\n",
            name_json = json!(name), mode = method["mode"],
        ));
    }
    Ok(source)
}
