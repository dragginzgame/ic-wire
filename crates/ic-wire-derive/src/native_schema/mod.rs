//! Metadata only: Serde retains complete ownership of serialization.
use proc_macro2::TokenStream;
use quote::quote;
use syn::visit::Visit;
use syn::{Data, DeriveInput, Fields, parse_quote};

// Recursive field bounds would make trait selection recurse before the metadata
// traversal can return Complexity. Keep the existing parameter bounds for these
// fields; ordinary fields can instead constrain their actual serialized type.
struct RecursiveType<'a> {
    name: &'a syn::Ident,
    found: bool,
}
impl<'ast> Visit<'ast> for RecursiveType<'_> {
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        self.found |= path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "Self" || segment.ident == *self.name);
        syn::visit::visit_type_path(self, path);
    }
}

struct Projection {
    transparent: bool,
    bytes: bool,
}

fn attributes(
    attrs: &[syn::Attribute],
    allow_transparent: bool,
    allow_borrow: bool,
    allow_bytes: bool,
) -> syn::Result<Projection> {
    let mut transparent = false;
    let mut borrow = false;
    let mut bytes = false;
    let mut deny_unknown = false;
    for attr in attrs {
        if attr.path().is_ident("wire") {
            return Err(syn::Error::new_spanned(
                attr,
                "Wire follows Serde order; wire IDs are not supported",
            ));
        }
        if attr.path().is_ident("serde") {
            attr.parse_nested_meta(|meta| {
                if allow_transparent && meta.path.is_ident("transparent") && !transparent {
                    transparent = true;
                    Ok(())
                } else if allow_transparent && meta.path.is_ident("deny_unknown_fields") && !deny_unknown {
                    // This changes map-format input handling, not Postcard's
                    // positional layout. TS already rejects extra record keys.
                    deny_unknown = true;
                    Ok(())
                } else if allow_bytes && meta.path.is_ident("with") && !bytes {
                    let adapter = meta.value()?.parse::<syn::LitStr>()?;
                    if !matches!(adapter.value().as_str(), "serde_bytes" | "::serde_bytes") {
                        return Err(meta.error("unsupported Serde byte adapter; custom layouts require explicit verified metadata"));
                    }
                    bytes = true;
                    Ok(())
                } else if allow_borrow && meta.path.is_ident("borrow") && !borrow {
                    // Borrowing changes Deserialize lifetimes, not Serde's layout.
                    // Previously rejected DTOs can now derive the same metadata;
                    // frame bytes, admission and client representations are unchanged.
                    // Serde remains responsible for validating the named lifetimes.
                    if meta.input.peek(syn::Token![=]) {
                        meta.value()?.parse::<syn::LitStr>()?;
                    }
                    borrow = true;
                    Ok(())
                } else {
                    Err(meta.error("unsupported Serde attribute for Wire; use an explicit verified Wire implementation for custom layouts"))
                }
            })?;
        }
    }
    Ok(Projection { transparent, bytes })
}

