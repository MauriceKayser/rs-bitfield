# Bit fields for Rust

This crate provides the three macros `bitfield`, `Field` and `Flags` (and the additional
convenience macro `FromPrimitive`) to interoperate with low level, typically operating system
related types which store data with sub-byte precision, like boolean flags or sub-byte fields,
in a type-safe, typical rust way.

For more specific documentation look at the documentation of the macros, or at the files in
`examples/*`.