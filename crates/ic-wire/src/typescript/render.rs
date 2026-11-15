use super::{
    GenerationError, Method, Node, PayloadLimits, RUNTIME_SCHEMA_FORMAT, ValidatedRegistry,
    identifier,
};
use alloc::{collections::BTreeSet, string::String};
use core::fmt::{self, Write};

// Check before extending the output, including escaped field names. Syntax
// recursion is bounded at import; named references never expand during rendering.
struct Output {
    text: String,
    remaining: usize,
}

impl Write for Output {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.remaining = self.remaining.checked_sub(value.len()).ok_or(fmt::Error)?;
        self.text.push_str(value);
        Ok(())
    }
}

impl Output {
    fn quoted(&mut self, value: &str) -> fmt::Result {
        self.write_char('"')?;
        for character in value.chars() {
            match character {
                '"' => self.write_str("\\\"")?,
                '\\' => self.write_str("\\\\")?,
                '\u{0000}'..='\u{001f}' | '\u{2028}' | '\u{2029}' => {
                    write!(self, "\\u{:04x}", u32::from(character))?;
                }
                _ => self.write_char(character)?,
            }
        }
        self.write_char('"')
    }

    fn bounds(&mut self, min: Option<u64>, max: Option<u64>) -> fmt::Result {
        if let Some(min) = min {
            write!(self, "/* min {min} */ ")?;
        }
        if let Some(max) = max {
            write!(self, "/* max {max} */ ")?;
        }
        Ok(())
    }
}

impl ValidatedRegistry {
    /// Generate a typed payload client with privately owned metadata and policy.
    ///
    /// `root` names an admitted type. `export` names the exported codec object.
    /// The trusted runtime module must export `schemaFormat` matching
    /// [`RUNTIME_SCHEMA_FORMAT`] and `createPayloadCodec(metadata, registry)`.
    /// The factory returns `encode(unknown): Uint8Array` and `decode(Uint8Array):
    /// unknown`, using the reference JSON grammar. Qualification uses both the
    /// candidate runtime and published Core 0.22.4's interpreter.
    /// Runtime code is not bundled, downloaded or selected by this function.
    ///
    /// # Errors
    /// Returns Schema for invalid names, missing roots, empty module specifiers
    /// or zero node/depth allowances; Complexity for unsafe JS integers or an
    /// artifact exceeding the registry's output allowance.
    pub fn payload_module(
        &self,
        export: &str,
        root: &str,
        runtime_module: &str,
        limits: PayloadLimits,
    ) -> Result<String, GenerationError> {
        self.admit_client(export, root, runtime_module, limits)?;
        let mut output = Output {
            text: String::new(),
            remaining: self.output_bytes,
        };
        self.render_payload_module(&mut output, export, root, runtime_module, limits)
            .map_err(|_| GenerationError::Complexity)?;
        Ok(output.text)
    }

    fn admit_client(
        &self,
        export: &str,
        root: &str,
        runtime_module: &str,
        limits: PayloadLimits,
    ) -> Result<(), GenerationError> {
        identifier(export)?;
        if export == "globalThis"
            || self
                .names
                .binary_search_by(|name| name.as_str().cmp(root))
                .is_err()
            || self
                .names
                .binary_search_by(|name| name.as_str().cmp(export))
                .is_ok()
            || runtime_module.is_empty()
            || runtime_module.contains('\0')
            || limits.nodes == 0
            || limits.depth == 0
        {
            return Err(GenerationError::Schema);
        }
        if [limits.bytes, limits.nodes, limits.depth]
            .into_iter()
            .any(|n| n > (1_u64 << 53) - 1)
        {
            return Err(GenerationError::Complexity);
        }
        Ok(())
    }

