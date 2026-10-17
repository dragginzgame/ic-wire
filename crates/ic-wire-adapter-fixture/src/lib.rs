//! Private endpoint attributes for IC Wire qualification.
use proc_macro::TokenStream;

mod adapter_fixture;

/// Evaluation-only query adapter; not a supported Canic macro API.
#[proc_macro_attribute]
pub fn query(args: TokenStream, item: TokenStream) -> TokenStream {
    adapter_fixture::expand(args.into(), item.into(), false)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Evaluation-only sync/async update adapter; not a supported Canic macro API.
#[proc_macro_attribute]
pub fn update(args: TokenStream, item: TokenStream) -> TokenStream {
    adapter_fixture::expand(args.into(), item.into(), true)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
