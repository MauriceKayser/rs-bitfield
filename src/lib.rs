//! # Bit fields for Rust
//!
//! Provides structures which simplify bit level access to primitive types in Rust.
//!
//! ## Dependencies
//!
//! None, the types work in a `#[no_std]` environment.
//!
//! ## Description
//!
//! A bit field can store simple boolean flags, as well as values of multiple bits in size.
//! A `BitField` structure, for example `BitField16`, has the following 6 `const` functions:
//!
//! - `const fn new() -> Self`
//! - `const fn value(&self) -> u16`
//! - `const fn bit(&self, position: u8) -> bool`
//! - `const fn set_bit(&self, position: u8, value: bool) -> Self`
//! - `const fn field(&self, position: u8, size: u8) -> u16`
//! - `const fn set_field(&self, position: u8, size: u8, value: u16) -> Self`
//!
//! The setters return a modified copy of their own value, so the builder pattern can be used
//! to construct such a bit field.
//!
//! ## Simple example
//!
//! Imagine the following type which can store up to 16 flags in a `u16` value:
//!
//! ```ignore
//! pub const IS_SYSTEM:    u16 = 1 << 0; // 1
//! pub const IS_LIBRARY:   u16 = 1 << 1; // 2
//! // Undefined:                 1 << 2; // 4
//! pub const IS_X64:       u16 = 1 << 3; // 8
//! // ... up to 12 more flags ...
//!
//! extern "C" fn bla() -> u16;
//! extern "C" fn foo(executable_flags: u16);
//!
//! // Usage
//!
//! let mut executable_flags = bla();
//!
//! // Add the system and x64 flags.
//! executable_flags |= IS_SYSTEM | IS_X64;
//!
//! // Execute `foo` if the library flag is set.
//! if (executable_flags & IS_LIBRARY) != 0 {
//!     foo(executable_flags);
//! }
//! ```
//!
//! With the help of this crate this can be expressed as follows:
//!
//! ```ignore
//! #[repr(C)]
//! pub struct ExecutableFlags(bitfield::BitField16);
//!
//! #[repr(u8)]
//! pub enum ExecutableFlag {
//!     System,
//!     Library,
//!     X64 = 3
//! }
//!
//! impl ExecutableFlags {
//!     pub const fn new() -> Self {
//!         Self(bitfield::BitField16::new())
//!     }
//!
//!     pub const fn is_set(&self, flag: ExecutableFlag) -> bool {
//!         self.0.bit(flag as u8)
//!     }
//!
//!     pub const fn set(&self, flag: ExecutableFlag, value: bool) -> Self {
//!         Self(self.0.set_bit(flag as u8, value))
//!     }
//! }
//!
//! extern "C" fn bla() -> ExecutableFlags;
//! extern "C" fn foo(executable_flags: ExecutableFlags);
//!
//! // Usage
//!
//! let executable_flags = bla().set(ExecutableFlag::System).set(ExecutableFlag::X64);
//!
//! if executable_flags.is_set(ExecutableFlag::Library) {
//!     foo(executable_flags);
//! }
//! ```
//!
//! ## Detailed Example
//!
//! This example is based on the 4. parameter `UINT uType` of Microsoft Windows
//! [user32.MessageBox function](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messagebox)
//! which not only stores boolean flags, but also fields with more than one bit in size.
//!
//! A Microsoft Visual C++ `UINT` is a `u32` in Rust. So all constants for the parameter `uType` can be written as follows:
//!
//! ```rust
//! // Buttons
//! const MB_BUTTON_OK:                  u32 = 0;
//! const MB_BUTTON_OK_CANCEL:           u32 = 1;
//! const MB_BUTTON_ABORT_RETRY_IGNORE:  u32 = 2;
//! const MB_BUTTON_YES_NO_CANCEL:       u32 = 3;
//! const MB_BUTTON_YES_NO:              u32 = 4;
//! const MB_BUTTON_RETRY_CANCEL:        u32 = 5;
//! const MB_BUTTON_CANCEL_TRY_CONTINUE: u32 = 6;
//!
//! // Icons
//! const MB_ICON_NONE:                  u32 = 0x00;
//! const MB_ICON_ERROR:                 u32 = 0x10;
//! const MB_ICON_QUESTION:              u32 = 0x20;
//! const MB_ICON_EXCLAMATION:           u32 = 0x30;
//! const MB_ICON_INFORMATION:           u32 = 0x40;
//!
//! // Default buttons
//! const MB_DEFAULT_BUTTON1:            u32 = 0x000;
//! const MB_DEFAULT_BUTTON2:            u32 = 0x100;
//! const MB_DEFAULT_BUTTON3:            u32 = 0x200;
//! const MB_DEFAULT_BUTTON4:            u32 = 0x300;
//!
//! // Modality
//! const MB_MODALITY_APPLICATION:       u32 = 0x0000;
//! const MB_MODALITY_SYSTEM:            u32 = 0x1000;
//! const MB_MODALITY_TASK:              u32 = 0x2000;
//!
//! // Other flags
//! const MB_HELP:                       u32 = 1 << 14;
//! const MB_FOREGROUND:                 u32 = 1 << 16;
//! const MB_DEFAULT_DESKTOP_ONLY:       u32 = 1 << 17;
//! const MB_TOP_MOST:                   u32 = 1 << 18;
//! const MB_RIGHT:                      u32 = 1 << 19;
//! const MB_RIGHT_TO_LEFT_READING:      u32 = 1 << 20;
//! const MB_SERVICE_NOTIFICATION:       u32 = 1 << 21;
//! ```
//!
//! One problem is that `u32` is not type safe like an `enum` value, another is that the usage of an `u32` is error prone,
//! because several "flags" of the same "type group" (called a "field"), like a button, can be `|`-ed together
//! (f. e. `MB_BUTTON_ABORT_RETRY_IGNORE | MB_BUTTON_YES_NO`) which might result in some unexpected behaviour.
//! Checking if certain fields, like a button, have a specific value is also unnecessarily complicated (bit fiddling
//! operators like `>>` and `&` etc. are necessary).
//!
//! The previously mentioned fields are stored in the `u32` value as follows:
//!
//! | Type            | Min. def. value (`> 0`) | Max. def. value | Storage bits                                            | Max. storable value             |
//! | --------------- | ----------------------- | --------------- | ------------------------------------------------------- | ------------------------------- |
//! | `Button`        | 0x1                     | 0x6             | 0b0000_0000_0000_0000_0000_0000_0000_**XXXX** (0 - 4)   | `((1 << 4) - 1) <<  0` = 0x7    |
//! | `Icon`          | 0x10                    | 0x40            | 0b0000_0000_0000_0000_0000_0000_**XXXX**_0000 (4 - 8)   | `((1 << 4) - 1) <<  4` = 0x70   |
//! | `DefaultButton` | 0x100                   | 0x300           | 0b0000_0000_0000_0000_0000_**XXXX**_0000_0000 (8 - 12)  | `((1 << 4) - 1) <<  8` = 0x700  |
//! | `Modality`      | 0x1000                  | 0x2000          | 0b0000_0000_0000_0000_00**XX**_0000_0000_0000 (12 - 13) | `((1 << 2) - 1) << 12` = 0x3000 |
//!
//! All of the fields can be expressed by shifting them to the right (removing the trailing zeros):
//!
//! ```rust
//! #[repr(u8)]
//! pub enum Button {
//!     Ok,
//!     OkCancel,
//!     AbortRetryIgnore,
//!     YesNoCancel,
//!     YesNo,
//!     RetryCancel,
//!     CancelTryContinue
//!     // Value `7` is unused.
//! }
//!
//! #[repr(u8)]
//! pub enum DefaultButton {
//!     One,
//!     Two,
//!     Three,
//!     Four
//!     // Values `4` - `7` are unused.
//! }
//!
//! #[repr(u8)]
//! pub enum Icon {
//!     None,
//!     Error,
//!     Question,
//!     Warning,
//!     Information
//!     // Values `5` - `7` are unused.
//! }
//!
//! #[repr(u8)]
//! pub enum Modality {
//!     Application,
//!     System,
//!     Task
//!     // Value `3` is unused.
//! }
//! ```
//!
//! The write- or construct-only variant of the `Styles` structure can be built with the
//! `BitField32` type like so:
//!
//! ```ignore
//! #[repr(C)]
//! pub struct Styles(bitfield::BitField32);
//!
//! #[repr(u8)]
//! pub enum Style {
//!     Help = 14,
//!     SetForeground = 16,
//!     DefaultDesktopOnly,
//!     TopMost,
//!     Right,
//!     RightToLeftReading,
//!     ServiceNotification
//! }
//!
//! impl Styles {
//!     pub const fn new() -> Self {
//!         Self(bitfield::BitField32::new())
//!     }
//!
//!     pub const fn set(&self, style: Style, value: bool) -> Self {
//!         Self(self.0.set_bit(style as u8, value))
//!     }
//!
//!     // Field setters
//!
//!     pub const fn set_button(&self, button: Button) -> Self {
//!         Self(self.0.set_field(0, 4, button as u32))
//!     }
//!
//!     pub const fn set_icon(&self, icon: Icon) -> Self {
//!         Self(self.0.set_field(4, 4, icon as u32))
//!     }
//!
//!     pub const fn set_default_button(&self, default_button: DefaultButton) -> Self {
//!         Self(self.0.set_field(8, 4, default_button as u32))
//!     }
//!
//!     pub const fn set_modality(&self, modality: Modality) -> Self {
//!         Self(self.0.set_field(12, 2, modality as u32))
//!     }
//! }
//! ```
//!
//! It can now be constructed and used as follows:
//!
//! ```ignore
//! let styles = Styles::new()
//!     .set_button(Button::OkCancel)
//!     .set_icon(Icon::Information)
//!     .set(Style::Right, true)
//!     .set(Style::TopMost, true);
//!
//! let result = user32::MessageBoxW(/* ... */, styles);
//! ```
//!
//! For the read-write variant of the `Styles` structure the following code has to be added:
//!
//! ```ignore
//! use core::convert::TryFrom;
//!
//! impl Styles {
//!     pub const fn is_set(&self, style: Style) -> bool {
//!         self.0.bit(style as u8)
//!     }
//!
//!     // Field getters
//!     //
//!     // They must return a `core::result::Result`, because the bits can represent values which
//!     // are not among the defined enumeration variants.
//!     //
//!     // They can not be `const` until [RFC-2632](https://github.com/rust-lang/rfcs/pull/2632) is done.
//!
//!     pub fn button(&self) -> Result<Button, u8> {
//!         Button::try_from(self.0.field(0, 4) as u8)
//!     }
//!
//!     pub fn icon(&self) -> Result<Icon, u8> {
//!         Icon::try_from(self.0.field(4, 4) as u8)
//!     }
//!
//!     pub fn default_button(&self) -> Result<DefaultButton, u8> {
//!         DefaultButton::try_from(self.0.field(8, 4) as u8)
//!     }
//!
//!     pub fn modality(&self) -> Result<Modality, u8> {
//!         Modality::try_from(self.0.field(12, 2) as u8)
//!     }
//! }
//!
//! // Convert from `u8` to our enumerations, necessary for the `Field getters` part.
//! // An alternative to the manual implementation is using a crate like:
//! // [from-primitive](https://github.com/MauriceKayser/rs-from-primitive)
//!
//! impl core::convert::TryFrom<u8> for Button {
//!     type Error = u8;
//!
//!     fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
//!         match value {
//!             v if v == Self::Ok                  as u8 => Ok(Self::Ok),
//!             v if v == Self::OkCancel            as u8 => Ok(Self::OkCancel),
//!             v if v == Self::AbortRetryIgnore    as u8 => Ok(Self::AbortRetryIgnore),
//!             v if v == Self::YesNoCancel         as u8 => Ok(Self::YesNoCancel),
//!             v if v == Self::YesNo               as u8 => Ok(Self::YesNo),
//!             v if v == Self::RetryCancel         as u8 => Ok(Self::RetryCancel),
//!             v if v == Self::CancelTryContinue   as u8 => Ok(Self::CancelTryContinue),
//!             _ => Err(value)
//!         }
//!     }
//! }
//!
//! impl core::convert::TryFrom<u8> for DefaultButton {
//!     type Error = u8;
//!
//!     fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
//!         match value {
//!             v if v == Self::One     as u8 => Ok(Self::One),
//!             v if v == Self::Two     as u8 => Ok(Self::Two),
//!             v if v == Self::Three   as u8 => Ok(Self::Three),
//!             v if v == Self::Four    as u8 => Ok(Self::Four),
//!             _ => Err(value)
//!         }
//!     }
//! }
//!
//! impl core::convert::TryFrom<u8> for Icon {
//!     type Error = u8;
//!
//!     fn try_from(value: u8) -> core::result::Result<Self, <Icon as core::convert::TryFrom<u8>>::Error> {
//!         match value {
//!             v if v == Self::None        as u8 => Ok(Self::None),
//!             v if v == Self::Error       as u8 => Ok(Self::Error),
//!             v if v == Self::Question    as u8 => Ok(Self::Question),
//!             v if v == Self::Warning     as u8 => Ok(Self::Warning),
//!             v if v == Self::Information as u8 => Ok(Self::Information),
//!             _ => Err(value)
//!         }
//!     }
//! }
//!
//! impl core::convert::TryFrom<u8> for Modality {
//!     type Error = u8;
//!
//!     fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
//!         match value {
//!             v if v == Self::Application as u8 => Ok(Self::Application),
//!             v if v == Self::System      as u8 => Ok(Self::System),
//!             v if v == Self::Task        as u8 => Ok(Self::Task),
//!             _ => Err(value)
//!         }
//!     }
//! }
//! ```
//!
//! It can now be constructed and used as follows:
//!
//! ```ignore
//! let styles = Styles::new()
//!     .set_button(Button::OkCancel)
//!     .set_icon(Icon::Information)
//!     .set(Style::Right, true)
//!     .set(Style::TopMost, true);
//!
//! // `Button == Button` needs `#[derive(PartialEq)]` for `Button`.
//! if styles.is_set(Style::Help) && styles.button().unwrap() == Button::OkCancel {
//!     let result = user32::MessageBoxW(/* ... */, styles.set_button(Button::YesNo));
//! }
//! ```
//!
//! ## TODO
//!
//! - Bounds checking has to wait until [Allow panicking in constants](https://github.com/rust-lang/rust/issues/51999)
//! is merged.
//! - Update documentation if [RFC-2632](https://github.com/rust-lang/rfcs/pull/2632) is done.

