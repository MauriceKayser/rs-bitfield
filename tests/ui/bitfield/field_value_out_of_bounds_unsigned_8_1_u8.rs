//! Assertion failure: _TYPE_IN_FIELD_0_EXCEEDS_FIELD_SIZE_OF_1_BIT

extern crate alloc;

#[bitfield::bitfield(8)]
struct BitField(#[field(size = 1)] Field); // Can only store values between `0..=1`.

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Field {
    F2 = 2
}

fn main() {}