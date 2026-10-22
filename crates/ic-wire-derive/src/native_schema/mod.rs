//! Metadata only: Serde retains complete ownership of serialization.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_quote};

fn attributes(
    attrs: &[syn::Attribute],
    allow_transparent: bool,
    allow_borrow: bool,
) -> syn::Result<bool> {
    let mut transparent = false;
    let mut borrow = false;
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
    Ok(transparent)
}

fn fields(fields: &Fields, context: &syn::Ident, transparent: bool) -> syn::Result<TokenStream> {
    for field in fields {
        attributes(&field.attrs, false, true)?;
    }
    let values: Vec<_> = fields
        .iter()
        .map(|f| {
            let ty = &f.ty;
            quote!(#context.describe::<#ty>()?)
        })
        .collect();
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
    let transparent = attributes(&input.attrs, true, false)?;
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
                attributes(&variant.attrs, false, false)?;
                let name = variant
                    .ident
                    .to_string()
                    .trim_start_matches("r#")
                    .to_owned();
                let value = fields(
                    &variant.fields,
                    &context,
                    matches!(&variant.fields, Fields::Unnamed(f) if f.unnamed.len() == 1),
                )?;
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
    let name = input.ident;
    let mut generics = input.generics;
    for parameter in generics.type_params_mut() {
        parameter
            .bounds
            .push(parse_quote!(::ic_wire::typescript::Wire));
    }
    let (implementation, ty, constraints) = generics.split_for_impl();
    Ok(quote! {
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
            "#[serde(borrow)] struct A { a: u8 }",
            "enum A { #[serde(borrow)] B(u8) }",
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