#![no_std]

#[cfg(test)]
extern crate std;

macro_rules! BitField {
    ($name:ident : $int:ident) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub struct $name($int);

        impl $name {
            /// Creates a new instance.
            #[inline(always)]
            pub const fn new() -> Self {
                Self(0)
            }

            /// Returns the complete internal value.
            #[inline(always)]
            pub const fn value(&self) -> $int {
                self.0
            }

            /// Returns a boolean value whether the specified flag is set.
            #[inline(always)]
            pub const fn bit(&self, position: u8) -> bool {
                ((self.0 >> position) & 1) != 0
            }

            /// Returns a modified variant with the flag set to the specified value.
            #[inline(always)]
            pub const fn set_bit(&self, position: u8, value: bool) -> Self {
                let cleared = self.0 & !(1 << position);

                Self(cleared | ((value as $int) << position))
            }

            /// Returns a field (subset of bits) from the internal value.
            #[inline(always)]
            pub const fn field(&self, position: u8, size: u8) -> $int {
                // TODO: Wait for https://github.com/rust-lang/rust/issues/51999.
                // assert!(size > 0);
                // assert!(size as usize <= (core::mem::size_of::<$int>() * 8));
                // assert!(position as usize + size as usize <= (core::mem::size_of::<$int>() * 8));

                let shifted = self.0 >> position;

                let rest = size as $int % (core::mem::size_of::<$int>() * 8) as $int;
                let bit = (rest > 0) as $int;

                let limit = bit.wrapping_shl(rest as u32);
                let mask = limit.wrapping_sub((size > 0) as $int);
                let result = shifted & mask;

                result
            }

            /// Returns a modified variant with the field set to the specified value.
            #[inline(always)]
            pub const fn set_field(&self, position: u8, size: u8, value: $int) -> Self {
                // TODO: Wait for https://github.com/rust-lang/rust/issues/51999.
                // assert!(size > 0);
                // assert!(size as usize <= (core::mem::size_of::<$int>() * 8));
                // assert!(position as usize + size as usize <= (core::mem::size_of::<$int>() * 8));
                // assert!((1 as $int).wrapping_shl(size as u32).wrapping_sub(1) >= value);

                let rest = size as $int % (core::mem::size_of::<$int>() * 8) as $int;
                let bit = (rest > 0) as $int;

                let limit = bit.wrapping_shl(rest as u32);
                let negative_mask = limit.wrapping_sub((size > 0) as $int);
                let positioned_used_bits = negative_mask << position;
                let positioned_mask = !positioned_used_bits;
                let cleared = self.0 & positioned_mask;

                let shifted_value = value << position;

                let result = cleared | shifted_value;

                Self(result)
            }
        }

        #[cfg(test)]
        impl core::cmp::PartialEq<$int> for $name {
            fn eq(&self, other: &$int) -> bool {
                self.0 == *other
            }
        }
    };
}

