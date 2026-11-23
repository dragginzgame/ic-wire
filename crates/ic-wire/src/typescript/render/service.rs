use super::{BTreeSet, GenerationError, Output, String, ValidatedRegistry, Write, fmt, identifier};
use crate::typescript::ServiceMethod;

impl ValidatedRegistry {
    /// Generate a TypeScript module exporting a generic `Transport<CanisterId>` and
    /// `createService(canisterId, transport)`. Methods keep the supplied order and
    /// derive their signatures from the imported method codecs. Composite queries
    /// use the transport's query operation. Each imported codec can use its own
    /// registry. This registry supplies only the service module's output allowance.
    ///
    /// # Errors
    /// Returns Schema for invalid codec identifiers, empty or duplicate client or deployed
    /// names, empty deployed names, invalid module specifiers or unknown modes.
    /// Returns Complexity when generation exceeds the registry's output allowance.
    pub fn service_module(&self, methods: &[ServiceMethod<'_>]) -> Result<String, GenerationError> {
        if methods.len() > self.output_bytes {
            return Err(GenerationError::Complexity);
        }
        let mut bindings = BTreeSet::new();
        let mut endpoints = BTreeSet::new();
        for entry in methods {
            Self::admit_runtime_module(entry.codec_module)?;
            identifier(entry.codec_export)?;
            if entry.binding.is_empty()
                || !bindings.insert(entry.binding)
                || entry.endpoint.is_empty()
                || !endpoints.insert(entry.endpoint)
                || !matches!(entry.mode, "query" | "composite_query" | "update")
            {
                return Err(GenerationError::Schema);
            }
        }

        let mut output = Output::new(self.output_bytes);
        render_service(&mut output, methods).map_err(|_| GenerationError::Complexity)?;
        Ok(output.text)
    }
}

fn render_service(output: &mut Output, methods: &[ServiceMethod<'_>]) -> fmt::Result {
    for (index, entry) in methods.iter().enumerate() {
        write!(
            output,
            "import {{ {} as $codec{index} }} from ",
            entry.codec_export
        )?;
        output.quoted(entry.codec_module)?;
        output.write_str(";\n")?;
    }

    output.write_str("export interface Transport<CanisterId> {\n")?;
    for call in ["query", "update"] {
        writeln!(
            output,
            "  {call}(canisterId: CanisterId, method: string, bytes: Uint8Array): Promise<Uint8Array>;"
        )?;
    }
    output.write_str("}\nexport function createService<CanisterId>(canisterId: CanisterId, transport: Transport<CanisterId>) {\n  return Object.freeze({\n")?;
    for (index, entry) in methods.iter().enumerate() {
        let call = if entry.mode == "update" {
            "update"
        } else {
            "query"
        };
        output.write_str("    [")?;
        output.quoted(entry.binding)?;
        writeln!(
            output,
            "]: async (value: Parameters<typeof $codec{index}.encodeRequest>[0]) => {{"
        )?;
        write!(
            output,
            "      const reply = await transport.{call}(canisterId, "
        )?;
        output.quoted(entry.endpoint)?;
        writeln!(output, ", $codec{index}.encodeRequest(value));")?;
        writeln!(
            output,
            "      return $codec{index}.decodeResponse(reply);\n    }},"
        )?;
    }
    output.write_str("  });\n}\n")
}
