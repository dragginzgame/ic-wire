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
    adapter: Option<Adapter>,
    rename: Option<String>,
}

#[derive(Clone, Copy)]
enum Adapter {
    Bytes,
    Bools,
    FiniteF32,
    FiniteF64,
    NonzeroI64,
    HeapU64,
}

#[derive(Clone, Copy, PartialEq)]
enum AttributeSite {
    Container,
    Field,
    Variant,
    NewtypeVariant,
}

fn attributes(attrs: &[syn::Attribute], site: AttributeSite) -> syn::Result<Projection> {
    let mut transparent = false;
    let mut borrow = false;
    let mut adapter = None;
    let mut deny_unknown = false;
    let mut serde_crate = false;
    let mut rename = None;
    for attr in attrs {
        if attr.path().is_ident("wire") {
            return Err(syn::Error::new_spanned(
                attr,
                "Wire follows Serde order; wire IDs are not supported",
            ));
        }
        if attr.path().is_ident("serde") {
            attr.parse_nested_meta(|meta| {
                if site == AttributeSite::Container && meta.path.is_ident("transparent") && !transparent {
                    transparent = true;
                    Ok(())
                } else if site == AttributeSite::Container && meta.path.is_ident("deny_unknown_fields") && !deny_unknown {
                    // This changes map-format input handling, not Postcard's
                    // positional layout. TS already rejects extra record keys.
                    deny_unknown = true;
                    Ok(())
                } else if site == AttributeSite::Container && meta.path.is_ident("crate") && !serde_crate {
                    // Crate selection changes generated Rust paths, not Serde's
                    // layout. Previously rejected containers now describe their
                    // existing bytes; metadata/admission and consumers are unchanged.
                    // Serde owns validation and resolution of the supplied path.
                    meta.value()?.parse::<syn::LitStr>()?;
                    serde_crate = true;
                    Ok(())
                } else if site == AttributeSite::Field && meta.path.is_ident("with") && adapter.is_none() {
                    let path = meta.value()?.parse::<syn::LitStr>()?;
                    adapter = Some(match path.value().strip_prefix("::").unwrap_or(path.value().as_str()) {
                        "serde_bytes" => Adapter::Bytes,
                        "ic_wire::serde_packed::bools" => Adapter::Bools,
                        "ic_wire::serde_packed::finite_f32" => Adapter::FiniteF32,
                        "ic_wire::serde_packed::finite_f64" => Adapter::FiniteF64,
                        "ic_wire::serde_packed::nonzero_i64" => Adapter::NonzeroI64,
                        "ic_wire::serde_packed::heap_u64" => Adapter::HeapU64,
                        _ => return Err(meta.error("unsupported Serde byte adapter; custom layouts require explicit verified metadata")),
                    });
                    Ok(())
                } else if site != AttributeSite::Container && meta.path.is_ident("rename") && rename.is_none() {
                    rename = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                    Ok(())
                } else if matches!(site, AttributeSite::Field | AttributeSite::NewtypeVariant) && meta.path.is_ident("borrow") && !borrow {
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
    Ok(Projection {
        transparent,
        adapter,
        rename,
    })
}

fn fields<'a>(
    fields: &'a Fields,
    context: &syn::Ident,
    transparent: bool,
    collected: &mut Vec<(&'a syn::Field, Projection)>,
) -> syn::Result<TokenStream> {
    // Parse once in declaration order; metadata labels and generic bounds share
    // the same validated field projection. Emitted layouts and refusals are unchanged.
    let start = collected.len();
    for field in fields {
        collected.push((field, attributes(&field.attrs, AttributeSite::Field)?));
    }
    let projections = &collected[start..];
    let values: Vec<_> = projections
        .iter()
        .map(|(f, projection)| {
            let ty = &f.ty;
            match projection.adapter {
                Some(Adapter::Bytes) => quote!(#context.describe_bytes::<#ty>()?),
                Some(adapter @ (Adapter::FiniteF32 | Adapter::FiniteF64)) => {
                    let (primitive, word) = match adapter {
                        Adapter::FiniteF32 => (quote!(::core::primitive::f32), "finite-f32"),
                        _ => (quote!(::core::primitive::f64), "finite-f64"),
                    };
                    quote!(({
                        let _: ::core::marker::PhantomData<#ty> =
                            ::core::marker::PhantomData::<::ic_wire::__private::Vec<#primitive>>;
                        ::ic_wire::__private::serde_json::json!({"k":"packed", "word":#word})
                    }))
                }
                Some(Adapter::Bools) => quote!(({
                    let _: ::core::marker::PhantomData<#ty> =
                        ::core::marker::PhantomData::<::ic_wire::__private::Vec<::core::primitive::bool>>;
                    ::ic_wire::__private::serde_json::json!({"k":"packed", "word":"bool"})
                })),
                Some(Adapter::NonzeroI64) => quote!(({
                    // Type equality accepts aliases without trusting a syntactic
                    // type name or introducing a second public schema trait.
                    let _: ::core::marker::PhantomData<#ty> =
                        ::core::marker::PhantomData::<::ic_wire::__private::Vec<::core::num::NonZeroI64>>;
                    let mut schema = #context.describe::<::ic_wire::Packed<::core::primitive::i64>>()?;
                    schema["nonzero"] = ::ic_wire::__private::serde_json::Value::Bool(true);
                    schema
                })),
                Some(Adapter::HeapU64) => quote!(({
                    let _: ::core::marker::PhantomData<#ty> =
                        ::core::marker::PhantomData::<::ic_wire::__private::BinaryHeap<::core::primitive::u64>>;
                    #context.describe::<::ic_wire::Packed<::core::primitive::u64>>()?
                })),
                None => quote!(#context.describe::<#ty>()?),
            }
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
        Fields::Named(_) => {
            // Previously renamed fields could not derive metadata. Project their
            // browser labels without changing positional bytes or field order;
            // consumers adopting the derive must retain their existing labels.
            let mut unique = std::collections::BTreeSet::new();
            let names = projections
                .iter()
                .map(|(f, projection)| {
                    let name = projection.rename.clone().unwrap_or_else(|| {
                        f.ident
                            .as_ref()
                            .unwrap()
                            .to_string()
                            .trim_start_matches("r#")
                            .to_owned()
                    });
                    if !unique.insert(name.clone()) {
                        return Err(syn::Error::new_spanned(
                            f,
                            "duplicate Wire field name after Serde rename",
                        ));
                    }
                    Ok(name)
                })
                .collect::<syn::Result<Vec<_>>>()?;
            quote!(#json!({"k":"record","fields":[#({"name":#names,"value":#values}),*]}))
        }
        Fields::Unnamed(_) if values.len() == 1 => {
            let value = &values[0];
            quote!(#json!({"k":"newtype","item":#value}))
        }
        Fields::Unnamed(_) => quote!(#json!({"k":"tuple","items":[#(#values),*]})),
    })
}

pub(super) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let transparent = attributes(&input.attrs, AttributeSite::Container)?.transparent;
    let context = super::fresh_ident(&quote!(#input).to_string(), "__ic_wire_schema");
    // The retired finite Wire IDs sorted fields and determined enum tags. This
    // projection instead describes unchanged Serde bytes: declaration-order fields
    // and zero-based variant indices. Reordering is a protocol change requiring
    // coordinated consumers; no tags, compatibility reader or serializer is added.
    let mut collected = Vec::new();
    let body = match &input.data {
        Data::Struct(data) => fields(&data.fields, &context, transparent, &mut collected)?,
        Data::Enum(data) if !transparent && !data.variants.is_empty() => {
            let mut variants = Vec::new();
            let mut unique = std::collections::BTreeSet::new();
            for (id, variant) in data.variants.iter().enumerate() {
                let newtype = matches!(&variant.fields, Fields::Unnamed(f) if f.unnamed.len() == 1);
                // Serde also permits borrow on a newtype variant: it applies to
                // the sole field's lifetimes and leaves the tag/payload unchanged.
                // Previously symmetric renames were rejected. Only browser labels
                // change: Int64 renamed Int keeps its declaration-order tag 00.
                // No field sorting/IDs or byte changes; regenerate client labels.
                let name = attributes(
                    &variant.attrs,
                    if newtype {
                        AttributeSite::NewtypeVariant
                    } else {
                        AttributeSite::Variant
                    },
                )?
                .rename
                .unwrap_or_else(|| {
                    variant
                        .ident
                        .to_string()
                        .trim_start_matches("r#")
                        .to_owned()
                });
                if !unique.insert(name.clone()) {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "duplicate Wire variant name after Serde rename",
                    ));
                }
                let value = fields(&variant.fields, &context, newtype, &mut collected)?;
                variants.push(quote!({"name":#name,"id":#id,"value":#value}));
            }
            quote!(
                ::ic_wire::__private::serde_json::json!({"k":"enum","variants":[#(#variants),*]})
            )
        }
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Wire supports structs and nonempty externally tagged enums",
            ));
        }
    };
    let name = &input.ident;
    let mut generics = input.generics.clone();
    let mut recursive = false;
    for (field, projection) in collected {
        let mut visitor = RecursiveType { name, found: false };
        visitor.visit_type(&field.ty);
        recursive |= visitor.found;
        // Concrete fields need no where-clause: their calls are checked in the
        // body. Redundant A: Wire / B: Wire clauses on mutually recursive types
        // would instead make trait selection cycle before projection can start.
        if !visitor.found && input.generics.type_params().next().is_some() {
            let ty = &field.ty;
            let bound = match projection.adapter {
                Some(Adapter::Bytes) => quote!(::ic_wire::typescript::SerdeBytesField),
                Some(
                    Adapter::Bools
                    | Adapter::FiniteF32
                    | Adapter::FiniteF64
                    | Adapter::NonzeroI64
                    | Adapter::HeapU64,
                ) => continue,
                None => quote!(::ic_wire::typescript::Wire),
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
mod tests;
