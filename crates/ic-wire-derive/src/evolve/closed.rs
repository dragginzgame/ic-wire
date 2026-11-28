//! Closed enum tags keep their declaration order. Each selected payload is projected
//! through the same Inline implementation as an ordinary field or record.
use super::record;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, GenericParam, Index, Lifetime, LifetimeParam, Path};

pub(crate) fn expand(input: &DeriveInput, owner: &Path) -> syn::Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        unreachable!()
    };
    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "a Wire enum needs at least one variant",
        ));
    }
    let local = |s: &str| crate::local_name(input, s);
    let de = Lifetime::new(&format!("'{}", local("__wire_read")), Span::call_site());
    let stage = Lifetime::new(&format!("'{}", local("__wire_stage")), Span::call_site());
    let proxy = local("__WireVariant");
    let dto = local("__wire_dto");
    let value = local("__wire_value");
    let name = &input.ident;
    let item = local("__wire_item");
    let tail = local("__wire_tail");
    let context = local("__wire_context");
    let scratch = local("__wire_scratch");
    let inline = quote!(#owner::evolution::inline::Inline<#de>);
    let serde = quote!(#owner::evolution::__serde);
    let serde_path = serde.to_string();
    let mut generics = input.generics.clone();
    let mut schema_generics = generics.clone();
    generics
        .params
        .insert(0, GenericParam::Lifetime(LifetimeParam::new(de)));
    let mut payloads = Vec::new();
    let mut declarations = Vec::new();
    let mut variants = Vec::new();
    let mut schema = Vec::new();
    let mut prepares = Vec::new();
    let mut projects = Vec::new();
    for (index, variant) in data.variants.iter().enumerate() {
        if variant.discriminant.is_some() {
            return Err(syn::Error::new_spanned(
                variant,
                "explicit discriminants are not qualified",
            ));
        }
        let tag = &variant.ident;
        let (rename, _) = crate::member_attributes(&variant.attrs, false, None)?;
        let label = rename.unwrap_or_else(|| tag.to_string().trim_start_matches("r#").to_string());
        if matches!(variant.fields, Fields::Unit) {
            variants.push(quote!(#tag));
            prepares.push(quote!(Self::#tag => (#proxy::#tag,#scratch)));
            projects.push(quote!(#proxy::#tag => Self::#tag));
            schema.push(quote!({"name":#label,"id":#index,"value":{"k":"unit"}}));
            continue;
        }
        let param = format_ident!("F{}", payloads.len());
        let newtype =
            matches!(&variant.fields, Fields::Unnamed(fields) if fields.unnamed.len() == 1);
        let bindings: Vec<_> = (0..variant.fields.len())
            .map(|i| local(&format!("__field_{i}")))
            .collect();
        let (ty, pattern, prepare, construct) = if newtype {
            let f = variant.fields.iter().next().unwrap();
            if f.attrs.iter().any(|a| a.path().is_ident("wire")) {
                return Err(syn::Error::new_spanned(
                    f,
                    "newtype variants project their payload directly",
                ));
            }
            if crate::member_attributes(&f.attrs, true, None)?.0.is_some() {
                return Err(syn::Error::new_spanned(
                    f,
                    "renaming an unnamed field is not supported",
                ));
            }
            let ty = &f.ty;
            (
                quote!(#ty),
                quote!(Self::#tag(#item)),
                quote!(<#ty as #inline>::prepare(#item,&#context.child(),#scratch)?),
                quote!(Self::#tag(#item)),
            )
        } else {
            let helper = format_ident!(
                "__WireBody{}Variant{}",
                name.to_string().trim_start_matches("r#"),
                index
            );
            let method = local("__prepare_parts");
            let mut fields = variant.fields.clone();
            let mut arguments = Vec::new();
            let mut parameters = Vec::new();
            for (n, field) in fields.iter_mut().enumerate() {
                let parameter = format_ident!("P{n}");
                if let Some(inner) = record::optional_extension(field)? {
                    arguments.push(inner.clone());
                    field.ty = syn::parse_quote!(::core::option::Option<#parameter>);
                } else {
                    arguments.push(field.ty.clone());
                    field.ty = syn::parse_quote!(#parameter);
                }
                parameters.push(parameter);
            }
            let decl = quote!(#helper <#(#parameters),*>);
            let helper_input: DeriveInput = match fields {
                Fields::Named(_) => syn::parse2(quote!(struct #decl #fields))?,
                _ => syn::parse2(quote!(struct #decl #fields;))?,
            };
            declarations.push(record::variant(&helper_input, owner, &method)?);
            let members: Vec<_> = variant
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    f.ident.as_ref().map_or_else(
                        || {
                            let idx = Index::from(i);
                            quote!(#idx)
                        },
                        |id| quote!(#id),
                    )
                })
                .collect();
            let ty = quote!(#helper <#(#arguments),*>);
            for arg in arguments {
                generics
                    .make_where_clause()
                    .predicates
                    .push(syn::parse_quote!(#arg: #inline));
                schema_generics
                    .make_where_clause()
                    .predicates
                    .push(syn::parse_quote!(#arg: #owner::typescript::Wire));
            }
            (
                ty.clone(),
                quote!(Self::#tag { #(#members: #bindings),* }),
                quote!(<#ty>::#method(&#context.child(),#scratch,#(#bindings),*)?),
                quote!(Self::#tag { #(#members: #item.#members),* }),
            )
        };
        if newtype {
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: #inline));
            schema_generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: #owner::typescript::Wire));
        }
        variants.push(quote!(#tag(#param)));
        prepares
            .push(quote!(#pattern => {let (#item,#tail)=#prepare; (#proxy::#tag(#item),#tail)}));
        projects.push(quote!(#proxy::#tag(#item) => {let #item=<#ty as #inline>::project(#item,&#context.child())?; #construct}));
        schema.push(quote!({"name":#label,"id":#index,"value":#context.describe::<#ty>()?}));
        payloads.push(ty);
    }
    let params = (0..payloads.len()).map(|i| format_ident!("F{i}"));
    let (_, args, _) = input.generics.split_for_impl();
    let (imp, _, bounds) = generics.split_for_impl();
    let (simp, _, sbounds) = schema_generics.split_for_impl();
    let encoded = quote!(#proxy <#(<#payloads as #inline>::Encoded<#stage>),*>);
    let decoded = quote!(#proxy <#(<#payloads as #inline>::Decoded),*>);
    let views = record::views(input, owner, &generics, encoded, decoded, quote!(#value));
    let record::Views {
        declarations: wrappers,
        encoded,
        decoded,
        wrap,
        unwrap,
    } = views;
    Ok(quote! {const _: () = {
        #(#declarations)*
        #[derive(#serde::Serialize,#serde::Deserialize)]
        #[serde(crate = #serde_path)]
        pub enum #proxy <#(#params),*> { #(#variants),* }
        #wrappers
        impl #imp #inline for #name #args #bounds {
            const MAY_STAGE: bool = false #(|| <#payloads as #inline>::MAY_STAGE)*;
            type Encoded<#stage> = #encoded where Self:#stage;
            type Decoded = #decoded;
            fn prepare<#stage>(&#stage self,#context:&#owner::evolution::WriteContext<'_>,#scratch:&#stage mut [u8]) -> ::core::result::Result<(Self::Encoded<#stage>,&#stage mut [u8]),#owner::Error>{
                #context.admit_preparation()?;
                let (#value,#tail)=match self {#(#prepares),*};
                ::core::result::Result::Ok((#wrap,#tail))
            }
            fn project(#dto:Self::Decoded, #context:&#owner::evolution::ReadContext<'_>)->::core::result::Result<Self,#owner::Error>{
                #unwrap
                ::core::result::Result::Ok(match #dto {#(#projects),*})
            }
        }
        #owner::__wire_evolution_schema!{
            impl #simp #owner::typescript::Wire for #name #args #sbounds {
                fn schema(#context:&mut #owner::typescript::SchemaContext)->::core::result::Result<#owner::typescript::__private::Value,#owner::typescript::GenerationError>{
                    ::core::result::Result::Ok(#owner::typescript::__private::json!({"k":"enum","projected":true,"variants":[#(#schema),*]}))
                }
            }
        }
    };})
}
