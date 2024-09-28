//! Assertion failure: _COMPLETE_FIELD_0_MUST_NOT_HAVE_GAPS

extern crate alloc;

#[bitfield::bitfield(8)]
struct BitField(#[field(size = 2, complete)] Field); // Field value `3` is missing.

#[derive(Clone, Copy, bitfield::Field)]
#[repr(u8)]
enum Field {
    F0,
    F1,
    F2
}

fn main() {}