use proc_macro2::TokenStream;
use quote::quote;

use crate::util::{crate_of, find_field_with_attr};

/// Implement the `kali_ast::Node` trait for a struct, using the specified
/// fields for the `id` and `span` methods.
pub(crate) fn expand_node(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let id = find_field_with_attr(&input, "id")?.ident.as_ref().unwrap();
    let span = find_field_with_attr(&input, "span")?.ident.as_ref().unwrap();
    let kali_ast = crate_of("kali-ast")?;
    Ok(quote! {
        impl #kali_ast::Node for #name {
            fn id(&self) -> #kali_ast::NodeId {
                self.#id
            }

            fn span(&self) -> &#kali_ast::Span {
                &self.#span
            }
        }
    })
}
