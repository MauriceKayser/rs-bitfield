//! Assertion failure: _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE

extern crate alloc;

#[bitfield::bitfield(8)] // Can only store flags between 0 - 7.
struct BitField(Flags);

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Flags {
    F0,
    F8 = 8 // This flag is too high to store it in `BitField`.
}

fn main() {}