//! Wire codec hooks only; Platform owns export and execution generation.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, FnArg, ItemFn, LitStr, Path, ReturnType, ext::IdentExt, parse::Parser};

pub(super) fn expand(
    query: bool,
    args: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let source = format!("{args} {item}");
    let function: ItemFn = syn::parse2(item)?;
    let mut id: Option<Expr> = None;
    let mut request: Option<Expr> = None;
    let mut response: Option<Expr> = None;
    let mut name: Option<LitStr> = None;
    let mut contract: Option<syn::Ident> = None;
    // Package move only: default expansion now targets the independent endpoint
    // facade. Consumers move attribute imports; frame bytes and settings stay the same.
    let mut wire: Path = syn::parse_quote!(::ic_wire_endpoint);
    let mut composite = false;
    let mut borrow = false;
    let mut seen = std::collections::BTreeSet::new();
    syn::meta::parser(|meta| {
        let ident = meta
            .path
            .get_ident()
            .ok_or_else(|| meta.error("expected an endpoint option"))?;
        let key = ident.to_string();
        if !seen.insert(key.clone()) {
            return Err(meta.error("duplicate Wire endpoint option"));
        }
        match key.as_str() {
            "id" => id = Some(meta.value()?.parse()?),
            "request" => request = Some(meta.value()?.parse()?),
            "response" => response = Some(meta.value()?.parse()?),
            "name" => name = Some(meta.value()?.parse()?),
            "contract" => contract = Some(meta.value()?.parse()?),
            "crate" => wire = meta.value()?.parse::<LitStr>()?.parse()?,
            "composite" if query => composite = true,
            "borrow" => borrow = true,
            _ => return Err(meta.error("unsupported Wire endpoint option")),
        }
        Ok(())
    })
    .parse2(args)?;
    let signature = &function.sig;
    if signature.inputs.len() > 1
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || signature.constness.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !matches!(signature.safety, syn::Safety::Default)
    {
        return Err(syn::Error::new_spanned(
            signature,
            "Wire endpoints require a safe, nongeneric function with zero or one owned request value",
        ));
    }
    let function_name = &signature.ident;
    let export = name
        .unwrap_or_else(|| LitStr::new(&function_name.unraw().to_string(), function_name.span()));
    if export.value().is_empty()
        || export.value().len() > u16::MAX as usize
        || export.value().chars().any(char::is_control)
    {
        return Err(syn::Error::new_spanned(export, "invalid Wire method name"));
    }
    // Omitted settings previously failed compilation. They now select bounded
    // endpoint defaults and a stable exported-name identity, evaluated as consts.
    // Explicit settings keep their exact frames/admission. Dropping an existing
    // ID or limits can change the contract: coordinate clients, or retain them.
    let id = id.map_or_else(
        || quote!(#wire::native::method_id(#export)),
        |value| quote!(#value),
    );
    let request = request.map_or_else(
        || quote!(#wire::DEFAULT_REQUEST_LIMITS),
        |value| quote!(#value),
    );
    let response = response.map_or_else(
        || quote!(#wire::DEFAULT_RESPONSE_LIMITS),
        |value| quote!(#value),
    );
    let (request_type, takes_argument) = match signature.inputs.first() {
        Some(FnArg::Typed(arg)) => {
            let ty = &arg.ty;
            if !borrow && matches!(ty.as_ref(), syn::Type::Reference(_)) {
                return Err(syn::Error::new_spanned(
                    ty,
                    "Wire endpoints require an owned request value",
                ));
            }
            (quote!(#ty), true)
        }
        Some(arg) => {
            return Err(syn::Error::new_spanned(
                arg,
                "Wire endpoints cannot receive self",
            ));
        }
        None => (quote!(()), false),
    };
    if borrow && (signature.asyncness.is_some() || !takes_argument) {
        return Err(syn::Error::new_spanned(
            signature,
            "borrow requires a synchronous handler with one request value",
        ));
    }
    let response_type = match &signature.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };
    // Keep all user types/expressions in their original lexical scope. An injected
    // module with `use super::*` would shadow DTOs named Request or constants ID.
    let value = crate::fresh_ident(&source, "__wire_value");
    let argument = takes_argument.then(|| quote!(#value));
    let asynchronous = &signature.asyncness;
    let await_result = asynchronous.map(|_| quote!(.await));
    let attribute = if query { quote!(query) } else { quote!(update) };
    let mode = if composite {
        "composite_query"
    } else if query {
        "query"
    } else {
        "update"
    };
    let composite = composite.then(|| quote!(composite,));
    let cdk_path = LitStr::new(
        &format!("{}::__private::cdk", quote!(#wire)),
        function_name.span(),
    );
    let visibility = &function.vis;
    let cfg: Vec<_> = function
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"))
        .collect();
    let settings = crate::fresh_ident(&source, "__WIRE_CONTRACT");
    let declaration = quote!(#wire::Contract::<#request_type, #response_type>::new(
        #export, #mode, #id, #request, #response
    ));
    let (metadata, settings_value) = if let Some(contract) = contract {
        (
            quote! {
                #(#cfg)*
                #[doc = "Wire contract derived from the handler; no registry or artifact admission is implied."]
                #visibility const #contract: #wire::Contract<#request_type, #response_type> = #declaration;
            },
            quote!(#contract),
        )
    } else {
        (quote!(), declaration)
    };
    let admit = crate::fresh_ident(&source, "__wire_admit");
    let decode = crate::fresh_ident(&source, "__wire_decode");
    let encode = crate::fresh_ident(&source, "__wire_encode");
    let entry = crate::fresh_ident(&source, "__wire_entry");
    let result_type = crate::fresh_ident(&source, "__WireResponse");
    let admit_path = LitStr::new(&admit.to_string(), admit.span());
    let decode_path = LitStr::new(&decode.to_string(), decode.span());
    let encode_path = LitStr::new(&encode.to_string(), encode.span());
    if borrow {
        let bytes = crate::fresh_ident(&source, "__wire_bytes");
        let reply = crate::fresh_ident(&source, "__wire_reply");
        // This ownership scope retains the original argument buffer through reply
        // encoding. Frame bytes, budgets and complete-input validation are unchanged.
        // Platform still owns copying, exports, guards and the execution context.
        return Ok(quote! {
            #function
            #metadata
            #(#cfg)*
            const _: () = {
                const #settings: #wire::Contract<#request_type, #response_type> = #settings_value;
                fn #admit() -> ::core::result::Result<(), ::std::string::String> {
                    #settings.admit_request();
                    Ok(())
                }
                fn #decode(bytes: ::std::vec::Vec<u8>) -> ::std::vec::Vec<u8> { bytes }
                #[#wire::__private::cdk::#attribute(
                    name = #export, #composite crate = #cdk_path,
                    guard = #admit_path, decode_with = #decode_path, encode_with = #decode_path
                )]
                fn #entry(#bytes: ::std::vec::Vec<u8>) -> ::std::vec::Vec<u8> {
                    let #value: #request_type = #wire::decode_request(
                        &#bytes, #settings.id, #settings.request);
                    let #reply = #function_name(#value);
                    #wire::encode_response(&#reply, #settings.id, #settings.response)
                }
            };
        });
    }
    Ok(quote! {
        #function
        #metadata
        #(#cfg)*
        const _: () = {
            type #result_type = #response_type;
            const #settings: #wire::Contract<#request_type, #response_type> = #settings_value;
            fn #admit() -> ::core::result::Result<(), ::std::string::String> {
                #settings.admit_request();
                Ok(())
            }
            fn #decode(bytes: ::std::vec::Vec<u8>) -> #request_type {
                #wire::decode_request(&bytes, #settings.id, #settings.request)
            }
            fn #encode(value: #response_type) -> ::std::vec::Vec<u8> {
                #wire::encode_response(&value, #settings.id, #settings.response)
            }
            #[#wire::__private::cdk::#attribute(
                name = #export, #composite crate = #cdk_path,
                guard = #admit_path, decode_with = #decode_path, encode_with = #encode_path
            )]
            #asynchronous fn #entry(#value: #request_type) -> #result_type {
                let _ = &#value;
                // Give Platform one result value even for unit and tuple responses.
                // Its encoder arity follows syntax, so the alias preserves unit.
                #function_name(#argument)#await_result
            }
        };
    })
}
