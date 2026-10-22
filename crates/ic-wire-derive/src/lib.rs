//! Explicit-layout derives for IC Wire contracts.
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::collections::BTreeSet;
use syn::{
    Data, DeriveInput, Fields, GenericParam, Lifetime, LifetimeParam, parse_macro_input,
    parse_quote,
};

mod enum_expansion;

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

// Generated local types and generics must not shadow names in the contract.
fn fresh_ident(source: &str, base: &str) -> syn::Ident {
    let mut name = base.to_owned();
    while source.contains(&name) {
        name.push('_');
    }
    format_ident!("{name}")
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if input
        .generics
        .params
        .iter()
        .any(|p| !matches!(p, GenericParam::Lifetime(_)))
        || input.generics.where_clause.is_some()
    {
        return Err(syn::Error::new_spanned(
            input.generics,
            "Wire supports lifetime parameters only; type/const parameters and where clauses are unsupported",
        ));
    }
    if input.attrs.iter().any(|a| a.path().is_ident("wire")) {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "wire ids belong on fields or variants",
        ));
    }
    let source = input.to_token_stream().to_string();
    let serializer = fresh_ident(&source, "S");
    let deserializer = fresh_ident(&source, "D");
    let serializer_binding = fresh_ident(&source, "__ic_wire_serializer");
    let deserializer_binding = fresh_ident(&source, "__ic_wire_deserializer");
    let fields_binding = fresh_ident(&source, "__ic_wire_fields");
    let generics = input.generics;
    let mut decode_name = "__ic_wire_de".to_owned();
    while generics
        .lifetimes()
        .any(|p| p.lifetime.ident == decode_name)
    {
        decode_name.push('_');
    }
    let decode_lifetime = Lifetime::new(&format!("'{decode_name}"), proc_macro2::Span::call_site());
    let mut decode_generics = generics.clone();
    decode_generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(decode_lifetime.clone())),
    );
    for lifetime in generics.lifetimes().map(|p| &p.lifetime) {
        decode_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#decode_lifetime: #lifetime));
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let (decode_impl_generics, _, decode_where_clause) = decode_generics.split_for_impl();
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
            let indices: Vec<_> = (0..fields.len()).map(syn::Index::from).collect();
            let field_ids: Vec<_> = fields.iter().map(|f| f.0).collect();
            (
                quote! { ::ic_wire::__private::serde::Serialize::serialize(&(#(&self.#names,)*), #serializer_binding) },
                quote! {
                    let #fields_binding: (#(#types,)*) = ::ic_wire::__private::serde::Deserialize::deserialize(#deserializer_binding)?;
                    ::core::result::Result::Ok(Self { #(#names: #fields_binding.#indices,)* })
                },
                quote! { ::ic_wire::Shape::record(&[#(<#types as ::ic_wire::Wire>::SHAPE,)*]) },
                quote! { ::ic_wire::schema::Schema::Record(&[#(::ic_wire::schema::Field { name: #labels, id: #field_ids, value: &<#types as ::ic_wire::Wire>::SCHEMA },)*]) },
            )
        }
        Data::Enum(data) => enum_expansion::expand(
            &name,
            &generics,
            &decode_generics,
            &decode_lifetime,
            data,
            &source,
            (&serializer_binding, &deserializer_binding),
        )?,
        Data::Union(_) => return Err(syn::Error::new_spanned(name, "unions are unsupported")),
    };
    Ok(quote! {
        impl #impl_generics ::ic_wire::__private::serde::Serialize for #name #type_generics #where_clause {
            fn serialize<#serializer: ::ic_wire::__private::serde::Serializer>(&self, #serializer_binding: #serializer) -> ::core::result::Result<<#serializer as ::ic_wire::__private::serde::Serializer>::Ok, <#serializer as ::ic_wire::__private::serde::Serializer>::Error> { #serialize }
        }
        impl #decode_impl_generics ::ic_wire::__private::serde::Deserialize<#decode_lifetime> for #name #type_generics #decode_where_clause {
            fn deserialize<#deserializer: ::ic_wire::__private::serde::Deserializer<#decode_lifetime>>(#deserializer_binding: #deserializer) -> ::core::result::Result<Self, <#deserializer as ::ic_wire::__private::serde::Deserializer<#decode_lifetime>>::Error> { #deserialize }
        }
        impl #impl_generics ::ic_wire::Wire for #name #type_generics #where_clause {
            const SHAPE: ::ic_wire::Shape = #shape;
            const SCHEMA: ::ic_wire::schema::Schema = #schema;
        }
    })
}
