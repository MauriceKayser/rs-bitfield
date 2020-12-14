//! Assertion failure: _TYPE_IN_FIELD_0_IS_SMALLER_THAN_THE_SPECIFIED_SIZE_OF_9_BITS

extern crate alloc;

#[bitfield::bitfield(16)]
struct BitField(#[field(0, 9)] Field); // `Field` is only 8 bits wide, and can not store 9 bits.

#[derive(Debug, bitfield::Field)]
#[repr(u8)]
enum Field {
    F1 = 1,
    F2,
    F3
}

fn main() {}