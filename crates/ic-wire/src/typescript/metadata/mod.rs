//! Rendering for consumer-owned Serde metadata.
use super::{GenerationError, MAX_SAFE_INTEGER, SCHEMA_DEPTH, SCHEMA_WORK};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
};
use core::fmt::Write as _;
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
    render: bool,
) -> Result<String, GenerationError> {
    if depth > SCHEMA_DEPTH || *remaining == 0 {
        return Err(GenerationError::Complexity);
    }
    *remaining -= 1;
    let kind = text(value, "k")?;
    // Only the qualified NonZeroI64 adapter emits this constraint. Ordinary
    // packed values still accept zero; regeneration is required on adoption.
    if value.get("nonzero").is_some_and(|nonzero| {
        kind != "packed" || value["word"] != "i64" || nonzero != &Value::Bool(true)
    }) {
        return Err(GenerationError::Schema);
    }
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
    // A runtime flag keeps one traversal/validation owner without specializing
    // the walker. Validation visits every child but allocates no type strings.
    macro_rules! output {
        ($($tokens:tt)*) => {
            if render { format!($($tokens)*) } else { String::new() }
        };
    }
    let mut child = |node| ty(node, types, depth + 1, remaining, render);
    Ok(match kind {
        "ref" => {
            let name = text(value, "name")?;
            if !types.contains_key(name) {
                return Err(GenerationError::Schema);
            }
            output!("{name}")
        }
        "unit" => output!("null"),
        "bool" => output!("boolean"),
        "text" => output!("string"),
        // Qualify built-ins so consumer aliases cannot capture field/frame types.
        "bytes" => output!("globalThis.Uint8Array"),
        "packed-i16" => output!("globalThis.Int16Array"),
        "packed" if value["word"] == "bool" => output!("globalThis.Array<boolean>"),
        "packed" => {
            let array = match text(value, "word")? {
                "i8" => "Int8Array",
                "i16" => "Int16Array",
                "u16" => "Uint16Array",
                "i32" => "Int32Array",
                "u32" => "Uint32Array",
                "i64" => "BigInt64Array",
                "u64" => "BigUint64Array",
                "finite-f32" => "Float32Array",
                "finite-f64" => "Float64Array",
                "hash16" | "hash32" => "Uint8Array",
                _ => return Err(GenerationError::Schema),
            };
            output!("globalThis.{array}")
        }
        "u" | "i" => {
            let bits = value["bits"].as_u64().ok_or(GenerationError::Schema)?;
            if ![8, 16, 32, 64, 128].contains(&bits) {
                return Err(GenerationError::Schema);
            }
            let scalar = if bits > 32 { "bigint" } else { "number" };
            output!("{scalar}")
        }
        "float" => {
            if !matches!(value["bits"].as_u64(), Some(32 | 64)) {
                return Err(GenerationError::Schema);
            }
            output!("number")
        }
        "newtype" => child(&value["item"])?,
        "list" | "array" => {
            let item = child(&value["item"])?;
            output!("globalThis.ReadonlyArray<{item}>")
        }
        "tuple" => {
            let mut rendered = String::new();
            if render {
                rendered.push_str("readonly [");
            }
            for (index, item) in value["items"]
                .as_array()
                .ok_or(GenerationError::Schema)?
                .iter()
                .enumerate()
            {
                let item = child(item)?;
                if render {
                    if index != 0 {
                        rendered.push_str(", ");
                    }
                    rendered.push_str(&item);
                }
            }
            if render {
                rendered.push(']');
            }
            rendered
        }
        "option" => {
            let item = child(&value["item"])?;
            output!("null | {{ readonly some: {item} }}")
        }
        "result" => {
            let ok = child(&value["ok"])?;
            let err = child(&value["err"])?;
            output!("{{ readonly ok: {ok} }} | {{ readonly err: {err} }}")
        }
        "record" | "enum" => {
            let is_enum = value["k"] == "enum";
            let fields = value[if is_enum { "variants" } else { "fields" }]
                .as_array()
                .ok_or(GenerationError::Schema)?;
            if is_enum && fields.is_empty() {
                return Err(GenerationError::Schema);
            }
            let mut rendered = String::new();
            if render && !is_enum {
                rendered.push_str("{ ");
            }
            let mut names = BTreeSet::new();
            let mut ids = BTreeSet::new();
            for (index, field) in fields.iter().enumerate() {
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
                if render {
                    if index != 0 {
                        rendered.push_str(if is_enum { " | " } else { "; " });
                    }
                    if is_enum {
                        write!(
                            rendered,
                            "{{ readonly tag: {}; readonly value: {item} }}",
                            json!(name)
                        )
                        .expect("writing to String cannot fail");
                    } else {
                        write!(rendered, "readonly {}: {item}", json!(name))
                            .expect("writing to String cannot fail");
                    }
                }
            }
            if render && !is_enum {
                rendered.push_str(" }");
            }
            rendered
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
///
/// Bulk advice is always computed and delivered to `diagnostics` once on success,
/// before source is returned. Host tools must display it (for example,
/// `|report| eprint!("{report}")`); build.rs can emit Cargo warning directives.
/// Exact-path acknowledgements with nonempty reasons silence intentional lists.
/// Paths start with /methods/NAME/request or /methods/NAME/response; without
/// methods, all exported types are reviewed under /types/NAME. With methods,
/// only their reachable shapes are advised. Stale acknowledgements are warnings.
/// Shape validation/rendering share 65,536 nodes; rooted advice separately allows
/// 65,536 expanded visits per root and 1,000,000 per report. Exhaustion returns
/// Complexity without invoking diagnostics. See [`super::lint_bulk_roots`].
/// This host API replaces the former three-argument, silent generator; generated
/// TypeScript, wire bytes, identities and runtime admission are unchanged.
pub fn generate_registry(
    types: &BTreeMap<String, Value>,
    methods: &[Value],
    runtime_module: &str,
    acknowledgements: &BTreeMap<String, String>,
    diagnostics: impl FnOnce(super::BulkLintReport),
) -> Result<String, GenerationError> {
    let (mut source, mut remaining) = registry_source(types, runtime_module, "createCodec")?;
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for method in methods {
        let name = text(method, "name")?;
        identifier(name)?;
        let id = text(method, "id")?
            .parse::<u64>()
            .map_err(|_| GenerationError::Schema)?;
        // Previously distinct names could reuse an identity (including "7"/"07"),
        // letting the same frame acquire incompatible payload meanings. Reject
        // numeric collisions within this registry, across query/update modes.
        // Valid bytes are unchanged; consumers must assign distinct IDs and
        // coordinate affected clients. This is not cross-registry schema binding.
        if types.contains_key(name)
            || !names.insert(name)
            || !matches!(text(method, "mode")?, "query" | "update")
            || !ids.insert(id)
        {
            return Err(GenerationError::Schema);
        }
        super::limits::validate(method)?;
        let request = ty(&method["request"], types, 0, &mut remaining, true)?;
        let response = ty(&method["response"], types, 0, &mut remaining, true)?;
        writeln!(source,
            "export const {name} = (() => {{ const codec = $createCodec({method}, $types); return {{ methodName: {name_json}, mode: {mode} as const, encodeRequest: (value: {request}): globalThis.Uint8Array => codec.encodeRequest(value), decodeRequest: (bytes: globalThis.Uint8Array): {request} => codec.decodeRequest(bytes) as {request}, encodeResponse: (value: {response}): globalThis.Uint8Array => codec.encodeResponse(value), decodeResponse: (bytes: globalThis.Uint8Array): {response} => codec.decodeResponse(bytes) as {response} }}; }})();",
            name_json = json!(name), mode = method["mode"],
        ).expect("writing to String cannot fail");
    }
    let report = super::bulk_lint::lint_generated(types, methods, acknowledgements)?;
    diagnostics(report);
    Ok(source)
}

/// Generate explicit payload codecs using the same schemas and runtime as frames.
///
/// Each payload has `name`, `schema`, `bytes`, `nodes` and `depth`. Names must be
/// unique and distinct from exported type names. Byte limits exclude envelopes
/// and may be zero; node/depth limits are positive. Use [`super::payload_limits`]
/// to project native policy. Schema/reference validation and complexity bounds
/// match [`generate_registry`]. Scalar checks belong in the runtime adapter.
/// Bulk advice is mandatory and uses `/payloads/NAME/...` acknowledgement paths.
/// Consumers explicitly choose payload APIs; framed bytes and identities are
/// unchanged. For example u64 7 encodes as `07`, with no synthetic method ID.
#[expect(
    clippy::missing_panics_doc,
    reason = "Payload names are validated before the immutable lint walk; writing to String is infallible."
)]
pub fn generate_payload_registry(
    types: &BTreeMap<String, Value>,
    payloads: &[Value],
    runtime_module: &str,
    acknowledgements: &BTreeMap<String, String>,
    diagnostics: impl FnOnce(super::BulkLintReport),
) -> Result<String, GenerationError> {
    let (mut source, mut remaining) = registry_source(types, runtime_module, "createPayloadCodec")?;
    let mut names = BTreeSet::new();
    for payload in payloads {
        let name = text(payload, "name")?;
        identifier(name)?;
        if types.contains_key(name) || !names.insert(name) {
            return Err(GenerationError::Schema);
        }
        super::limits::validate_payload(payload)?;
        let rendered = ty(&payload["schema"], types, 0, &mut remaining, true)?;
        writeln!(source,
            "export const {name} = (() => {{ const codec = $createCodec({payload}, $types); return {{ encode: (value: {rendered}): globalThis.Uint8Array => codec.encode(value), decode: (bytes: globalThis.Uint8Array): {rendered} => codec.decode(bytes) as {rendered} }}; }})();"
        ).expect("writing to String cannot fail");
    }
    let report = if payloads.is_empty() {
        super::bulk_lint::lint_generated(types, &[], acknowledgements)?
    } else {
        super::bulk_lint::lint_validated(
            payloads.iter().map(|payload| {
                (
                    format!("/payloads/{}", payload["name"].as_str().unwrap()),
                    &payload["schema"],
                )
            }),
            types,
            acknowledgements,
        )?
    };
    diagnostics(report);
    Ok(source)
}

/// Validate a standalone payload schema and its complete named registry.
///
/// Uses the generator's shape, identifier, reference, alias-cycle and complexity
/// checks without emitting a module. Recursive records/collections are allowed;
/// alias-only cycles and invalid unused definitions are rejected. Registry and
/// root share the generator's 65,536-node allowance and inline depth limit of 64.
///
/// This checks trusted metadata, not application values, resource policies or
/// consumer scalar callbacks. It does not produce bulk advice; use
/// [`super::lint_bulk_roots`] for that. Runtime acceptance and protocol bytes
/// are unchanged; existing consumers need no migration.
///
/// ```
/// use ic_wire::typescript::validate_payload_schema;
/// let schema = serde_json::json!({"k":"tuple","items":[{"k":"u","bits":64},{"k":"text"}]});
/// validate_payload_schema(&Default::default(), &schema)?;
/// # Ok::<(), ic_wire::typescript::GenerationError>(())
/// ```
pub fn validate_payload_schema(
    types: &BTreeMap<String, Value>,
    schema: &Value,
) -> Result<(), GenerationError> {
    let mut remaining = SCHEMA_WORK;
    render_types(types, &mut remaining, false, |_, _| {})?;
    ty(schema, types, 0, &mut remaining, false).map(|_| ())
}

// Validation and generation consume definitions in the same order, with one
// shared work allowance and alias check. Validation suppresses type rendering;
// generation appends them without building another intermediate registry.
fn render_types(
    types: &BTreeMap<String, Value>,
    remaining: &mut usize,
    render: bool,
    mut emit: impl FnMut(&str, String),
) -> Result<(), GenerationError> {
    for (name, node) in types {
        identifier(name)?;
        emit(name, ty(node, types, 0, remaining, render)?);
    }
    validate_aliases(types)
}

fn registry_source(
    types: &BTreeMap<String, Value>,
    runtime_module: &str,
    factory: &str,
) -> Result<(String, usize), GenerationError> {
    let mut remaining = SCHEMA_WORK;
    let mut source = format!(
        "// Generated from IC Wire metadata; edit the source contracts.\nimport {{ {factory} as $createCodec }} from {};\n",
        json!(runtime_module)
    );
    render_types(types, &mut remaining, true, |name, rendered| {
        writeln!(source, "export type {name} = {rendered};")
            .expect("writing to String cannot fail");
    })?;
    // `$` is outside consumer identifiers, so helpers cannot capture exports.
    // Computed __proto__ keys preserve the own properties JSON.parse previously
    // produced. JSON escapes quotes inside strings, so this matches keys only.
    // Registry meaning and wire schemas stay unchanged; no consumer migration.
    // Serialize the borrowed registry directly instead of constructing a second
    // Value tree. JSON bytes, validation and consumer contracts are unchanged.
    let registry = serde_json::to_string(types).map_err(|_| GenerationError::Schema)?;
    // Reuse the serialized buffer when no computed prototype key is needed.
    let registry = if registry.contains("\"__proto__\":") {
        registry.replace("\"__proto__\":", "[\"__proto__\"]:")
    } else {
        registry
    };
    source.push_str("const $types = ");
    source.push_str(&registry);
    source.push_str(";\n");
    Ok((source, remaining))
}
