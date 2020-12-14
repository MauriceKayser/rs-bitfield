extern crate alloc;

#[derive(Copy, Clone, Debug, Eq, bitfield::Flags, PartialEq)]
#[repr(u8)]
enum B {
    C,
    D = 5,
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
    fn max() {
        assert_eq!(B::max(), B::E);
        assert_eq!(B::max() as u8, 6);
    }
}