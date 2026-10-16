//! Evaluation-only attribute expansion; Canic remains the production macro owner.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, Path, parse::Parser};

pub(crate) fn expand(
    args: TokenStream,
    item: TokenStream,
    update: bool,
) -> syn::Result<TokenStream> {
    let (mut method, mut guard): (Option<Path>, Option<Path>) = (None, None);
    syn::meta::parser(|meta| {
        let slot = if meta.path.is_ident("method") {
            &mut method
        } else if meta.path.is_ident("guard") {
            &mut guard
        } else {
            return Err(meta.error("expected method or guard; no codec selector"));
        };
        if slot.is_some() {
            return Err(meta.error("duplicate adapter option"));
        }
        *slot = Some(meta.value()?.parse()?);
        Ok(())
    })
    .parse2(args)?;
    let method = method.ok_or_else(|| syn::Error::new_spanned(&item, "method binding required"))?;
    let guard = guard.ok_or_else(|| syn::Error::new_spanned(&item, "request guard required"))?;
    let function: ItemFn = syn::parse2(item)?;
    let sig = &function.sig;
    if !sig.generics.params.is_empty()
        || sig.generics.where_clause.is_some()
        || sig.inputs.len() != 1
        || !matches!(sig.inputs.first(), Some(syn::FnArg::Typed(_)))
        || sig.unsafety.is_some()
        || sig.constness.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
    {
        return Err(syn::Error::new_spanned(
            sig,
            "one concrete safe request handler required",
        ));
    }
    if !update && sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            sig,
            "async query is outside this fixture",
        ));
    }
    let name = &sig.ident;
    let export = name.to_string();
    let entry = format_ident!("__ic_wire_entry_{name}");
    let kind = if update {
        quote!(update)
    } else {
        quote!(query)
    };
    let asynchronous = &sig.asyncness;
    let await_handler = asynchronous.map(|_| quote!(.await));
    let body = quote! {
        // Deliberate fixture order, not an assertion about Canic's current stages.
        transport::startup().map_err(str::to_owned)?;
        if transport::argument_size() > <#method as ::ic_wire::Method>::REQUEST_LIMITS.bytes {
            return Err("request limit".to_owned());
        }
        transport::authorize_caller().map_err(str::to_owned)?;
        let bytes = transport::arguments();
        let request = ::ic_wire::decode_request::<#method>(&bytes).map_err(|e| e.to_string())?;
        #guard(&request).map_err(str::to_owned)?;
        let response: <#method as ::ic_wire::Method>::Response = #name(request) #await_handler;
        match ::ic_wire::encode_response::<#method>(&response) {
            Ok(reply) => Ok(reply),
            Err(error) => transport::trap(&format!("response codec: {error}")),
        }
    };
    let execute = if asynchronous.is_some() {
        quote!((async { #body }).await)
    } else {
        quote!((|| { #body })())
    };
    Ok(quote! {
        #function
        const _: () = {
            let declared = <#method as ::ic_wire::Method>::NAME.as_bytes();
            let exported = #export.as_bytes();
            assert!(declared.len() == exported.len(), "method/export name mismatch");
            let mut index = 0;
            while index < declared.len() {
                assert!(declared[index] == exported[index], "method/export name mismatch");
                index += 1;
            }
        };
        #[allow(dead_code)] // Native tests invoke the same entry without IC exports.
        #[cfg_attr(target_arch = "wasm32", ::ic_cdk::#kind(name = #export, manual_reply = true))]
        #asynchronous fn #entry() {
            let result: Result<::ic_wire::ResponseBytes<#method>, String> = #execute;
            match result {
                Ok(bytes) => transport::reply(bytes.as_bytes()),
                Err(error) => transport::reject(&error),
            }
        }
    })
}
