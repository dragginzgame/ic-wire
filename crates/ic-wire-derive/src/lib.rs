//! Strict metadata derivation; Serde alone owns payload serialization.
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Describe ordinary structs and externally tagged enums, including generics.
///
/// Type parameters require Wire; existing lifetimes, const parameters and where
/// clauses are preserved. Field `serde(borrow)` and symmetric field/variant
/// `serde(rename = "name")`, standard byte and qualified packed field adapters
/// are supported; other attributes/discriminants reject.
/// Use `#[wire(crate = "renamed_dependency")]` when ic-wire is renamed.
#[proc_macro_derive(Wire, attributes(serde, wire))]
pub fn derive_wire(input: TokenStream) -> TokenStream {
    expand(&parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn reject_attributes(attrs: &[syn::Attribute]) -> syn::Result<()> {
    for attr in attrs {
        if attr.path().is_ident("serde") || attr.path().is_ident("wire") {
            return Err(syn::Error::new_spanned(
                attr,
                "unsupported metadata attribute; supply an explicitly qualified manual projection",
            ));
        }
    }
    Ok(())
}

fn fields(
    fields: &Fields,
    owner: &syn::Path,
    context: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    Ok(match fields {
        Fields::Unit => quote!(#owner::typescript::__private::json!({"k":"unit"})),
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let bytes = unnamed_attributes(&fields.unnamed[0].attrs, owner)?;
            let value = projection(&fields.unnamed[0].ty, bytes, context);
            quote!(#owner::typescript::__private::json!({"k":"newtype","item":#value}))
        }
        Fields::Named(fields) => {
            let entries = fields
                .named
                .iter()
                .map(|field| -> syn::Result<_> {
                    let name = field.ident.as_ref().unwrap().to_string();
                    let (rename, bytes) = member_attributes(&field.attrs, true, owner)?;
                    let name = rename
                        .unwrap_or_else(|| name.strip_prefix("r#").unwrap_or(&name).to_owned());
                    let value = projection(&field.ty, bytes, context);
                    Ok(quote!({"name":#name,"value":#value}))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            quote!(#owner::typescript::__private::json!({"k":"record","fields":[#(#entries),*]}))
        }
        Fields::Unnamed(fields) => {
            let entries = fields
                .unnamed
                .iter()
                .map(|field| -> syn::Result<_> {
                    let bytes = unnamed_attributes(&field.attrs, owner)?;
                    Ok(projection(&field.ty, bytes, context))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            quote!(#owner::typescript::__private::json!({"k":"tuple","items":[#(#entries),*]}))
        }
    })
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let source = input.to_token_stream().to_string();
    let mut local_name = String::from("__wire_context");
    while source.contains(&local_name) {
        local_name.push('_');
    }
    let context = format_ident!("{local_name}");
    let mut owner: syn::Path = syn::parse_quote!(::ic_wire);
    let mut renamed = false;
    for attr in &input.attrs {
        if attr.path().is_ident("wire") {
            attr.parse_nested_meta(|meta| {
                if !meta.path.is_ident("crate") || renamed {
                    return Err(meta.error("expected one wire(crate = \"dependency\")"));
                }
                let value: syn::LitStr = meta.value()?.parse()?;
                owner = value.parse()?;
                renamed = true;
                Ok(())
            })?;
        } else {
            reject_attributes(core::slice::from_ref(attr))?;
        }
    }
    let body = match &input.data {
        Data::Struct(data) => fields(&data.fields, &owner, &context)?,
        Data::Enum(data) => {
            let mut variants = Vec::new();
            for (id, variant) in data.variants.iter().enumerate() {
                let (renamed_variant, _) = member_attributes(&variant.attrs, false, &owner)?;
                if variant.discriminant.is_some() {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "explicit discriminants are not qualified",
                    ));
                }
                let name = variant.ident.to_string();
                let name = renamed_variant
                    .unwrap_or_else(|| name.strip_prefix("r#").unwrap_or(&name).to_owned());
                let value = if let Fields::Unnamed(unnamed) = &variant.fields {
                    if unnamed.unnamed.len() == 1 {
                        let bytes = unnamed_attributes(&unnamed.unnamed[0].attrs, &owner)?;
                        projection(&unnamed.unnamed[0].ty, bytes, &context)
                    } else {
                        fields(&variant.fields, &owner, &context)?
                    }
                } else {
                    fields(&variant.fields, &owner, &context)?
                };
                variants.push(quote!({"name":#name,"id":#id,"value":#value}));
            }
            quote!(#owner::typescript::__private::json!({"k":"enum","variants":[#(#variants),*]}))
        }
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "unions cannot be projected",
            ));
        }
    };
    let name = &input.ident;
    let mut generics = input.generics.clone();
    for parameter in generics.type_params_mut() {
        parameter
            .bounds
            .push(syn::parse_quote!(#owner::typescript::Wire));
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics #owner::typescript::Wire for #name #type_generics #where_clause {
            fn schema(#context: &mut #owner::typescript::SchemaContext) -> ::core::result::Result<#owner::typescript::__private::Value, #owner::typescript::GenerationError> {
                ::core::result::Result::Ok(#body)
            }
        }
    })
}

#[derive(Clone, Copy)]
enum Adapter {
    Bytes,
    Packed(u8),
}

fn projection(
    ty: &syn::Type,
    bytes: Option<Adapter>,
    context: &syn::Ident,
) -> proc_macro2::TokenStream {
    match bytes {
        Some(Adapter::Bytes) => quote!(#context.describe_bytes::<#ty>()?),
        Some(Adapter::Packed(kind)) => quote!(#context.describe_packed::<#ty,#kind>()?),
        None => quote!(#context.describe::<#ty>()?),
    }
}

fn member_attributes(
    attrs: &[syn::Attribute],
    allow_field: bool,
    owner: &syn::Path,
) -> syn::Result<(Option<String>, Option<Adapter>)> {
    let mut borrowed = false;
    let mut renamed = None;
    let mut bytes = None;
    for attr in attrs {
        if attr.path().is_ident("serde") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") && renamed.is_none() {
                    let name: syn::LitStr = meta.value()?.parse()?;
                    renamed = Some(name.value());
                } else if meta.path.is_ident("with") && allow_field && bytes.is_none() {
                    let path: syn::LitStr = meta.value()?.parse()?;
                    let path = path.value();
                    let path = path.trim_start_matches("::");
                    if path == "serde_bytes" {
                        bytes = Some(Adapter::Bytes);
                    } else {
                        let owner = owner.to_token_stream().to_string().replace(' ', "");
                        let prefix = format!("{}::serde_packed::", owner.trim_start_matches("::"));
                        let kind =
                            match path.strip_prefix(&prefix) {
                                Some("bools") => 0,
                                Some("finite_f32") => 1,
                                Some("finite_f64") => 2,
                                Some("nonzero_i64") => 3,
                                Some("heap_u64") => 4,
                                _ => return Err(meta.error(
                                    "only qualified standard byte/packed adapters are supported",
                                )),
                            };
                        bytes = Some(Adapter::Packed(kind));
                    }
                } else if meta.path.is_ident("borrow") && allow_field && !borrowed {
                    borrowed = true;
                    if meta.input.peek(syn::Token![=]) {
                        let _: syn::LitStr = meta.value()?.parse()?;
                    }
                } else {
                    return Err(meta.error("unsupported or duplicate Serde metadata attribute"));
                }
                Ok(())
            })?;
        } else {
            reject_attributes(core::slice::from_ref(attr))?;
        }
    }
    Ok((renamed, bytes))
}

fn unnamed_attributes(attrs: &[syn::Attribute], owner: &syn::Path) -> syn::Result<Option<Adapter>> {
    let (rename, bytes) = member_attributes(attrs, true, owner)?;
    if rename.is_some() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "renaming an unnamed field is not supported",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests;
