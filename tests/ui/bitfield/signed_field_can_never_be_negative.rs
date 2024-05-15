//! Assertion failure: _SIGNED_TYPE_IN_FIELD_0_CAN_NEVER_BE_NEGATIVE

extern crate alloc;

#[bitfield::bitfield(16)]
struct BitField(#[field(1, 7)] Field); // Needs a size of 8 instead of 7 to store negative values.

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(i8)] // Alternatively, use `u8` instead.
enum Field {
    FMinus1 = -1,
    F1 = 1,
    F2,
    F3
}

fn main() {}