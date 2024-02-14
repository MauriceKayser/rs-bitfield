//! Contains code to parse bit field fields.

impl super::Field {
    pub fn parse(item: proc_macro2::TokenStream) -> syn::Result<Self> {
        Ok(Self(crate::enumeration::Enumeration::parse(item)?))
    }
}