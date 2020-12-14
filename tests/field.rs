extern crate alloc;

#[derive(Clone, Copy, Debug, Eq, bitfield::Field, PartialEq)]
#[repr(u16)]
enum B {
    C = 1,
    D = 0xFEDC,
    E,
    F = 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryInto;

    #[test]
    fn try_from() {
        assert_eq!(TryInto::<B>::try_into(0), Err(0));
        assert_eq!(TryInto::<B>::try_into(1), Ok(B::C));
        assert_eq!(TryInto::<B>::try_into(2), Err(2));
        assert_eq!(TryInto::<B>::try_into(3), Err(3));
        assert_eq!(TryInto::<B>::try_into(4), Ok(B::F));
        assert_eq!(TryInto::<B>::try_into(5), Err(5));
        assert_eq!(TryInto::<B>::try_into(0xFEDB), Err(0xFEDB));
        assert_eq!(TryInto::<B>::try_into(0xFEDC), Ok(B::D));
        assert_eq!(TryInto::<B>::try_into(0xFEDD), Ok(B::E));
        assert_eq!(TryInto::<B>::try_into(0xFEDE), Err(0xFEDE));
    }
}