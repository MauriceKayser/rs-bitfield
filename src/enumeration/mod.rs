#[macro_use]
pub(crate) mod parse;
pub(crate) mod generate;

/// Stores all necessary information about a C-like enumeration from a `derive` perspective.
pub(crate) struct Enumeration {
    pub(crate) repr: syn::Ident,
    pub(crate) vis: syn::Visibility,
    pub(crate) ident: syn::Ident,
    pub(crate) variants: Vec<syn::Ident>
}