# Bit fields for Rust

This crate provides the three macros `bitfield`, `Field` and `Flags` (and the additional
convenience macro `FromPrimitive`) to interoperate with low level, typically operating system
related types which store data with sub-byte precision, like boolean flags or sub-byte fields,
in a type-safe, typical rust way.

It supports:
- `bool`s and C-like enums as bit flags + enumerability over flags, if C-like enums are used
- Primitive types and C-like enums as multi-bit fields
- Explicit and implicit positioning and sizing of fields and flags
- Optional `core::fmt::Debug` and `core::fmt::Display` implementations
- Compile-time overlap and boundary checking.

For more specific documentation look at the documentation of the macros, or at the files in
`examples/*`.