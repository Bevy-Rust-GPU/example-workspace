#![feature(proc_macro_span)]

extern crate proc_macro;

mod permutate;

use proc_macro::TokenStream;

use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn permutate(attr: TokenStream, item: TokenStream) -> TokenStream {
    permutate::macro_impl(parse_macro_input!(attr), parse_macro_input!(item))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
