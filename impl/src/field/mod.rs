//! Contains all data types to represent a field in a bit field.

pub(super) mod generate;
pub(super) mod parse;

/// Stores all information about a field of a bit field.
pub struct Field(pub crate::enumeration::Enumeration);