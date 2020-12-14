//! Assertion failure: _FLAGS_IN_FIELD_0_OVERLAP_WITH_FLAGS_IN_FIELD_1

extern crate alloc;

#[bitfield::bitfield(8)]
struct BitField {
    flags: Flags,
    flags2: Flags2
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Flags {
    F0,
    F1,
    F3 = 3 // This flag overlaps with `Flags2::G3`.
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Flags2 {
    G3 = 3, // This flag overlaps with `Flags::F3`.
    G4,
    G5,
    G7 = 7
}

fn main() {}