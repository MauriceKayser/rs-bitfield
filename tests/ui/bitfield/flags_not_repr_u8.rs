//! Assertion failure: _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8

extern crate alloc;

#[bitfield::bitfield(16)]
struct BitField(Flags);

#[derive(Copy, Clone, Debug)]
#[repr(u16)] // Must be `u8` instead of `u16`.
enum Flags {
    F0,
    F1,
    F3 = 3
}

impl Flags {
    #[allow(unused)]
    #[inline(always)]
    const fn iter() -> &'static [Self] {
        &[Self::F0, Self::F1, Self::F3]
    }

    #[allow(unused)]
    #[inline(always)]
    const fn max() -> Self {
        let mut i = 0;
        let mut max = Flags::F0;

        while i < Self::iter().len() {
            let current = Self::iter()[i];
            if current as u8 > max as u8 {
                max = current;
            }

            i += 1;
        }

        max
    }
}

fn main() {}