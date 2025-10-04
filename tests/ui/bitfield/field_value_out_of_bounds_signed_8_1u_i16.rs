//! Assertion failure: _TYPE_IN_FIELD_0_EXCEEDS_FIELD_SIZE_OF_1_BIT

extern crate alloc;

#[bitfield::bitfield(8)]
struct BitField(#[field(size = 1)] Field);

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(i16)]
enum Field {
    F2 = -300
}

fn main() {}