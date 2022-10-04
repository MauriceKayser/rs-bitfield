//! Assertion failure: _FIELD_0_HAS_THE_SIZE_OF_THE_WHOLE_BIT_FIELD

extern crate alloc;

#[bitfield::bitfield(size)]
struct BitField(#[field(0, 64)] Field); // Uses the whole bit field, a use a plain `Field` instead.

#[derive(Debug, bitfield::Field)]
#[repr(u64)]
enum Field {
    F1 = 1,
    F2,
    F3
}

fn main() {}