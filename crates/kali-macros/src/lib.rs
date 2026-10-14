//! Macros for the `kali-ast` crate.

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::ast::expand_node;

mod ast;
mod util;

/// Generates an implementation of the `kali_ast::Node` trait for a struct,
/// using the specified fields for the `id` and `span` methods.
#[proc_macro_derive(Node, attributes(id, span))]
pub fn derive_node(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item);
    expand_node(input).unwrap_or_else(syn::Error::into_compile_error).into()
}
