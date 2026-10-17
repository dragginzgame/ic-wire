use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::{DataEnum, Fields, Ident};

pub(super) fn expand(
    name: &Ident,
    data: DataEnum,
) -> syn::Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
    let mut ids = BTreeSet::new();
    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(name, "empty enum is unsupported"));
    }
    let visitor = format_ident!("__IcWireVisitorFor{}", name);
    let mut serializers = Vec::new();
    let mut decoders = Vec::new();
    let mut shapes = Vec::new();
    let mut variants = Vec::new();
    for variant in data.variants {
        let id = super::identity(&variant.attrs)?;
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
        let (ty, pattern, payload, constructor) = match variant.fields {
            Fields::Unit => (quote!(()), quote!(Self::#v), quote!(()), quote!(#name::#v)),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = &fields.unnamed[0];
                if field.attrs.iter().any(|a| a.path().is_ident("wire")) {
                    return Err(syn::Error::new_spanned(
                        field,
                        "variant payload inherits its variant id",
                    ));
                }
                let ty = &field.ty;
                (
                    quote!(#ty),
                    quote!(Self::#v(value)),
                    quote!(value),
                    quote!(#name::#v(value)),
                )
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    v,
                    "use unit variants or one named-record payload",
                ));
            }
        };
        serializers.push(quote! { #pattern => ::ic_wire::__private::serde::Serialize::serialize(&(#id, #payload), serializer) });
        decoders.push(quote! { #id => {
                    let value: #ty = seq.next_element()?.ok_or_else(|| ::ic_wire::__private::serde::de::Error::custom("missing variant payload"))?;
                    let _ = &value;
                    Ok(#constructor)
                } });
        shapes.push(quote! { <#ty as ::ic_wire::Wire>::SHAPE });
        variants.push(quote! { ::ic_wire::schema::Variant { name: #label, id: #id, value: &<#ty as ::ic_wire::Wire>::SCHEMA } });
    }
    Ok((
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
    ))
}
