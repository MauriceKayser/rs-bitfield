//! Assertion failure: _FIELD_0_EXCEEDS_THE_BIT_FIELD_SIZE

extern crate alloc;

#[bitfield::bitfield(size)]
struct BitField(#[field(250, 2)] Field); // Can only store bits between 0 - (sizeof(usize) * 8).

#[derive(Debug, bitfield::Field)]
#[repr(u8)]
enum Field {
    F1 = 1,
    F2,
    F3
}

fn main() {}