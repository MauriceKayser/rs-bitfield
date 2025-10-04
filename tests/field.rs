extern crate alloc;

#[derive(Clone, Copy, Debug, Eq, bitfield::Field, PartialEq)]
#[repr(u16)]
enum B {
    C = 1,
    D = 0xFEDC,
    E,
    F = 4
}

#[derive(Clone, Copy, Debug, Eq, bitfield::Field, PartialEq)]
#[repr(i16)]
enum C {
    C = 1,
    D = -292,
    E,
    F = 4
}

#[derive(Clone, Copy, Debug, Eq, bitfield::Field, PartialEq)]
#[repr(i16)]
enum D {
    C = 1,
    D = 292,
    E,
    F = 4
}

#[derive(Clone, Copy, Debug, Eq, bitfield::Field, PartialEq)]
#[repr(i16)]
enum E {
    C = 1,
    D = 100,
    E,
    F = 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        assert_eq!(B::iter().len(), 4);
    }

    #[test]
    fn size() {
        assert_eq!(B::size(), 16);
        assert_eq!(C::size(), 16);
        assert_eq!(D::size(), 9);
        assert_eq!(E::size(), 7);
    }

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

        assert_eq!(TryInto::<C>::try_into(0), Err(0));
        assert_eq!(TryInto::<C>::try_into(1), Ok(C::C));
        assert_eq!(TryInto::<C>::try_into(2), Err(2));
        assert_eq!(TryInto::<C>::try_into(3), Err(3));
        assert_eq!(TryInto::<C>::try_into(4), Ok(C::F));
        assert_eq!(TryInto::<C>::try_into(5), Err(5));
        assert_eq!(TryInto::<C>::try_into(-293), Err(-293));
        assert_eq!(TryInto::<C>::try_into(-292), Ok(C::D));
        assert_eq!(TryInto::<C>::try_into(-291), Ok(C::E));
        assert_eq!(TryInto::<C>::try_into(-290), Err(-290));
    }
}