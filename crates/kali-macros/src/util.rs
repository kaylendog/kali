use quote::{format_ident, quote};

/// Find and return the first field with the given attribute. Returns an error
/// if no such field is found.
pub(crate) fn find_field_with_attr<'a>(
    input: &'a syn::DeriveInput,
    attr_name: &str,
) -> syn::Result<&'a syn::Field> {
    match &input.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => fields
            .iter()
            .filter(|field| field.ident.is_some())
            .find(|field| field.attrs.iter().any(|attr| attr.path().is_ident(attr_name)))
            .map(|f| f)
            .ok_or_else(|| {
                syn::Error::new_spanned(input, format!("Missing #[{}] attribute", attr_name))
            }),
        _ => Err(syn::Error::new_spanned(input, "Expected a struct")),
    }
}

/// Return a token stream that refers to the crate with the given name,
/// returning an error if the crate cannot be found in the current workspace.
pub fn crate_of(name: &str) -> syn::Result<proc_macro2::TokenStream> {
    match proc_macro_crate::crate_name(name) {
        Ok(proc_macro_crate::FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let name = format_ident!("{name}");
            Ok(quote!(::#name))
        }
        Err(_) => Err(syn::Error::new_spanned(
            proc_macro2::Ident::new(name, proc_macro2::Span::call_site()),
            format!("Could not find crate `{name}` in the current workspace"),
        )),
    }
}
