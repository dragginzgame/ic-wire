//! Explicit-layout derives for IC Wire contracts.
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive a bounded record or tagged enum. Fields/variants require `#[wire(id = N)]`.
#[proc_macro_derive(Wire, attributes(wire))]
pub fn derive_wire(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn identity(attrs: &[syn::Attribute]) -> syn::Result<u32> {
    let mut id = None;
    for attr in attrs.iter().filter(|a| a.path().is_ident("wire")) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("id") || id.is_some() {
                return Err(meta.error("expected one wire id"));
            }
            id = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        })?;
    }
    id.ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "explicit #[wire(id = N)] required",
        )
    })
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "Wire requires concrete contracts",
        ));
    }
    if input.attrs.iter().any(|a| a.path().is_ident("wire")) {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "wire ids belong on fields or variants",
        ));
    }
    let name = input.ident;
    let mut ids = BTreeSet::new();
    let (serialize, deserialize, shape, schema) = match input.data {
        Data::Struct(data) => {
            let Fields::Named(fields) = data.fields else {
                return Err(syn::Error::new_spanned(
                    name,
                    "use a named-field record (empty records are allowed)",
                ));
            };
            if fields.named.len() > 16 {
                return Err(syn::Error::new_spanned(
                    name,
                    "at most 16 fields; group related fields in a nested record",
                ));
            }
            let mut fields = fields
                .named
                .into_iter()
                .map(|field| {
                    let id = identity(&field.attrs)?;
                    if !ids.insert(id) {
                        return Err(syn::Error::new_spanned(&field, "duplicate wire id"));
                    }
                    Ok((id, field.ident.unwrap(), field.ty))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            fields.sort_by_key(|f| f.0);
            let names: Vec<_> = fields.iter().map(|f| &f.1).collect();
            let types: Vec<_> = fields.iter().map(|f| &f.2).collect();
            let labels: Vec<_> = names
                .iter()
                .map(|name| name.to_string().trim_start_matches("r#").to_owned())
                .collect();
            let field_ids: Vec<_> = fields.iter().map(|f| f.0).collect();
            (
                quote! { ::ic_wire::__private::serde::Serialize::serialize(&(#(&self.#names,)*), serializer) },
                quote! {
                    let (#(#names,)*): (#(#types,)*) = ::ic_wire::__private::serde::Deserialize::deserialize(deserializer)?;
                    Ok(Self { #(#names,)* })
                },
                quote! { ::ic_wire::Shape::record(&[#(<#types as ::ic_wire::Wire>::SHAPE,)*]) },
                quote! { ::ic_wire::schema::Schema::Record(&[#(::ic_wire::schema::Field { name: #labels, id: #field_ids, value: &<#types as ::ic_wire::Wire>::SCHEMA },)*]) },
            )
        }
        Data::Enum(data) => {
            if data.variants.is_empty() {
                return Err(syn::Error::new_spanned(name, "empty enum is unsupported"));
            }
            let visitor = format_ident!("__IcWireVisitorFor{}", name);
            let mut serializers = Vec::new();
            let mut decoders = Vec::new();
            let mut shapes = Vec::new();
            let mut variants = Vec::new();
            for variant in data.variants {
                let id = identity(&variant.attrs)?;
                if !ids.insert(id) {
                    return Err(syn::Error::new_spanned(&variant, "duplicate wire id"));
                }
                if variant.discriminant.is_some() {
                    return Err(syn::Error::new_spanned(
                        &variant,
                        "use wire id, not Rust discriminants",
                    ));
                }
                let v = variant.ident;
                let label = v.to_string().trim_start_matches("r#").to_owned();
                match variant.fields {
                    Fields::Unit => {
                        serializers.push(quote! { Self::#v => ::ic_wire::__private::serde::Serialize::serialize(&(#id, ()), serializer) });
                        decoders.push(quote! { #id => { let _: () = seq.next_element()?.ok_or_else(|| ::ic_wire::__private::serde::de::Error::custom("missing variant payload"))?; Ok(#name::#v) } });
                        shapes.push(quote! { <() as ::ic_wire::Wire>::SHAPE });
                        variants.push(quote! { ::ic_wire::schema::Variant { name: #label, id: #id, value: &<() as ::ic_wire::Wire>::SCHEMA } });
                    }
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        let field = &fields.unnamed[0];
                        if field.attrs.iter().any(|a| a.path().is_ident("wire")) {
                            return Err(syn::Error::new_spanned(
                                field,
                                "variant payload inherits its variant id",
                            ));
                        }
                        let ty = &field.ty;
                        serializers.push(quote! { Self::#v(value) => ::ic_wire::__private::serde::Serialize::serialize(&(#id, value), serializer) });
                        decoders.push(quote! { #id => Ok(#name::#v(seq.next_element::<#ty>()?.ok_or_else(|| ::ic_wire::__private::serde::de::Error::custom("missing variant payload"))?)) });
                        shapes.push(quote! { <#ty as ::ic_wire::Wire>::SHAPE });
                        variants.push(quote! { ::ic_wire::schema::Variant { name: #label, id: #id, value: &<#ty as ::ic_wire::Wire>::SCHEMA } });
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            v,
                            "use unit variants or one named-record payload",
                        ));
                    }
                }
            }
            (
                quote! { match self { #(#serializers,)* } },
                quote! {
                    struct #visitor;
                    impl<'de> ::ic_wire::__private::serde::de::Visitor<'de> for #visitor {
                        type Value = #name;
                        fn expecting(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result { f.write_str("explicit wire variant") }
                        fn visit_seq<A: ::ic_wire::__private::serde::de::SeqAccess<'de>>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error> {
                            let tag: u32 = seq.next_element()?.ok_or_else(|| ::ic_wire::__private::serde::de::Error::custom("missing variant id"))?;
                            match tag { #(#decoders,)* _ => Err(::ic_wire::__private::serde::de::Error::custom("unknown variant id")) }
                        }
                    }
                    deserializer.deserialize_tuple(2, #visitor)
                },
                quote! { ::ic_wire::Shape::choice(5, &[#(#shapes,)*]) },
                quote! { ::ic_wire::schema::Schema::Enum(&[#(#variants,)*]) },
            )
        }
        Data::Union(_) => return Err(syn::Error::new_spanned(name, "unions are unsupported")),
    };
    Ok(quote! {
        impl ::ic_wire::__private::serde::Serialize for #name {
            fn serialize<S: ::ic_wire::__private::serde::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> { #serialize }
        }
        impl<'de> ::ic_wire::__private::serde::Deserialize<'de> for #name {
            fn deserialize<D: ::ic_wire::__private::serde::Deserializer<'de>>(deserializer: D) -> ::core::result::Result<Self, D::Error> { #deserialize }
        }
        impl ::ic_wire::Wire for #name {
            const SHAPE: ::ic_wire::Shape = #shape;
            const SCHEMA: ::ic_wire::schema::Schema = #schema;
        }
    })
}

#[cfg(feature = "adapter-fixture")]
mod adapter_fixture;

/// Evaluation-only query adapter; not a supported Canic macro API.
#[cfg(feature = "adapter-fixture")]
#[proc_macro_attribute]
pub fn query(args: TokenStream, item: TokenStream) -> TokenStream {
    adapter_fixture::expand(args.into(), item.into(), false)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Evaluation-only sync/async update adapter; not a supported Canic macro API.
#[cfg(feature = "adapter-fixture")]
#[proc_macro_attribute]
pub fn update(args: TokenStream, item: TokenStream) -> TokenStream {
    adapter_fixture::expand(args.into(), item.into(), true)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
