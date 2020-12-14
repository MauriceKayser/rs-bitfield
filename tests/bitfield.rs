extern crate alloc;

/// When used as a field in a bit field, the field can only contain one of the enum variants.
/// All variants can be represented in 2 bits, but the variant which maps to `0` is non-existent,
/// so the field getter will return `Err(0)` right after initialization of the bit field.
#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Field {
    F1 = 1,
    F2,
    F3
}

/// When used as flags in a bit field, the flags can contain any amount of the enum variants.
/// All variants can be represented in 2 bits, but the variant which maps to the flag `1 << 2` is
/// non-existent, so this bit (flag) can not be accessed via this enum (f. e. deprecated operating
/// system flags).
#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Flags {
    F0,
    F1,
    F3 = 3
}

/// When used as flags in a bit field, the flags can contain any amount of the enum variants.
/// All variants can be represented in 2 bits, but the variant which maps to the flag `1 << 6` is
/// non-existent, so this bit (flag) can not be accessed via this enum (f. e. deprecated operating
/// system flags).
#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Flags2 {
    G4 = 4,
    G5,
    G7 = 7
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_print_eq {
        ($field:expr, $format:expr, $inner_value:expr, $result:expr) => {
            assert_eq!($field.0, $inner_value);
            assert_eq!(format!($format, &$field), $result);
        };
    }

    // Test macros.

    #[test]
    #[should_panic]
    fn test_assert_print_eq_result() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField(Flags);

        // No flags set.
        let field = BitField::new();
        assert_print_eq!(field, "{}", 0, "F1");
    }

    #[test]
    #[should_panic]
    fn test_assert_print_eq_value() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField(Flags);

        // No flags set.
        let field = BitField::new();
        assert_print_eq!(field, "{}", 1, "-");
    }

    // Test bit field features.

    #[test]
    fn debug_named_field() {
        #[bitfield::bitfield(8)]
        #[derive(Debug)]
        struct BitField { #[field(3, 2)] field: Field };

        // Default value which maps to `0`, which is non-existent in `TestField`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { field: Err(0) }");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_field(Field::F1); }
        assert_print_eq!(field, "{:?}", 0, "BitField { field: Err(0) }");

        // Unpacked F1, not `Ok(F1)`.
        field = field.set_field(Field::F1);
        assert_print_eq!(field, "{:?}", 1 << 3, "BitField { field: F1 }");

        // Unpacked F2, not `Ok(F2)`.
        field = field.set_field(Field::F2);
        assert_print_eq!(field, "{:?}", 2 << 3, "BitField { field: F2 }");

        // Unpacked F3, not `Ok(F3)`.
        field = field.set_field(Field::F3);
        assert_print_eq!(field, "{:?}", 3 << 3, "BitField { field: F3 }");
    }

    #[test]
    fn debug_named_flags_multi() {
        #[bitfield::bitfield(8)]
        #[derive(Debug)]
        struct BitField {
            flags: Flags,
            flags2: Flags2
        }

        // No flags set.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { flags: Flags { F0: false, F1: false, F3: false }, flags2: Flags2 { G4: false, G5: false, G7: false } }");

        // Still no flags set, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_flags(Flags::F0, true); }
        assert_print_eq!(field, "{:?}", 0, "BitField { flags: Flags { F0: false, F1: false, F3: false }, flags2: Flags2 { G4: false, G5: false, G7: false } }");

        // Still no flags set, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_flags2(Flags2::G4, true); }
        assert_print_eq!(field, "{:?}", 0, "BitField { flags: Flags { F0: false, F1: false, F3: false }, flags2: Flags2 { G4: false, G5: false, G7: false } }");

        // One flag set.
        field = field.set_flags(Flags::F0, true);
        assert_print_eq!(field, "{:?}", 1 << 0, "BitField { flags: Flags { F0: true, F1: false, F3: false }, flags2: Flags2 { G4: false, G5: false, G7: false } }");

        // Two flags set.
        field = field.set_flags2(Flags2::G4, true);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 4, "BitField { flags: Flags { F0: true, F1: false, F3: false }, flags2: Flags2 { G4: true, G5: false, G7: false } }");

        // Three flags set.
        field = field.set_flags(Flags::F3, true);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 3 | 1 << 4, "BitField { flags: Flags { F0: true, F1: false, F3: true }, flags2: Flags2 { G4: true, G5: false, G7: false } }");

        // Four flags set.
        field = field.set_flags2(Flags2::G7, true);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 3 | 1 << 4 | 1 << 7, "BitField { flags: Flags { F0: true, F1: false, F3: true }, flags2: Flags2 { G4: true, G5: false, G7: true } }");

        // Three flags set.
        field = field.set_flags(Flags::F3, false);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 4 | 1 << 7, "BitField { flags: Flags { F0: true, F1: false, F3: false }, flags2: Flags2 { G4: true, G5: false, G7: true } }");

        // Two flags set.
        field = field.set_flags2(Flags2::G4, false);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 7, "BitField { flags: Flags { F0: true, F1: false, F3: false }, flags2: Flags2 { G4: false, G5: false, G7: true } }");

        // One flag set.
        field = field.set_flags(Flags::F0, false);
        assert_print_eq!(field, "{:?}", 1 << 7, "BitField { flags: Flags { F0: false, F1: false, F3: false }, flags2: Flags2 { G4: false, G5: false, G7: true } }");

        // No flags set.
        field = field.set_flags2(Flags2::G7, false);
        assert_print_eq!(field, "{:?}", 0, "BitField { flags: Flags { F0: false, F1: false, F3: false }, flags2: Flags2 { G4: false, G5: false, G7: false } }");
    }

    #[test]
    fn debug_named_field_single() {
        #[bitfield::bitfield(8)]
        #[derive(Debug)]
        struct BitField {
            flags: Flags
        }

        // No flags set.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { flags: Flags { F0: false, F1: false, F3: false } }");

        // Still no flags set, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_flags(Flags::F0, true); }
        assert_print_eq!(field, "{:?}", 0, "BitField { flags: Flags { F0: false, F1: false, F3: false } }");

        // One flag set.
        field = field.set_flags(Flags::F0, true);
        assert_print_eq!(field, "{:?}", 1 << 0, "BitField { flags: Flags { F0: true, F1: false, F3: false } }");

        // Two flags set.
        field = field.set_flags(Flags::F1, true);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 1, "BitField { flags: Flags { F0: true, F1: true, F3: false } }");

        // Three flags set.
        field = field.set_flags(Flags::F3, true);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 1 | 1 << 3, "BitField { flags: Flags { F0: true, F1: true, F3: true } }");

        // Two flags set.
        field = field.set_flags(Flags::F0, false);
        assert_print_eq!(field, "{:?}", 1 << 1 | 1 << 3, "BitField { flags: Flags { F0: false, F1: true, F3: true } }");

        // One flag set.
        field = field.set_flags(Flags::F1, false);
        assert_print_eq!(field, "{:?}", 1 << 3, "BitField { flags: Flags { F0: false, F1: false, F3: true } }");

        // No flags set.
        field = field.set_flags(Flags::F3, false);
        assert_print_eq!(field, "{:?}", 0, "BitField { flags: Flags { F0: false, F1: false, F3: false } }");
    }

    #[test]
    fn debug_named_mixed() {
        #[bitfield::bitfield(16)]
        #[derive(Debug)]
        struct BitField {
            #[field(0, 2)]
            field: Field,
            flags: Flags2,
            #[field(8, 2)]
            int: u8
        }

        // No flags set, and default field value.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { field: Err(0), flags: Flags2 { G4: false, G5: false, G7: false }, int: 0 }");

        // Still no flags set and default field values, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_flags(Flags2::G4, true); }
        assert_print_eq!(field, "{:?}", 0, "BitField { field: Err(0), flags: Flags2 { G4: false, G5: false, G7: false }, int: 0 }");

        // Still no flags set and default field values, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_field(Field::F1); }
        assert_print_eq!(field, "{:?}", 0, "BitField { field: Err(0), flags: Flags2 { G4: false, G5: false, G7: false }, int: 0 }");

        // Still no flags set and default field values, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_int(1).unwrap(); }
        assert_print_eq!(field, "{:?}", 0, "BitField { field: Err(0), flags: Flags2 { G4: false, G5: false, G7: false }, int: 0 }");

        // One flag set.
        field = field.set_flags(Flags2::G4, true);
        assert_print_eq!(field, "{:?}", 1 << 4, "BitField { field: Err(0), flags: Flags2 { G4: true, G5: false, G7: false }, int: 0 }");

        // Unpacked F1, not `Ok(F1)`.
        field = field.set_field(Field::F1);
        assert_print_eq!(field, "{:?}", 1 | 1 << 4, "BitField { field: F1, flags: Flags2 { G4: true, G5: false, G7: false }, int: 0 }");

        // Integer value.
        field = field.set_int(1).unwrap();
        assert_print_eq!(field, "{:?}", 1 | 1 << 4 | 1 << 8, "BitField { field: F1, flags: Flags2 { G4: true, G5: false, G7: false }, int: 1 }");

        // Two flags set.
        field = field.set_flags(Flags2::G5, true);
        assert_print_eq!(field, "{:?}", 1 | 1 << 4 | 1 << 5 | 1 << 8, "BitField { field: F1, flags: Flags2 { G4: true, G5: true, G7: false }, int: 1 }");

        // Unpacked F2, not `Ok(F2)`.
        field = field.set_field(Field::F2);
        assert_print_eq!(field, "{:?}", 2 | 1 << 4 | 1 << 5 | 1 << 8, "BitField { field: F2, flags: Flags2 { G4: true, G5: true, G7: false }, int: 1 }");

        // Integer value.
        field = field.set_int(2).unwrap();
        assert_print_eq!(field, "{:?}", 2 | 1 << 4 | 1 << 5 | 2 << 8, "BitField { field: F2, flags: Flags2 { G4: true, G5: true, G7: false }, int: 2 }");

        // Three flags set.
        field = field.set_flags(Flags2::G7, true);
        assert_print_eq!(field, "{:?}", 2 | 1 << 4 | 1 << 5 | 1 << 7 | 2 << 8, "BitField { field: F2, flags: Flags2 { G4: true, G5: true, G7: true }, int: 2 }");

        // Unpacked F2, not `Ok(F2)`.
        field = field.set_field(Field::F3);
        assert_print_eq!(field, "{:?}", 3 | 1 << 4 | 1 << 5 | 1 << 7 | 2 << 8, "BitField { field: F3, flags: Flags2 { G4: true, G5: true, G7: true }, int: 2 }");

        // Integer value.
        field = field.set_int(3).unwrap();
        assert_print_eq!(field, "{:?}", 3 | 1 << 4 | 1 << 5 | 1 << 7 | 3 << 8, "BitField { field: F3, flags: Flags2 { G4: true, G5: true, G7: true }, int: 3 }");

        // Two flags set.
        field = field.set_flags(Flags2::G4, false);
        assert_print_eq!(field, "{:?}", 3 | 1 << 5 | 1 << 7 | 3 << 8, "BitField { field: F3, flags: Flags2 { G4: false, G5: true, G7: true }, int: 3 }");

        // Invalid integer value.
        assert!(field.set_int(4).is_none());

        // One flag set.
        field = field.set_flags(Flags2::G5, false);
        assert_print_eq!(field, "{:?}", 3 | 1 << 7 | 3 << 8, "BitField { field: F3, flags: Flags2 { G4: false, G5: false, G7: true }, int: 3 }");

        // Integer value.
        field = field.set_int(0).unwrap();
        assert_print_eq!(field, "{:?}", 3 | 1 << 7, "BitField { field: F3, flags: Flags2 { G4: false, G5: false, G7: true }, int: 0 }");

        // No flags set.
        field = field.set_flags(Flags2::G7, false);
        assert_print_eq!(field, "{:?}", 3, "BitField { field: F3, flags: Flags2 { G4: false, G5: false, G7: false }, int: 0 }");
    }

    #[test]
    fn debug_tuple_field() {
        #[bitfield::bitfield(8)]
        #[derive(Debug)]
        struct BitField(#[field(3, 2)] Field);

        // Default value which maps to `0`, which is non-existent in `TestField`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { Field: Err(0) }");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(Field::F1); }
        assert_print_eq!(field, "{:?}", 0, "BitField { Field: Err(0) }");

        // Unpacked F1, not `Ok(F1)`.
        field = field.set(Field::F1);
        assert_print_eq!(field, "{:?}", 1 << 3, "BitField { Field: F1 }");

        // Unpacked F2, not `Ok(F2)`.
        field = field.set(Field::F2);
        assert_print_eq!(field, "{:?}", 2 << 3, "BitField { Field: F2 }");

        // Unpacked F3, not `Ok(F3)`.
        field = field.set(Field::F3);
        assert_print_eq!(field, "{:?}", 3 << 3, "BitField { Field: F3 }");
    }

    #[test]
    fn debug_tuple_field_primitive_bool_explicit() {
        #[bitfield::bitfield(8)]
        #[derive(Debug)]
        struct BitField(#[field(bit = 2)] bool);

        // Default value which maps to `0`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { bool: false }");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(true); }
        assert_print_eq!(field, "{:?}", 0, "BitField { bool: false }");

        // `true`.
        field = field.set(true);
        assert_print_eq!(field, "{:?}", 1 << 2, "BitField { bool: true }");

        // `false`.
        field = field.set(false);
        assert_print_eq!(field, "{:?}", 0, "BitField { bool: false }");
    }

    #[test]
    fn debug_tuple_field_primitive_bool_implicit() {
        #[bitfield::bitfield(8)]
        #[derive(Debug)]
        struct BitField(bool);

        // Default value which maps to `0`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { bool: false }");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(true); }
        assert_print_eq!(field, "{:?}", 0, "BitField { bool: false }");

        // `true`.
        field = field.set(true);
        assert_print_eq!(field, "{:?}", 1 << 0, "BitField { bool: true }");

        // `false`.
        field = field.set(false);
        assert_print_eq!(field, "{:?}", 0, "BitField { bool: false }");
    }

    #[test]
    fn debug_tuple_field_primitive_u8_explicit() {
        #[bitfield::bitfield(16)]
        #[derive(Debug)]
        struct BitField(#[field(4, 2)] u8);

        // Default value which maps to `0`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { u8: 0 }");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(3); }
        assert_print_eq!(field, "{:?}", 0, "BitField { u8: 0 }");

        // Unpacked 3, not `Ok(3)`.
        field = field.set(3).unwrap();
        assert_print_eq!(field, "{:?}", 3 << 4, "BitField { u8: 3 }");

        // Out of bounds value.
        assert!(field.set(4).is_none());

        // Unpacked 0, not `Ok(0)`.
        field = field.set(0).unwrap();
        assert_print_eq!(field, "{:?}", 0, "BitField { u8: 0 }");
    }

    #[test]
    fn debug_tuple_field_primitive_u8_implicit() {
        #[bitfield::bitfield(16)]
        #[derive(Debug)]
        struct BitField(u8);

        // Default value which maps to `0`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { u8: 0 }");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(3); }
        assert_print_eq!(field, "{:?}", 0, "BitField { u8: 0 }");

        // Unpacked 3, not `Ok(3)`.
        field = field.set(3).unwrap();
        assert_print_eq!(field, "{:?}", 3 << 0, "BitField { u8: 3 }");

        // Unpacked 0, not `Ok(0)`.
        field = field.set(0).unwrap();
        assert_print_eq!(field, "{:?}", 0, "BitField { u8: 0 }");
    }

    #[test]
    fn debug_tuple_flags() {
        #[bitfield::bitfield(8)]
        #[derive(Debug)]
        struct BitField(Flags);

        // No flags set.
        let mut field = BitField::new();
        assert_print_eq!(field, "{:?}", 0, "BitField { F0: false, F1: false, F3: false }");

        // Still no flags set, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(Flags::F0, true); }
        assert_print_eq!(field, "{:?}", 0, "BitField { F0: false, F1: false, F3: false }");

        // One flag set.
        field = field.set(Flags::F0, true);
        assert_print_eq!(field, "{:?}", 1 << 0, "BitField { F0: true, F1: false, F3: false }");

        // Two flags set.
        field = field.set(Flags::F1, true);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 1, "BitField { F0: true, F1: true, F3: false }");

        // Three flags set.
        field = field.set(Flags::F3, true);
        assert_print_eq!(field, "{:?}", 1 << 0 | 1 << 1 | 1 << 3, "BitField { F0: true, F1: true, F3: true }");

        // Two flags set.
        field = field.set(Flags::F0, false);
        assert_print_eq!(field, "{:?}", 1 << 1 | 1 << 3, "BitField { F0: false, F1: true, F3: true }");

        // One flag set.
        field = field.set(Flags::F1, false);
        assert_print_eq!(field, "{:?}", 1 << 3, "BitField { F0: false, F1: false, F3: true }");

        // No flags set.
        field = field.set(Flags::F3, false);
        assert_print_eq!(field, "{:?}", 0, "BitField { F0: false, F1: false, F3: false }");
    }

    #[test]
    fn display_named_field() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField { #[field(3, 2)] field: Field };

        // Default value which maps to `0`, which is non-existent in `TestField`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "Err(0)");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_field(Field::F1); }
        assert_print_eq!(field, "{}", 0, "Err(0)");

        // Unpacked F1, not `Ok(F1)`.
        field = field.set_field(Field::F1);
        assert_print_eq!(field, "{}", 1 << 3, "F1");

        // Unpacked F2, not `Ok(F2)`.
        field = field.set_field(Field::F2);
        assert_print_eq!(field, "{}", 2 << 3, "F2");

        // Unpacked F3, not `Ok(F3)`.
        field = field.set_field(Field::F3);
        assert_print_eq!(field, "{}", 3 << 3, "F3");
    }

    #[test]
    fn display_named_field_integer() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField { #[field(3, 2)] field: u8 };

        // Default value which maps to `0`, which is non-existent in `TestField`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "0");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_field(1).unwrap(); }
        assert_print_eq!(field, "{}", 0, "0");

        // Integer in bounds.
        field = field.set_field(1).unwrap();
        assert_print_eq!(field, "{}", 1 << 3, "1");

        // Integer in bounds.
        field = field.set_field(2).unwrap();
        assert_print_eq!(field, "{}", 2 << 3, "2");

        // Integer in bounds.
        field = field.set_field(3).unwrap();
        assert_print_eq!(field, "{}", 3 << 3, "3");

        // Integer out of bounds.
        assert!(field.set_field(4).is_none());

        // Integer in bounds.
        field = field.set_field(0).unwrap();
        assert_print_eq!(field, "{}", 0, "0");
    }

    #[test]
    fn display_named_flags_multi() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField {
            flags: Flags,
            flags2: Flags2
        }

        // No flags set.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "-");

        // Still no flags set, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_flags(Flags::F0, true); }
        assert_print_eq!(field, "{}", 0, "-");

        // Still no flags set, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_flags2(Flags2::G4, true); }
        assert_print_eq!(field, "{}", 0, "-");

        // One flag set.
        field = field.set_flags(Flags::F0, true);
        assert_print_eq!(field, "{}", 1 << 0, "bitfield::Flags::F0");

        // Two flags set.
        field = field.set_flags2(Flags2::G4, true);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 4, "bitfield::Flags::F0 | bitfield::Flags2::G4");

        // Three flags set.
        field = field.set_flags(Flags::F3, true);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 3 | 1 << 4, "bitfield::Flags::F0 | bitfield::Flags::F3 | bitfield::Flags2::G4");

        // Four flags set.
        field = field.set_flags2(Flags2::G7, true);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 3 | 1 << 4 | 1 << 7, "bitfield::Flags::F0 | bitfield::Flags::F3 | bitfield::Flags2::G4 | bitfield::Flags2::G7");

        // Three flags set.
        field = field.set_flags(Flags::F3, false);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 4 | 1 << 7, "bitfield::Flags::F0 | bitfield::Flags2::G4 | bitfield::Flags2::G7");

        // Two flags set.
        field = field.set_flags2(Flags2::G4, false);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 7, "bitfield::Flags::F0 | bitfield::Flags2::G7");

        // One flag set.
        field = field.set_flags(Flags::F0, false);
        assert_print_eq!(field, "{}", 1 << 7, "bitfield::Flags2::G7");

        // No flags set.
        field = field.set_flags2(Flags2::G7, false);
        assert_print_eq!(field, "{}", 0, "-");
    }

    #[test]
    fn display_named_field_single() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField {
            flags: Flags
        }

        // No flags set.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "-");

        // Still no flags set, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set_flags(Flags::F0, true); }
        assert_print_eq!(field, "{}", 0, "-");

        // One flag set.
        field = field.set_flags(Flags::F0, true);
        assert_print_eq!(field, "{}", 1 << 0, "F0");

        // Two flags set.
        field = field.set_flags(Flags::F1, true);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 1, "F0 | F1");

        // Three flags set.
        field = field.set_flags(Flags::F3, true);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 1 | 1 << 3, "F0 | F1 | F3");

        // Two flags set.
        field = field.set_flags(Flags::F0, false);
        assert_print_eq!(field, "{}", 1 << 1 | 1 << 3, "F1 | F3");

        // One flag set.
        field = field.set_flags(Flags::F1, false);
        assert_print_eq!(field, "{}", 1 << 3, "F3");

        // No flags set.
        field = field.set_flags(Flags::F3, false);
        assert_print_eq!(field, "{}", 0, "-");
    }

    #[test]
    fn display_tuple_field() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField(#[field(3, 2)] Field);

        // Default value which maps to `0`, which is non-existent in `TestField`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "Err(0)");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(Field::F1); }
        assert_print_eq!(field, "{}", 0, "Err(0)");

        // Unpacked F1, not `Ok(F1)`.
        field = field.set(Field::F1);
        assert_print_eq!(field, "{}", 1 << 3, "F1");

        // Unpacked F2, not `Ok(F2)`.
        field = field.set(Field::F2);
        assert_print_eq!(field, "{}", 2 << 3, "F2");

        // Unpacked F3, not `Ok(F3)`.
        field = field.set(Field::F3);
        assert_print_eq!(field, "{}", 3 << 3, "F3");
    }

    #[test]
    fn display_tuple_field_primitive_bool_explicit() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField(#[field(bit = 2)] bool);

        // Default value which maps to `0`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "false");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(true); }
        assert_print_eq!(field, "{}", 0, "false");

        // `true`.
        field = field.set(true);
        assert_print_eq!(field, "{}", 1 << 2, "true");

        // `false`.
        field = field.set(false);
        assert_print_eq!(field, "{}", 0, "false");
    }

    #[test]
    fn display_tuple_field_primitive_bool_implicit() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField(bool);

        // Default value which maps to `0`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "false");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(true); }
        assert_print_eq!(field, "{}", 0, "false");

        // `true`.
        field = field.set(true);
        assert_print_eq!(field, "{}", 1 << 0, "true");

        // `false`.
        field = field.set(false);
        assert_print_eq!(field, "{}", 0, "false");
    }

    #[test]
    fn display_tuple_field_primitive_u8_explicit() {
        #[bitfield::bitfield(16)]
        #[derive(Display)]
        struct BitField(#[field(4, 2)] u8);

        // Default value which maps to `0`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "0");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(3); }
        assert_print_eq!(field, "{}", 0, "0");

        // Unpacked 3, not `Ok(3)`.
        field = field.set(3).unwrap();
        assert_print_eq!(field, "{}", 3 << 4, "3");

        // Out of bounds value.
        assert!(field.set(4).is_none());

        // Unpacked 0, not `Ok(0)`.
        field = field.set(0).unwrap();
        assert_print_eq!(field, "{}", 0, "0");
    }

    #[test]
    fn display_tuple_field_primitive_u8_implicit() {
        #[bitfield::bitfield(16)]
        #[derive(Display)]
        struct BitField(u8);

        // Default value which maps to `0`.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "0");

        // Still default value, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(3); }
        assert_print_eq!(field, "{}", 0, "0");

        // Unpacked 3, not `Ok(3)`.
        field = field.set(3).unwrap();
        assert_print_eq!(field, "{}", 3 << 0, "3");

        // Unpacked 0, not `Ok(0)`.
        field = field.set(0).unwrap();
        assert_print_eq!(field, "{}", 0, "0");
    }

    #[test]
    fn display_tuple_flags() {
        #[bitfield::bitfield(8)]
        #[derive(Display)]
        struct BitField(Flags);

        // No flags set.
        let mut field = BitField::new();
        assert_print_eq!(field, "{}", 0, "-");

        // Still no flags set, as the modified result is not stored.
        #[allow(unused_must_use)]
        { field.set(Flags::F0, true); }
        assert_print_eq!(field, "{}", 0, "-");

        // One flag set.
        field = field.set(Flags::F0, true);
        assert_print_eq!(field, "{}", 1 << 0, "F0");

        // Two flags set.
        field = field.set(Flags::F1, true);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 1, "F0 | F1");

        // Three flags set.
        field = field.set(Flags::F3, true);
        assert_print_eq!(field, "{}", 1 << 0 | 1 << 1 | 1 << 3, "F0 | F1 | F3");

        // Two flags set.
        field = field.set(Flags::F0, false);
        assert_print_eq!(field, "{}", 1 << 1 | 1 << 3, "F1 | F3");

        // One flag set.
        field = field.set(Flags::F1, false);
        assert_print_eq!(field, "{}", 1 << 3, "F3");

        // No flags set.
        field = field.set(Flags::F3, false);
        assert_print_eq!(field, "{}", 0, "-");
    }

    #[test]
    fn flags() {
        #[bitfield::bitfield(8)]
        struct BitField {
            flags: Flags,
            flags2: Flags2
        }

        assert_eq!(BitField::flags_mask(), 0b0000_1011);
        assert_eq!(BitField::flags2_mask(), 0b1011_0000);

        let mut field = BitField::new();
        assert!(!field.flags_any());
        assert!(!field.flags_all());
        assert!(!field.flags2_any());
        assert!(!field.flags2_all());

        field = field.set_flags_all();
        assert!(field.flags_any());
        assert!(field.flags_all());
        assert!(!field.flags2_any());
        assert!(!field.flags2_all());

        field = field.set_flags_none();
        assert!(!field.flags_any());
        assert!(!field.flags_all());
        assert!(!field.flags2_any());
        assert!(!field.flags2_all());

        field = field.set_flags2_all();
        assert!(!field.flags_any());
        assert!(!field.flags_all());
        assert!(field.flags2_any());
        assert!(field.flags2_all());

        field = field.set_flags2_none();
        assert!(!field.flags_any());
        assert!(!field.flags_all());
        assert!(!field.flags2_any());
        assert!(!field.flags2_all());

        field = field.set_flags(Flags::F1, true);
        assert!(field.flags_any());
        assert!(!field.flags_all());
        assert!(!field.flags2_any());
        assert!(!field.flags2_all());

        field = field.set_flags2(Flags2::G7, true);
        assert!(field.flags_any());
        assert!(!field.flags_all());
        assert!(field.flags2_any());
        assert!(!field.flags2_all());

        field = field.set_flags(Flags::F0, true).set_flags(Flags::F3, true);
        assert!(field.flags_any());
        assert!(field.flags_all());
        assert!(field.flags2_any());
        assert!(!field.flags2_all());

        field = field.set_flags2(Flags2::G4, true).set_flags2(Flags2::G5, true);
        assert!(field.flags_any());
        assert!(field.flags_all());
        assert!(field.flags2_any());
        assert!(field.flags2_all());

        field = field.set_flags2_none();
        assert!(field.flags_any());
        assert!(field.flags_all());
        assert!(!field.flags2_any());
        assert!(!field.flags2_all());

        field = field.set_flags_none();
        assert!(!field.flags_any());
        assert!(!field.flags_all());
        assert!(!field.flags2_any());
        assert!(!field.flags2_all());
    }

    #[test]
    fn ui() {
        trybuild::TestCases::new().compile_fail("tests/ui/bitfield/*.rs");

        #[cfg(target_pointer_width = "8")]
            trybuild::TestCases::new().compile_fail(
            "tests/ui/bitfield/architecture/field_uses_whole_bit_field_8.rs"
        );
        #[cfg(target_pointer_width = "16")]
            trybuild::TestCases::new().compile_fail(
            "tests/ui/bitfield/architecture/field_uses_whole_bit_field_16.rs"
        );
        #[cfg(target_pointer_width = "32")]
            trybuild::TestCases::new().compile_fail(
            "tests/ui/bitfield/architecture/field_uses_whole_bit_field_32.rs"
        );
        #[cfg(target_pointer_width = "64")]
            trybuild::TestCases::new().compile_fail(
            "tests/ui/bitfield/architecture/field_uses_whole_bit_field_64.rs"
        );
        #[cfg(target_pointer_width = "128")]
            trybuild::TestCases::new().compile_fail(
            "tests/ui/bitfield/architecture/field_uses_whole_bit_field_128.rs"
        );
    }
}