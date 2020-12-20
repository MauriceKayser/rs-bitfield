//! Contains all data types to represent bit fields.

#[macro_use]
pub(super) mod parse;
pub(super) mod generate;

/// Stores the information that is transmitted via the proc-macro attribute header.
struct Attribute {
    size: syn::Ident,
    bits: Option<u8>,
    allow_overlaps: Option<syn::Ident>
}

/// Stores all information about a bit field, which is parsed from a struct with named fields, or a
/// tuple struct with one element.
pub(super) struct BitField {
    attr: Attribute,
    debug: Option<proc_macro2::Span>,
    display: Option<proc_macro2::Span>,
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    ident: syn::Ident,
    data: Data
}

/// Stores the parsed data from either from a struct with named fields, or a tuple struct.
/// The tuple struct only supports one tuple entry and it should be used for simple bit fields.
enum Data {
    Named(Vec<EntryNamed>),
    Tuple(Entry)
}

impl Data {
    /// Easy access to all entries, regardless of the struct type.
    fn entries(&self) -> Vec<&Entry> {
        match self {
            Self::Named(entries) => entries.iter().map(
                |e| &e.entry
            ).collect(),

            Self::Tuple(entry) => vec!(entry)
        }
    }

    /// Easy, mutable access to all entries, regardless of the struct type.
    fn entries_mut(&mut self) -> Vec<&mut Entry> {
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
struct Entry {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    ty: syn::Path,
    field: Option<FieldDetails>
}

/// Stores a bit field entry from a struct with named fields.
struct EntryNamed {
    ident: syn::Ident,
    entry: Entry
}

/// Stores details about the boundaries of a field.
struct FieldDetails {
    /// Span of `bit, size`. Used for out of bounds error reporting.
    span: proc_macro2::Span,
    /// This must never be `None` after parsing.
    bit: Option<syn::LitInt>,
    /// This must never be `None` after parsing.
    size: Option<syn::LitInt>,
    signed: Option<syn::Ident>
}