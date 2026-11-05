//! Attribute generation owned and released with IC Wire Endpoint.
//! Import these attributes through `ic_wire_endpoint`.
use proc_macro::TokenStream;
mod endpoint;

/// Connect a typed Wire query to Platform's maintained entrypoint generator.
#[proc_macro_attribute]
pub fn query(args: TokenStream, item: TokenStream) -> TokenStream {
    endpoint::expand(true, args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Connect a typed Wire update to Platform's maintained entrypoint generator.
#[proc_macro_attribute]
pub fn update(args: TokenStream, item: TokenStream) -> TokenStream {
    endpoint::expand(false, args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
