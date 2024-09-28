//! Contains all data types to represent bit fields.

#[macro_use]
pub(super) mod parse;
pub(super) mod generate;

/// Stores the information that is transmitted via the proc-macro attribute header.
pub struct Attribute {
    pub base_type: syn::Path,
    pub primitive_type: syn::Ident,
    /// `None` for `isize` and `usize`.
    pub bits: Option<u8>,
    pub is_non_zero: bool,
    pub allow_overlaps: Option<syn::Ident>
}

/// Stores all information about a bit field, which is parsed from a struct with named fields, or a
/// tuple struct with one element.
pub struct BitField {
    pub attr: Attribute,
    pub debug: Option<proc_macro2::Span>,
    pub display: Option<proc_macro2::Span>,
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub data: Data
}

/// Stores the parsed data from either from a struct with named fields, or a tuple struct.
/// The tuple struct only supports one tuple entry and it should be used for simple bit fields.
pub enum Data {
    Named(Vec<EntryNamed>),
    Tuple(Entry)
}

impl Data {
    /// Easy access to all entries, regardless of the struct type.
    pub fn entries(&self) -> Vec<&Entry> {
        match self {
            Self::Named(entries) => entries.iter().map(
                |e| &e.entry
            ).collect(),

            Self::Tuple(entry) => vec!(entry)
        }
    }

    /// Easy, mutable access to all entries, regardless of the struct type.
    pub fn entries_mut(&mut self) -> Vec<&mut Entry> {
        match self {
            Self::Named(entries) => entries.iter_mut().map(
                |e| &mut e.entry
            ).collect(),

            Self::Tuple(entry) => vec!(entry)
        }
    }
}

/// Stores an unnamed bit field entry which can be a field or flags. If `field` is `None`, then `ty`
/// references flags, otherwise `field` describes the field information.
pub struct Entry {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ty: syn::Path,
    pub field: Option<FieldDetails>
}

/// Stores a bit field entry from a struct with named fields.
pub struct EntryNamed {
    pub ident: syn::Ident,
    pub entry: Entry
}

/// Stores details about the boundaries of a field.
pub struct FieldDetails {
    /// Span of `bit, size`. Used for out of bounds error reporting.
    pub span: proc_macro2::Span,
    /// This must never be `None` after parsing.
    pub bit: Option<syn::LitInt>,
    /// This must never be `None` after parsing.
    pub size: Option<syn::LitInt>,
    pub complete: Option<syn::Ident>,
    pub signed: Option<syn::Ident>
}