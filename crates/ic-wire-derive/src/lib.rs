//! Host metadata for Serde contracts; no serializer generation.
use proc_macro::TokenStream;
use quote::format_ident;
use syn::{DeriveInput, parse_macro_input};

mod native_schema;

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
