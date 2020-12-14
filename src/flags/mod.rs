//! Contains all data types to represent flags in a bit field.

#[macro_use]
pub(super) mod parse;
pub(super) mod generate;

/// Stores all information about a flag type of a bit field.
pub(super) struct Flags(crate::enumeration::Enumeration);