fn fields(fields: &Fields, context: &syn::Ident, transparent: bool) -> syn::Result<TokenStream> {
    let values: Vec<_> = fields
        .iter()
        .map(|f| {
            let ty = &f.ty;
            let projection = attributes(&f.attrs, false, true, true)?;
            Ok(if projection.bytes {
                // serde_bytes encodes a byte buffer rather than visiting a list.
                // Existing annotated fields keep their bytes and budgets; only
                // previously unavailable metadata becomes Uint8Array. Regenerate
                // bindings. The sealed bound rejects unqualified adapter inputs.
                quote!(#context.describe_bytes::<#ty>()?)
            } else {
                quote!(#context.describe::<#ty>()?)
            })
        })
        .collect::<syn::Result<_>>()?;
    let json = quote!(::ic_wire::__private::serde_json::json);
    if transparent {
        if values.len() != 1 {
            return Err(syn::Error::new_spanned(
                fields,
                "Wire transparent structs require exactly one field",
            ));
        }
        return Ok(values[0].clone());
    }
    Ok(match fields {
        Fields::Unit => quote!(#json!({"k":"unit"})),
        Fields::Named(fields) => {
            let names = fields.named.iter().map(|f| {
                f.ident
                    .as_ref()
                    .unwrap()
                    .to_string()
                    .trim_start_matches("r#")
                    .to_owned()
            });
            quote!(#json!({"k":"record","fields":[#({"name":#names,"value":#values}),*]}))
        }
        Fields::Unnamed(_) if values.len() == 1 => {
            let value = &values[0];
            quote!(#json!({"k":"newtype","item":#value}))
        }
        Fields::Unnamed(_) => quote!(#json!({"k":"tuple","items":[#(#values),*]})),
    })
}

pub(super) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let transparent = attributes(&input.attrs, true, false, false)?.transparent;
    let context = super::fresh_ident(&quote!(#input).to_string(), "__ic_wire_schema");
    // The retired finite Wire IDs sorted fields and determined enum tags. This
    // projection instead describes unchanged Serde bytes: declaration-order fields
    // and zero-based variant indices. Reordering is a protocol change requiring
    // coordinated consumers; no tags, compatibility reader or serializer is added.
    let body = match &input.data {
        Data::Struct(data) => fields(&data.fields, &context, transparent)?,
        Data::Enum(data) if !transparent && !data.variants.is_empty() => {
            let mut variants = Vec::new();
            for (id, variant) in data.variants.iter().enumerate() {
                let newtype = matches!(&variant.fields, Fields::Unnamed(f) if f.unnamed.len() == 1);
                // Serde also permits borrow on a newtype variant: it applies to
                // the sole field's lifetimes and leaves the tag/payload unchanged.
                attributes(&variant.attrs, false, newtype, false)?;
                let name = variant
                    .ident
                    .to_string()
                    .trim_start_matches("r#")
                    .to_owned();
                let value = fields(&variant.fields, &context, newtype)?;
                variants.push(quote!({"name":#name,"id":#id,"value":#value}));
            }
            quote!(
                ::ic_wire::__private::serde_json::json!({"k":"enum","variants":[#(#variants),*]})
            )
        }
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Wire supports structs and nonempty externally tagged enums",
            ));
        }
    };
    let name = &input.ident;
    let mut generics = input.generics.clone();
    let fields: Vec<_> = match &input.data {
        Data::Struct(data) => data.fields.iter().collect(),
        Data::Enum(data) => data.variants.iter().flat_map(|v| &v.fields).collect(),
        Data::Union(_) => unreachable!(),
    };
    let mut recursive = false;
    for field in fields {
        let mut visitor = RecursiveType { name, found: false };
        visitor.visit_type(&field.ty);
        recursive |= visitor.found;
        // Concrete fields need no where-clause: their calls are checked in the
        // body. Redundant A: Wire / B: Wire clauses on mutually recursive types
        // would instead make trait selection cycle before projection can start.
        if !visitor.found && input.generics.type_params().next().is_some() {
            let ty = &field.ty;
            let bound = if attributes(&field.attrs, false, true, true)?.bytes {
                quote!(::ic_wire::typescript::SerdeBytesField)
            } else {
                quote!(::ic_wire::typescript::Wire)
            };
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: #bound));
        }
    }
    if recursive {
        for parameter in generics.type_params_mut() {
            parameter
                .bounds
                .push(parse_quote!(::ic_wire::typescript::Wire));
        }
    }
    // Serialize is Wire's supertrait. Let Serde's own bounds establish this
    // instead of assuming that every generic parameter is itself serialized.
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: ::ic_wire::__private::serde::Serialize));
    let (implementation, ty, constraints) = generics.split_for_impl();
    Ok(quote! {
        // Metadata is a native host capability even when a consumer forwards its
        // schema feature to a Wasm runtime build. Serde impls remain consumer-owned.
        #[cfg(not(target_family = "wasm"))]
        impl #implementation ::ic_wire::typescript::Wire for #name #ty #constraints {
            fn native_schema(#context: &mut ::ic_wire::typescript::SchemaContext)
                -> ::core::result::Result<::ic_wire::__private::serde_json::Value, ::ic_wire::typescript::GenerationError>
            {
                let schema = #body;
                ::core::result::Result::Ok(schema)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_unsupported_attributes_and_shapes() {
        for source in [
            "#[serde(untagged)] enum E { A(u8) }",
            "#[serde(tag = \"kind\")] enum E { A }",
            "#[serde(rename_all = \"snake_case\")] struct A { a: u8 }",
            "struct A { #[serde(skip)] a: u8 }",
            "struct A { #[serde(flatten)] a: u8 }",
            "struct A { #[serde(with = \"custom\")] a: u8 }",
            "struct A { #[serde(serialize_with = \"custom\")] a: u8 }",
            "struct A { #[serde(deny_unknown_fields)] a: u8 }",
            "#[serde(deny_unknown_fields, deny_unknown_fields)] struct A { a: u8 }",
            "struct A { #[serde(with = \"serde_bytes\", with = \"serde_bytes\")] a: Vec<u8> }",
            "struct A { #[serde(with = \"serde_bytes\", deserialize_with = \"custom\")] a: Vec<u8> }",
            "#[serde(borrow)] struct A { a: u8 }",
            "enum A { #[serde(borrow)] B }",
            "enum A { #[serde(borrow)] B(u8, u8) }",
            "enum A { #[serde(borrow)] B { a: u8 } }",
            "struct A { #[serde(borrow, borrow)] a: u8 }",
            "struct A { #[serde(borrow = 1)] a: u8 }",
            "struct A { #[serde(borrow, with = \"custom\")] a: u8 }",
            "struct A { #[wire(id = 0)] a: u8 }",
            "#[serde(transparent)] struct A(u8, u8);",
            "enum E {}",
        ] {
            assert!(
                super::expand(syn::parse_str(source).unwrap()).is_err(),
                "{source}"
            );
        }
    }
}
