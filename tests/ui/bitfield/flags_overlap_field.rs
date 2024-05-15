//! Assertion failure: _FLAGS_IN_FIELD_1_OVERLAP_WITH_FIELD_0

extern crate alloc;

#[bitfield::bitfield(8)]
struct BitField {
    #[field(0, 2)] // This field overlaps with the flags F1, F2 & F3.
    field: Field,
    flags: Flags
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Field {
    F1 = 1,
    F2,
    F3
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Flags {
    F0,
    F1,
    F3 = 3
}

fn main() {}