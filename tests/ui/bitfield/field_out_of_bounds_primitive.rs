//! Assertion failure: _FIELD_0_EXCEEDS_THE_BIT_FIELD_SIZE

extern crate alloc;

#[bitfield::bitfield(size)]
struct BitField(u128); // Can only store bits between 0 - (sizeof(usize) * 8).

fn main() {}