    /// Generate typed framed codecs with independent request/response policy.
    ///
    /// The runtime must export `schemaFormat` and `createCodec` with the
    /// reference framed-codec interface. IDs are emitted as decimal strings,
    /// preserving all u64 values without JavaScript Number rounding. Duplicate
    /// IDs or exports reject before rendering. This emits no routing/execution.
    ///
    /// # Errors
    /// Returns Schema for invalid names, roots, policies or duplicate IDs;
    /// Complexity for unsafe header-inclusive byte allowances or excess output.
    pub fn method_module(
        &self,
        methods: &[Method<'_>],
        runtime_module: &str,
    ) -> Result<String, GenerationError> {
        if methods.is_empty() {
            return Err(GenerationError::Schema);
        }
        // Each method necessarily emits at least one byte; bound admission work
        // before allocating duplicate-detection sets for caller-owned input.
        if methods.len() > self.output_bytes {
            return Err(GenerationError::Complexity);
        }
        let mut names = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for method in methods {
            for (root, limits) in [
                (method.request, method.request_limits),
                (method.response, method.response_limits),
            ] {
                self.admit_client(method.export, root, runtime_module, limits)?;
                if limits.bytes > (1_u64 << 53) - 12 {
                    return Err(GenerationError::Complexity);
                }
            }
            if !names.insert(method.export) || !ids.insert(method.id) {
                return Err(GenerationError::Schema);
            }
        }
        let mut output = Output {
            text: String::new(),
            remaining: self.output_bytes,
        };
        self.render_methods(&mut output, methods, runtime_module)
            .map_err(|_| GenerationError::Complexity)?;
        Ok(output.text)
    }

    fn render_methods(
        &self,
        output: &mut Output,
        methods: &[Method<'_>],
        runtime_module: &str,
    ) -> fmt::Result {
        self.module_prelude(output, runtime_module, "createCodec")?;
        for method in methods {
            writeln!(output, "export const {} = (() => {{", method.export)?;
            write!(
                output,
                "const $codec = $create({{id:'{}',request:{{k:'ref',name:",
                method.id
            )?;
            output.quoted(method.request)?;
            output.write_str("},response:{k:'ref',name:")?;
            output.quoted(method.response)?;
            for (prefix, limits) in [
                ("request", method.request_limits),
                ("response", method.response_limits),
            ] {
                write!(
                    output,
                    "{separator}{prefix}Bytes:{}, {prefix}Nodes:{}, {prefix}Depth:{}",
                    limits.bytes + 11,
                    limits.nodes,
                    limits.depth,
                    separator = if prefix == "request" { "}," } else { "," }
                )?;
            }
            output.write_str("}, $types);\nreturn globalThis.Object.freeze({\n")?;
            for (direction, root) in [("Request", method.request), ("Response", method.response)] {
                writeln!(
                    output,
                    "encode{direction}:(value: {root}): globalThis.Uint8Array => $codec.encode{direction}(value), decode{direction}:(bytes: globalThis.Uint8Array): {root} => $codec.decode{direction}(bytes) as {root},"
                )?;
            }
            output.write_str("});\n})();\n")?;
        }
        Ok(())
    }

    fn render_payload_module(
        &self,
        output: &mut Output,
        export: &str,
        root: &str,
        runtime_module: &str,
        limits: PayloadLimits,
    ) -> fmt::Result {
        self.module_prelude(output, runtime_module, "createPayloadCodec")?;
        writeln!(output, "export const {export} = (() => {{")?;
        output.write_str("const $codec = $create({schema:{k:'ref',name:")?;
        output.quoted(root)?;
        writeln!(
            output,
            "}},bytes:{},nodes:{},depth:{}}}, $types);",
            limits.bytes, limits.nodes, limits.depth
        )?;
        writeln!(
            output,
            "return globalThis.Object.freeze({{encode:(value: {root}): globalThis.Uint8Array => $codec.encode(value), decode:(bytes: globalThis.Uint8Array): {root} => $codec.decode(bytes) as {root}}});\n}})();"
        )
    }

    fn module_prelude(
        &self,
        output: &mut Output,
        runtime_module: &str,
        factory: &str,
    ) -> fmt::Result {
        write!(
            output,
            "// Generated IC Wire client; no transport or execution.\nimport {{ schemaFormat as $format, {factory} as $create }} from "
        )?;
        output.quoted(runtime_module)?;
        output.write_str(";\nif ($format !== ")?;
        output.quoted(RUNTIME_SCHEMA_FORMAT)?;
        output.write_str(") throw new globalThis.Error('IC Wire runtime format mismatch');\n")?;
        self.render(output)?;
        output.write_str("const $types = globalThis.JSON.parse(")?;
        // A bounded temporary JSON string permits correct JS string escaping,
        // including __proto__ keys, without another schema interpretation path.
        let metadata = self.metadata_json().map_err(|_| fmt::Error)?;
        output.quoted(&metadata)?;
        output.write_str(");\n")
    }

    /// Export the admitted subset in the reference JSON registry grammar.
    ///
    /// This is data for a trusted metadata consumer, not executable JavaScript.
    /// Parse it as JSON: embedding it as an object literal changes the meaning
    /// of `__proto__` keys. Supplied names and use-site constraints are preserved.
    /// No runtime version or payload policy is implied by this registry alone.
    ///
    /// # Errors
    /// Returns Complexity before extending output beyond its byte allowance.
    pub fn metadata_json(&self) -> Result<String, GenerationError> {
        let mut output = Output {
            text: String::new(),
            remaining: self.output_bytes,
        };
        self.render_metadata(&mut output)
            .map_err(|_| GenerationError::Complexity)?;
        Ok(output.text)
    }

    fn render_metadata(&self, output: &mut Output) -> fmt::Result {
        output.write_char('{')?;
        for (index, (name, root)) in self.names.iter().zip(&self.roots).enumerate() {
            if index != 0 {
                output.write_char(',')?;
            }
            output.quoted(name)?;
            output.write_char(':')?;
            self.metadata_node(*root, output)?;
        }
        output.write_char('}')
    }

    fn metadata_node(&self, id: usize, output: &mut Output) -> fmt::Result {
        output.write_str("{\"k\":")?;
        match &self.nodes[id] {
            Node::Unit => output.quoted("unit")?,
            Node::Bool => output.quoted("bool")?,
            Node::Integer { bits, signed } => {
                output.quoted(if *signed { "i" } else { "u" })?;
                write!(output, ",\"bits\":{bits}")?;
            }
            Node::Text { max } => {
                output.quoted("text")?;
                output.json_bound("max", *max)?;
            }
            Node::Bytes { min, max } => {
                output.quoted("bytes")?;
                output.json_bound("min", *min)?;
                output.json_bound("max", *max)?;
            }
            Node::PackedI16 { max } => {
                output.quoted("packed-i16")?;
                output.json_bound("max", Some(*max))?;
            }
            Node::List { item, max } => {
                output.quoted("list")?;
                output.json_bound("max", *max)?;
                output.write_str(",\"item\":")?;
                self.metadata_node(*item, output)?;
            }
            Node::Option(item) | Node::Alias(item) => {
                output.quoted(if matches!(self.nodes[id], Node::Option(_)) {
                    "option"
                } else {
                    "newtype"
                })?;
                output.write_str(",\"item\":")?;
                self.metadata_node(*item, output)?;
            }
            Node::Ref(root) => {
                output.write_str("\"ref\",\"name\":")?;
                output.quoted(&self.names[*root])?;
            }
            Node::Record(fields) => {
                output.write_str("\"record\",\"fields\":[")?;
                for (index, (name, child)) in fields.iter().enumerate() {
                    if index != 0 {
                        output.write_char(',')?;
                    }
                    output.write_str("{\"name\":")?;
                    output.quoted(name)?;
                    output.write_str(",\"value\":")?;
                    self.metadata_node(*child, output)?;
                    output.write_char('}')?;
                }
                output.write_char(']')?;
            }
            Node::Enum(variants) => {
                output.write_str("\"enum\",\"variants\":[")?;
                for (index, (name, tag, child)) in variants.iter().enumerate() {
                    if index != 0 {
                        output.write_char(',')?;
                    }
                    output.write_str("{\"name\":")?;
                    output.quoted(name)?;
                    write!(output, ",\"id\":{tag},\"value\":")?;
                    self.metadata_node(*child, output)?;
                    output.write_char('}')?;
                }
                output.write_char(']')?;
            }
        }
        output.write_char('}')
    }

    /// Render deterministic TypeScript type aliases from admitted nodes.
    ///
    /// Numeric widths, enum IDs and field bounds are informational comments;
    /// TypeScript types cannot enforce these runtime constraints. This produces
    /// declarations only, with no browser codec or executable metadata.
    ///
    /// # Errors
    /// Returns Complexity before extending output beyond its byte allowance.
    pub fn declarations(&self) -> Result<String, GenerationError> {
        let mut output = Output {
            text: String::new(),
            remaining: self.output_bytes,
        };
        self.render(&mut output)
            .map_err(|_| GenerationError::Complexity)?;
        Ok(output.text)
    }

    fn render(&self, output: &mut Output) -> fmt::Result {
        for (name, root) in self.names.iter().zip(&self.roots) {
            write!(output, "export type {name} = ")?;
            self.render_node(*root, output)?;
            output.write_str(";\n")?;
        }
        Ok(())
    }

    fn render_node(&self, id: usize, output: &mut Output) -> fmt::Result {
        match &self.nodes[id] {
            Node::Unit => output.write_str("null"),
            Node::Bool => output.write_str("boolean"),
            Node::Integer { bits, signed } => {
                write!(
                    output,
                    "/* {}{bits} */ {}",
                    if *signed { "i" } else { "u" },
                    if *bits > 32 { "bigint" } else { "number" }
                )
            }
            Node::Text { max } => {
                output.bounds(None, *max)?;
                output.write_str("string")
            }
            Node::Bytes { min, max } => {
                output.bounds(*min, *max)?;
                output.write_str("globalThis.Uint8Array")
            }
            Node::PackedI16 { max } => {
                output.bounds(None, Some(*max))?;
                output.write_str("globalThis.Int16Array")
            }
            Node::List { item, max } => {
                output.bounds(None, *max)?;
                output.write_str("globalThis.ReadonlyArray<")?;
                self.render_node(*item, output)?;
                output.write_char('>')
            }
            Node::Option(item) => {
                output.write_str("null | { readonly some: ")?;
                self.render_node(*item, output)?;
                output.write_str(" }")
            }
            Node::Alias(item) => self.render_node(*item, output),
            Node::Ref(root) => output.write_str(&self.names[*root]),
            Node::Record(fields) => {
                output.write_char('{')?;
                for (name, child) in fields {
                    output.write_str(" readonly ")?;
                    output.quoted(name)?;
                    output.write_str(": ")?;
                    self.render_node(*child, output)?;
                    output.write_char(';')?;
                }
                output.write_str(" }")
            }
            Node::Enum(variants) => {
                for (index, (name, tag, child)) in variants.iter().enumerate() {
                    if index != 0 {
                        output.write_str(" | ")?;
                    }
                    write!(output, "/* id {tag} */ {{ readonly tag: ")?;
                    output.quoted(name)?;
                    output.write_str("; readonly value: ")?;
                    self.render_node(*child, output)?;
                    output.write_str(" }")?;
                }
                Ok(())
            }
        }
    }
}

impl Output {
    fn json_bound(&mut self, name: &str, value: Option<u64>) -> fmt::Result {
        if let Some(value) = value {
            write!(self, ",\"{name}\":{value}")?;
        }
        Ok(())
    }
}
