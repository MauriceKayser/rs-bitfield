//! Assertion failure: Field should be signed.

extern crate alloc;

#[bitfield::bitfield(32)]
struct BitField(#[field(size = 16)] Field);

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(i16)]
enum Field {
    F2 = -300
}

fn main() {}