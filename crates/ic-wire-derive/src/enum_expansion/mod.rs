use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::{DataEnum, Fields, Generics, Ident, Lifetime};

pub(super) fn expand(
    name: &Ident,
    generics: &Generics,
    decode_generics: &Generics,
    decode_lifetime: &Lifetime,
    data: DataEnum,
    source: &str,
    (serializer, deserializer): (&Ident, &Ident),
) -> syn::Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
    let (_, type_generics, _) = generics.split_for_impl();
    let (decode_impl_generics, _, decode_where_clause) = decode_generics.split_for_impl();
    let mut ids = BTreeSet::new();
    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(name, "empty enum is unsupported"));
    }
    let visitor = super::fresh_ident(
        source,
        &format_ident!("__IcWireVisitorFor{}", name).to_string(),
    );
    let access = super::fresh_ident(source, "A");
    let value = super::fresh_ident(source, "__ic_wire_value");
    let tag = super::fresh_ident(source, "__ic_wire_tag");
    let seq = super::fresh_ident(source, "__ic_wire_seq");
    let formatter = super::fresh_ident(source, "__ic_wire_formatter");
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
        // Infer payloads from their constructors: `Self` in a consumer type
        // denotes the enum, but would denote the visitor in its methods.
        let decode_ty = match &variant.fields {
            Fields::Unit => quote!(()),
            _ => quote!(_),
        };
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
                    quote!(Self::#v(#value)),
                    quote!(#value),
                    quote!(#name::#v(#value)),
                )
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    v,
                    "use unit variants or one named-record payload",
                ));
            }
        };
        serializers.push(quote! { #pattern => ::ic_wire::__private::serde::Serialize::serialize(&(#id, #payload), #serializer) });
        decoders.push(quote! { #id => {
                    let #value: #decode_ty = ::core::option::Option::ok_or_else(::ic_wire::__private::serde::de::SeqAccess::next_element(&mut #seq)?, || ::ic_wire::__private::serde::de::Error::custom("missing variant payload"))?;
                    ::core::result::Result::Ok(#constructor)
                } });
        shapes.push(quote! { <#ty as ::ic_wire::Wire>::SHAPE });
        variants.push(quote! { ::ic_wire::schema::Variant { name: #label, id: #id, value: &<#ty as ::ic_wire::Wire>::SCHEMA } });
    }
    Ok((
        quote! {
            match self { #(#serializers,)* }
        },
        quote! {
            struct #visitor #generics (::core::marker::PhantomData<fn() -> #name #type_generics>);
            impl #decode_impl_generics ::ic_wire::__private::serde::de::Visitor<#decode_lifetime> for #visitor #type_generics #decode_where_clause {
                type Value = #name #type_generics;
                fn expecting(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result { ::core::fmt::Formatter::write_str(#formatter, "explicit wire variant") }
                fn visit_seq<#access: ::ic_wire::__private::serde::de::SeqAccess<#decode_lifetime>>(self, mut #seq: #access) -> ::core::result::Result<<Self as ::ic_wire::__private::serde::de::Visitor<#decode_lifetime>>::Value, <#access as ::ic_wire::__private::serde::de::SeqAccess<#decode_lifetime>>::Error> {
                    let #tag: ::core::primitive::u32 = ::core::option::Option::ok_or_else(::ic_wire::__private::serde::de::SeqAccess::next_element(&mut #seq)?, || ::ic_wire::__private::serde::de::Error::custom("missing variant id"))?;
                    match #tag { #(#decoders,)* _ => ::core::result::Result::Err(::ic_wire::__private::serde::de::Error::custom("unknown variant id")) }
                }
            }
            ::ic_wire::__private::serde::Deserializer::deserialize_tuple(#deserializer, 2, #visitor(::core::marker::PhantomData))
        },
        quote! { ::ic_wire::Shape::choice(5, &[#(#shapes,)*]) },
        quote! { ::ic_wire::schema::Schema::Enum(&[#(#variants,)*]) },
    ))
}
