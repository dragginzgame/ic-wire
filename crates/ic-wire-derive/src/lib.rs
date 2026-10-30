//! Host metadata for Serde contracts; no serializer generation.
use proc_macro::TokenStream;
use quote::format_ident;
use syn::{DeriveInput, parse_macro_input};

#[cfg(feature = "endpoint")]
mod endpoint;
mod native_schema;

/// Connect a typed Wire query to Platform's maintained entrypoint generator.
#[cfg(feature = "endpoint")]
#[proc_macro_attribute]
pub fn query(args: TokenStream, item: TokenStream) -> TokenStream {
    endpoint::expand(true, args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Connect a typed Wire update to Platform's maintained entrypoint generator.
#[cfg(feature = "endpoint")]
#[proc_macro_attribute]
pub fn update(args: TokenStream, item: TokenStream) -> TokenStream {
    endpoint::expand(false, args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Describe ordinary Serde layouts for host-side TypeScript generation, without IDs.
/// Emits metadata implementations on native targets only; Serde owns runtime codecs.
#[proc_macro_derive(Wire, attributes(serde, wire))]
pub fn derive_wire(input: TokenStream) -> TokenStream {
    native_schema::expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

// Generated local types and generics must not shadow names in the contract.
fn fresh_ident(source: &str, base: &str) -> syn::Ident {
    let mut name = base.to_owned();
    while source.contains(&name) {
        name.push('_');
    }
    format_ident!("{name}")
}
