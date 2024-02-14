//! This crate serves as the backbone for the [`bitfield`](https://github.com/MauriceKayser/rs-bitfield) crate.

#[cfg(test)]
#[macro_use]
mod test;

pub mod bitfield;
#[macro_use]
pub mod enumeration;
pub mod field;
pub mod flags;
mod primitive;