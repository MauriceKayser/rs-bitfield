//! Assertion failure: _TYPE_IN_FIELD_0_EXCEEDS_FIELD_SIZE_OF_9_BITS

extern crate alloc;

#[bitfield::bitfield(16)]
struct BitField(#[field(size = 9, signed)] Field);

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(i16)]
enum Field {
    F2 = -300
}

fn main() {}