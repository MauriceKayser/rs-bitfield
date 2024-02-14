#[macro_use]
pub(crate) mod parse;
pub(crate) mod generate;

/// Stores all necessary information about a C-like enumeration from a `derive` perspective.
pub struct Enumeration {
    pub repr: syn::Ident,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub variants: Vec<syn::Ident>
}