BitField!(BitField8: u8);
BitField!(BitField16: u16);
BitField!(BitField32: u32);
BitField!(BitField64: u64);
BitField!(BitField128: u128);
BitField!(BitFieldSize: usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_functions() {
        let mut bf = BitField8::new();
        assert_eq!(bf, 0b0000_0000);
        assert_eq!(bf.value(), 0b0000_0000);
        assert_eq!(bf.field(0, 8), 0b0000_0000);
        assert_eq!(bf.field(1, 7), 0b000_0000);
        assert_eq!(bf.field(0, 4), 0b0000);
        assert_eq!(bf.field(1, 3), 0b000);
        assert_eq!(bf.field(1, 2), 0b00);
        assert_eq!(bf.field(2, 2), 0b00);
        assert!(!bf.bit(0));
        assert!(!bf.bit(1));
        assert!(!bf.bit(2));
        assert!(!bf.bit(3));

        bf = bf.set_bit(1, true);
        assert_eq!(bf, 0b0000_0010);
        assert_eq!(bf.value(), 0b0000_0010);
        assert_eq!(bf.field(0, 8), 0b0000_0010);
        assert_eq!(bf.field(1, 7), 0b000_0001);
        assert_eq!(bf.field(0, 4), 0b0010);
        assert_eq!(bf.field(0, 2), 0b10);
        assert_eq!(bf.field(1, 2), 0b01);
        assert!(!bf.bit(0));
        assert!(bf.bit(1));
        assert!(!bf.bit(2));
        assert!(!bf.bit(3));

        bf = bf.set_bit(2, true);
        assert_eq!(bf, 0b0000_0110);
        assert_eq!(bf.value(), 0b0000_0110);
        assert_eq!(bf.field(0, 8), 0b0000_0110);
        assert_eq!(bf.field(1, 7), 0b000_0011);
        assert_eq!(bf.field(0, 4), 0b0110);
        assert_eq!(bf.field(0, 2), 0b10);
        assert_eq!(bf.field(1, 2), 0b11);
        assert!(!bf.bit(0));
        assert!(bf.bit(1));
        assert!(bf.bit(2));
        assert!(!bf.bit(3));
        
        bf = bf.set_field(4, 4, 0b1111);
        assert_eq!(bf, 0b1111_0110);
        assert_eq!(bf.value(), 0b1111_0110);
        assert_eq!(bf.field(0, 8), 0b1111_0110);
        assert_eq!(bf.field(1, 7), 0b111_1011);

        bf = BitField8::new().set_field(0, 8, 0b1111_1111);
        assert_eq!(bf, 0b1111_1111);
        assert_eq!(bf.value(), 0b1111_1111);
        assert_eq!(bf.field(0, 8), 0b1111_1111);

        bf = BitField8::new().set_field(1, 7, 0b0111_1111);
        assert_eq!(bf, 0b1111_1110);
        assert_eq!(bf.value(), 0b1111_1110);
        assert_eq!(bf.field(0, 8), 0b1111_1110);

        bf = BitField8::new().set_field(0, 7, 0b0111_1111);
        assert_eq!(bf, 0b01111_111);
        assert_eq!(bf.value(), 0b01111_111);
        assert_eq!(bf.field(0, 8), 0b01111_111);
    }

    #[test]
    #[should_panic(expected = "attempt to shift right with overflow")]
    fn bounds_bit_position() {
        BitField8::new().bit(8);
    }

    /*
    // TODO: Wait for https://github.com/rust-lang/rust/issues/51999.
    #[test]
    #[should_panic(expected = "TODO")]
    fn bounds_field_zero_size() {
        BitField8::new().field(7, 0);
    }
    */

    #[test]
    #[should_panic(expected = "attempt to shift right with overflow")]
    fn bounds_field_position() {
        BitField8::new().field(8, 1);
    }

    /*
    // TODO: Wait for https://github.com/rust-lang/rust/issues/51999.
    #[test]
    #[should_panic(expected = "TODO")]
    fn bounds_field_size() {
        BitField8::new().field(0, 9);
    }

    #[test]
    #[should_panic(expected = "TODO")]
    fn bounds_field_combination() {
        BitField8::new().field(7, 2);
    }
    */

    #[test]
    #[should_panic(expected = "attempt to shift left with overflow")]
    fn bounds_set_bit_position() {
        let _ = BitField8::new().set_bit(8, true);
    }

    /*
    // TODO: Wait for https://github.com/rust-lang/rust/issues/51999.
    #[test]
    #[should_panic(expected = "TODO")]
    fn bounds_set_field_zero_size() {
        let _ = BitField8::new().set_field(7, 0, 0);
    }
    */

    #[test]
    #[should_panic(expected = "attempt to shift left with overflow")]
    fn bounds_set_field_position() {
        let _ = BitField8::new().set_field(8, 1, 0);
    }

    /*
    // TODO: Wait for https://github.com/rust-lang/rust/issues/51999.
    #[test]
    #[should_panic(expected = "TODO")]
    fn bounds_set_field_size() {
        let _ = BitField8::new().set_field(0, 9, 0);
    }

    #[test]
    #[should_panic(expected = "TODO")]
    fn bounds_set_field_combination() {
        let _ = BitField8::new().set_field(7, 2, 0);
    }

    #[test]
    #[should_panic(expected = "TODO")]
    fn bounds_set_field_value() {
        let _ = BitField8::new().set_field(0, 1, 2);
    }
    */
}