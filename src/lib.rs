//! # Bit fields for Rust
//!
//! Provides macros and types which simplify bit level access to primitive types in Rust.
//!
//! ## Dependencies
//!
//! None, the types work in a `#[no_std]` environment.
//!
//! ## Description
//!
//! A bit field can store simple boolean flags, as well as values of multiple bits in size. This
//! crate provides types based on the primitive types `u8`, `u16`, `u32`, `u64`, `u128` and `usize`,
//! which simplify bit level access to values of those primitive types.
//!
//! ## Simple example
//!
//! Imagine the following type which can store up to 16 boolean flags in a `u16` value:
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
//! With the help of this crate this can be expressed in a type safe way as follows:
//!
//! ```ignore
//! // Implementation
//!
//! extern crate alloc;
//!
//! bitfield::bit_field!(
//!     ExecutableFlags: u16;
//!     flags:
//!         has + set: ExecutableFlag
//! );
//!
//! #[derive(Clone, Copy, Debug)]
//! #[repr(u8)]
//! enum ExecutableFlag {
//!     System,
//!     Library,
//!     X64 = 3
//! }
//!
//! /// Instead of manually implementing this, `#[derive(enum_extensions::Iterator)]` of the
//! /// [enum_extensions](https://github.com/MauriceKayser/rs-enum_extensions)
//! /// crate can be used for automatic generation.
//! impl ExecutableFlag {
//!     const fn iter() -> &'static [Self] {
//!         &[ExecutableFlag::System, ExecutableFlag::Library, ExecutableFlag::X64]
//!     }
//! }
//!
//! // Usage
//!
//! extern "C" fn bla() -> ExecutableFlags;
//! extern "C" fn foo(executable_flags: ExecutableFlags);
//!
//! // Usage
//!
//! let executable_flags = bla().set(ExecutableFlag::System).set(ExecutableFlag::X64);
//!
//! if executable_flags.has(ExecutableFlag::Library) {
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
//! enum Button {
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
//! enum DefaultButton {
//!     One,
//!     Two,
//!     Three,
//!     Four
//!     // Values `4` - `7` are unused.
//! }
//!
//! #[repr(u8)]
//! enum Icon {
//!     None,
//!     Error,
//!     Question,
//!     Warning,
//!     Information
//!     // Values `5` - `7` are unused.
//! }
//!
//! #[repr(u8)]
//! enum Modality {
//!     Application,
//!     System,
//!     Task
//!     // Value `3` is unused.
//! }
//! ```
//!
//! The 32-bit wide `Styles` bit field representing this structure can be generated like this:
//!
//! ```ignore
//! extern crate alloc;
//!
//! #[derive(Copy, Clone, Debug)]
//! #[repr(u8)]
//! enum Button {
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
//! /// Instead of manually implementing this, `#[derive(enum_extensions::FromPrimitive)]` of the
//! /// [enum_extensions](https://github.com/MauriceKayser/rs-enum_extensions)
//! /// crate can be used for automatic generation.
//! impl core::convert::TryFrom<u8> for Button {
//!     type Error = u8;
//!
//!     fn try_from(value: u8) -> Result<Self, Self::Error> {
//!         match value {
//!             v if v == Self::Ok                  as u8 => Ok(Self::Ok),
//!             v if v == Self::OkCancel            as u8 => Ok(Self::OkCancel),
//!             v if v == Self::AbortRetryIgnore    as u8 => Ok(Self::AbortRetryIgnore),
//!             v if v == Self::YesNoCancel         as u8 => Ok(Self::YesNoCancel),
//!             v if v == Self::YesNo               as u8 => Ok(Self::YesNo),
//!             v if v == Self::RetryCancel         as u8 => Ok(Self::RetryCancel),
//!             v if v == Self::CancelTryContinue   as u8 => Ok(Self::CancelTryContinue),
//!             _ => Err(value),
//!         }
//!     }
//! }
//!
//! #[derive(Copy, Clone, Debug, enum_extensions::FromPrimitive)]
//! #[repr(u8)]
//! enum DefaultButton {
//!     One,
//!     Two,
//!     Three,
//!     Four
//!     // Values `4` - `7` are unused.
//! }
//!
//! #[derive(Copy, Clone, Debug, enum_extensions::FromPrimitive)]
//! #[repr(u8)]
//! enum Icon {
//!     None,
//!     Error,
//!     Question,
//!     Warning,
//!     Information
//!     // Values `5` - `7` are unused.
//! }
//!
//! #[derive(Copy, Clone, Debug, enum_extensions::FromPrimitive)]
//! #[repr(u8)]
//! enum Modality {
//!     Application,
//!     System,
//!     Task
//!     // Value `3` is unused.
//! }
//!
//! #[derive(Clone, Copy, Debug, enum_extensions::Iterator)]
//! #[repr(u8)]
//! enum Style {
//!     Help = 14,
//!     SetForeground = 16,
//!     DefaultDesktopOnly,
//!     TopMost,
//!     Right,
//!     RightToLeftReading,
//!     ServiceNotification
//! }
//!
//! bitfield::bit_field!(
//!     /// MessageBox styles, see [user32.MessageBox function](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messagebox).
//!     Styles: u32;
//!     flags:
//!         // Flags spanning bits 14-21.
//!         has            + set:                Style;
//!     fields:
//!         // Field spanning bits 0-3.
//!         button         + set_button:         Button[u8:        0,  4]
//!         // Field spanning bits 4-7.
//!         icon           + set_icon:           Icon[u8:          4,  4]
//!         // Field spanning bits 8-11.
//!         default_button + set_default_button: DefaultButton[u8: 8,  4]
//!         // Field spanning bits 12-13.
//!         modality       + set_modality:       Modality[u8:      12, 2]
//! );
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
//! // `Button == Button` requires `#[derive(PartialEq)]` for `Button`.
//! if styles.has(Style::Help) && styles.button() == Ok(Button::OkCancel) {
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
extern crate alloc;
#[cfg(test)]
extern crate std;

/// This macro generates a bit field structure, a constructur, a getter and setter for each flag
/// enumeration and field, a `core::fmt::Debug` and for pure flag bit fields a `core::fmt::Display`
/// implementation.
///
/// Example code (one flag type):
///
/// ```rust
/// // Implementation
///
/// extern crate alloc;
///
/// bitfield::bit_field!(
///     pub(crate) Flags: u8;
///     flags:
///         pub(crate) has + pub(crate) set: Flag
/// );
///
/// #[derive(Clone, Copy, Debug)]
/// #[repr(u8)]
/// pub(crate) enum Flag {
///     F0,
///     F1,
///     F2,
///     FMax = 7
/// }
///
/// impl Flag {
///     const fn iter() -> &'static [Self] {
///         &[Flag::F0, Flag::F1, Flag::F2, Flag::FMax]
///     }
/// }
///
/// // Tests
///
/// let mut flags = Flags::new();
///
/// assert!(!flags.has(Flag::F2));
///
/// flags = flags.set(Flag::F2, true);
/// assert!(flags.has(Flag::F2));
///
/// flags = flags.set(Flag::FMax, true);
/// assert!(flags.has(Flag::FMax));
///
/// assert_eq!(&alloc::format!("{}", &flags), "F2 | FMax");
/// assert_eq!(
///     &alloc::format!("{:?}", &flags),
///     "Flags { F0: false, F1: false, F2: true, FMax: true }"
/// );
/// ```
///
/// Example code (multiple flag types):
///
/// ```rust
/// // Implementation
///
/// extern crate alloc;
///
/// bitfield::bit_field!(
///     pub(crate) FileFlags: u32;
///     flags:
///         pub(crate) has        + pub(crate) set:        FileFlag,
///         pub(crate) has_object + pub(crate) set_object: ObjectFlag
/// );
///
/// bitfield::bit_field!(
///     pub(crate) ProcessFlags: u32;
///     flags:
///         pub(crate) has        + pub(crate) set:        ProcessFlag,
///         pub(crate) has_object + pub(crate) set_object: ObjectFlag
/// );
///
/// // File object specific access flags, lower 16 bits.
/// #[derive(Copy, Clone, Debug)]
/// #[repr(u8)]
/// pub(crate) enum FileFlag {
///     Read,
///     Write,
///     Append,
///     Execute
/// }
///
/// impl FileFlag {
///     const fn iter() -> &'static [Self] {
///         &[FileFlag::Read, FileFlag::Write, FileFlag::Append, FileFlag::Execute]
///     }
/// }
///
/// // Process object specific access flags, lower 16 bits.
/// #[derive(Copy, Clone, Debug)]
/// #[repr(u8)]
/// pub(crate) enum ProcessFlag {
///     Terminate,
///     SuspendResume,
///     ReadVirtualMemory,
///     WriteVirtualMemory
/// }
///
/// impl ProcessFlag {
///     const fn iter() -> &'static [Self] {
///         &[
///             ProcessFlag::Terminate, ProcessFlag::SuspendResume,
///             ProcessFlag::ReadVirtualMemory, ProcessFlag::WriteVirtualMemory
///         ]
///     }
/// }
///
/// // General object access flags, upper 16 bits.
/// #[derive(Copy, Clone, Debug)]
/// #[repr(u8)]
/// pub(crate) enum ObjectFlag {
///     Delete = 16,
///     Synchronize
/// }
///
/// impl ObjectFlag {
///     const fn iter() -> &'static [Self] {
///         &[ObjectFlag::Delete, ObjectFlag::Synchronize]
///     }
/// }
///
/// // Tests
///
/// let mut file_flags = FileFlags::new();
///
/// assert!(!file_flags.has(FileFlag::Write));
/// file_flags = file_flags.set(FileFlag::Write, true);
/// assert!(file_flags.has(FileFlag::Write));
///
/// assert!(!file_flags.has_object(ObjectFlag::Delete));
/// file_flags = file_flags.set_object(ObjectFlag::Delete, true);
/// assert!(file_flags.has_object(ObjectFlag::Delete));
///
/// assert_eq!(&alloc::format!("{}", &file_flags), "Write | Delete");
/// assert_eq!(
///     &alloc::format!("{:?}", &file_flags),
///     "FileFlags { Read: false, Write: true, Append: false, Execute: false, Delete: true, Synchronize: false }"
/// );
///
/// let mut process_flags = ProcessFlags::new();
///
/// assert!(!process_flags.has(ProcessFlag::SuspendResume));
/// process_flags = process_flags.set(ProcessFlag::SuspendResume, true);
/// assert!(process_flags.has(ProcessFlag::SuspendResume));
///
/// assert!(!process_flags.has_object(ObjectFlag::Delete));
/// process_flags = process_flags.set_object(ObjectFlag::Delete, true);
/// assert!(process_flags.has_object(ObjectFlag::Delete));
///
/// assert_eq!(&alloc::format!("{}", &process_flags), "SuspendResume | Delete");
/// assert_eq!(
///     &alloc::format!("{:?}", &process_flags),
///     "ProcessFlags { Terminate: false, SuspendResume: true, ReadVirtualMemory: false, WriteVirtualMemory: false, Delete: true, Synchronize: false }"
/// );
/// ```
///
/// Example code (multiple field types):
///
/// ```rust
/// // Implementation
///
/// extern crate alloc;
///
/// bitfield::bit_field!(
///     pub(crate) Field: u32;
///     fields:
///         // Field spanning bits 0-3.
///         pub(crate) button         + pub(crate) set_button:         Button[u8: 0, 4],
///         // Field spanning bits 4-7.
///         pub(crate) default_button + pub(crate) set_default_button: DefaultButton[u8: 4, 4]
/// );
///
/// #[derive(Debug, Eq, PartialEq)]
/// #[repr(u8)]
/// pub enum Button {
///     Ok,
///     OkCancel,
///     AbortRetryIgnore,
///     YesNoCancel,
///     YesNo,
///     RetryCancel,
///     CancelTryContinue
///     // Value `7` is unused.
/// }
///
/// #[derive(Debug, Eq, PartialEq)]
/// #[repr(u8)]
/// pub enum DefaultButton {
///     One,
///     Two,
///     Three,
///     Four
///     // Values `4` - `7` are unused.
/// }
///
/// impl core::convert::TryFrom<u8> for Button {
///     type Error = u8;
///
///     fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
///         match value {
///             v if v == Self::Ok                as u8 => Ok(Self::Ok),
///             v if v == Self::OkCancel          as u8 => Ok(Self::OkCancel),
///             v if v == Self::AbortRetryIgnore  as u8 => Ok(Self::AbortRetryIgnore),
///             v if v == Self::YesNoCancel       as u8 => Ok(Self::YesNoCancel),
///             v if v == Self::YesNo             as u8 => Ok(Self::YesNo),
///             v if v == Self::RetryCancel       as u8 => Ok(Self::RetryCancel),
///             v if v == Self::CancelTryContinue as u8 => Ok(Self::CancelTryContinue),
///             _ => Err(value)
///         }
///     }
/// }
///
/// impl core::convert::TryFrom<u8> for DefaultButton {
///     type Error = u8;
///
///     fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
///         match value {
///             v if v == Self::One   as u8 => Ok(Self::One),
///             v if v == Self::Two   as u8 => Ok(Self::Two),
///             v if v == Self::Three as u8 => Ok(Self::Three),
///             v if v == Self::Four  as u8 => Ok(Self::Four),
///             _ => Err(value)
///         }
///     }
/// }
///
/// // Tests
///
/// let mut field = Field::new();
///
/// assert_eq!(field.button(), Ok(Button::Ok));
/// field = field.set_button(Button::CancelTryContinue);
/// assert_eq!(field.button(), Ok(Button::CancelTryContinue));
///
/// assert_eq!(field.default_button(), Ok(DefaultButton::One));
/// field = field.set_default_button(DefaultButton::Four);
/// assert_eq!(field.default_button(), Ok(DefaultButton::Four));
///
/// assert_eq!(
///     &alloc::format!("{:?}", &field),
///     "Field { button: Ok(CancelTryContinue), default_button: Ok(Four) }"
/// );
/// ```
///
/// Example code (mixed):
///
/// ```rust
/// // Implementation
///
/// extern crate alloc;
///
/// bitfield::bit_field!(
///     pub(crate) Field: u32;
///     flags:
///         pub(crate) has + pub(crate) set: Flag;
///     fields:
///         // Field spanning bits 8-11.
///         pub(crate) button + pub(crate) set_button: Button[u8: 8, 4]
/// );
///
/// #[derive(Debug, Eq, PartialEq)]
/// #[repr(u8)]
/// pub enum Button {
///     Ok,
///     OkCancel,
///     AbortRetryIgnore,
///     YesNoCancel,
///     YesNo,
///     RetryCancel,
///     CancelTryContinue
///     // Value `7` is unused.
/// }
///
/// impl core::convert::TryFrom<u8> for Button {
///     type Error = u8;
///
///     fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
///         match value {
///             v if v == Self::Ok                as u8 => Ok(Self::Ok),
///             v if v == Self::OkCancel          as u8 => Ok(Self::OkCancel),
///             v if v == Self::AbortRetryIgnore  as u8 => Ok(Self::AbortRetryIgnore),
///             v if v == Self::YesNoCancel       as u8 => Ok(Self::YesNoCancel),
///             v if v == Self::YesNo             as u8 => Ok(Self::YesNo),
///             v if v == Self::RetryCancel       as u8 => Ok(Self::RetryCancel),
///             v if v == Self::CancelTryContinue as u8 => Ok(Self::CancelTryContinue),
///             _ => Err(value)
///         }
///     }
/// }
///
/// #[derive(Clone, Copy, Debug)]
/// #[repr(u8)]
/// pub(crate) enum Flag {
///     F0,
///     F1,
///     F2,
///     FMax = 7
/// }
///
/// impl Flag {
///     const fn iter() -> &'static [Self] {
///         &[Flag::F0, Flag::F1, Flag::F2, Flag::FMax]
///     }
/// }
///
/// // Tests
///
/// let mut field = Field::new();
///
/// assert!(!field.has(Flag::F2));
///
/// field = field.set(Flag::F2, true);
/// assert!(field.has(Flag::F2));
///
/// field = field.set(Flag::FMax, true);
/// assert!(field.has(Flag::FMax));
///
/// assert_eq!(field.button(), Ok(Button::Ok));
/// field = field.set_button(Button::CancelTryContinue);
/// assert_eq!(field.button(), Ok(Button::CancelTryContinue));
///
/// assert_eq!(
///     &alloc::format!("{:?}", &field),
///     "Field { button: Ok(CancelTryContinue), F0: false, F1: false, F2: true, FMax: true }"
/// );
/// ```
#[macro_export]
macro_rules! bit_field {
    // Flags only.
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u8;
        flags: $($flag_get_visibility:vis $flag_get:ident + $flag_set_visibility:vis $flag_set:ident : $flag_type:ty),+
    ) => {
        $crate::bit_field!(1
            $(#[$attr])* $visibility $bit_field : $crate::BitField8, u8;
            flags: $($flag_get_visibility $flag_get + $flag_set_visibility $flag_set : $flag_type),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u16;
        flags: $($flag_get_visibility:vis $flag_get:ident + $flag_set_visibility:vis $flag_set:ident : $flag_type:ty),+
    ) => {
        $crate::bit_field!(1
            $(#[$attr])* $visibility $bit_field : $crate::BitField16, u16;
            flags: $($flag_get_visibility $flag_get + $flag_set_visibility $flag_set : $flag_type),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u32;
        flags: $($flag_get_visibility:vis $flag_get:ident + $flag_set_visibility:vis $flag_set:ident : $flag_type:ty),+
    ) => {
        $crate::bit_field!(1
            $(#[$attr])* $visibility $bit_field : $crate::BitField32, u32;
            flags: $($flag_get_visibility $flag_get + $flag_set_visibility $flag_set : $flag_type),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u64;
        flags: $($flag_get_visibility:vis $flag_get:ident + $flag_set_visibility:vis $flag_set:ident : $flag_type:ty),+
    ) => {
        $crate::bit_field!(1
            $(#[$attr])* $visibility $bit_field : $crate::BitField64, u64;
            flags: $($flag_get_visibility $flag_get + $flag_set_visibility $flag_set : $flag_type),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u128;
        flags: $($flag_get_visibility:vis $flag_get:ident + $flag_set_visibility:vis $flag_set:ident : $flag_type:ty),+
    ) => {
        $crate::bit_field!(1
            $(#[$attr])* $visibility $bit_field : $crate::BitField128, u128;
            flags: $($flag_get_visibility $flag_get + $flag_set_visibility $flag_set : $flag_type),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : usize;
        flags: $($flag_get_visibility:vis $flag_get:ident + $flag_set_visibility:vis $flag_set:ident : $flag_type:ty),+
    ) => {
        $crate::bit_field!(1
            $(#[$attr])* $visibility $bit_field : $crate::BitFieldSize, usize;
            flags: $($flag_get_visibility $flag_get + $flag_set_visibility $flag_set : $flag_type),+
        );
    };
    // Flags and fields.
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u8
          ; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),+
        $(; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),* )?
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField8, u8;
            flags:    $($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),+;
            fields: $($($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),*)?
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u8
        $(; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),* )?
          ; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),+
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField8, u8;
            flags: $($($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),* )?;
            fields:  $($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u16
          ; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),+
        $(; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),* )?
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField16, u16;
            flags:    $($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),+;
            fields: $($($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),*)?
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u16
        $(; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),* )?
          ; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),+
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField16, u16;
            flags: $($($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),* )?;
            fields:  $($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u32
          ; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),+
        $(; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),* )?
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField32, u32;
            flags:    $($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),+;
            fields: $($($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),*)?
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u32
        $(; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),* )?
          ; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),+
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField32, u32;
            flags: $($($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),* )?;
            fields:  $($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u64
          ; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),+
        $(; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),* )?
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField64, u64;
            flags:    $($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),+;
            fields: $($($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),*)?
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u64
        $(; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),* )?
          ; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),+
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField64, u64;
            flags: $($($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),* )?;
            fields:  $($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u128
          ; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),+
        $(; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),* )?
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField128, u128;
            flags:    $($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),+;
            fields: $($($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),*)?
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : u128
        $(; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),* )?
          ; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),+
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitField128, u128;
            flags: $($($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),* )?;
            fields:  $($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),+
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : usize
          ; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),+
        $(; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),* )?
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitFieldSize, usize;
            flags:    $($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),+;
            fields: $($($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),*)?
        );
    };
    (
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : usize
        $(; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),* )?
          ; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),+
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $crate::BitFieldSize, usize;
            flags: $($($flag_get_visibility  $flag_get  + $flag_set_visibility  $flag_set  : $flag_type),* )?;
            fields:  $($field_get_visibility $field_get + $field_set_visibility $field_set : $field_type [$field_sub_type : $field_index, $field_size]),+
        );
    };
    // Flags only.
    (1
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : $bit_field_type:ty, $bit_field_sub_type:ty;
        flags: $($flag_get_visibility:vis $flag_get:ident + $flag_set_visibility:vis $flag_set:ident : $flag_type:ty),+
    ) => {
        $crate::bit_field!(2
            $(#[$attr])* $visibility $bit_field : $bit_field_type, $bit_field_sub_type;
            flags: $($flag_get_visibility $flag_get + $flag_set_visibility $flag_set : $flag_type),+;
            fields:
        );

        impl core::fmt::Display for $bit_field {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                let mut formatted = alloc::string::String::new();

                $(
                    for flag in <$flag_type>::iter() {
                        if self.0.bit(*flag as u8) {
                            if formatted.len() > 0 {
                                formatted.push_str(" | ");
                            }
                            formatted.push_str(&alloc::format!("{:?}", flag));
                        }
                    }
                )+

                if formatted.len() == 0 {
                    formatted.push('-');
                }

                f.write_str(formatted.as_ref())
            }
        }
    };
    // Flags and fields.
    (2
        $(#[$attr:meta])* $visibility:vis $bit_field:ident : $bit_field_type:ty, $bit_field_sub_type:ty
        ; flags:  $($flag_get_visibility:vis  $flag_get:ident  + $flag_set_visibility:vis  $flag_set:ident  : $flag_type:ty),*
        ; fields: $($field_get_visibility:vis $field_get:ident + $field_set_visibility:vis $field_set:ident : $field_type:ty [$field_sub_type:ty : $field_index:expr, $field_size:expr]),*
    ) => {
        $(#[$attr])* $visibility struct $bit_field($bit_field_type);

        impl $bit_field {
            /// Creates a new instance with all flags set to `false`.
            #[inline(always)]
            $visibility const fn new() -> Self {
                Self(<$bit_field_type>::new())
            }

            // Generate flag getters and setters.
            $(
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                $flag_get_visibility const fn $flag_get(&self, flag: $flag_type) -> bool {
                    self.0.bit(flag as u8)
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                $flag_set_visibility const fn $flag_set(self, flag: $flag_type, value: bool) -> Self {
                    Self(self.0.set_bit(flag as u8, value))
                }
            )*

            // Generate field getters and setters.
            $(
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                $field_get_visibility fn $field_get(&self) -> core::result::Result<$field_type, $field_sub_type> {
                    core::convert::TryInto::<$field_type>::try_into(
                        self.0.field($field_index, $field_size) as $field_sub_type
                    )
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                $field_set_visibility const fn $field_set(&self, value: $field_type) -> Self {
                    Self(self.0.set_field($field_index, $field_size, value as $bit_field_sub_type))
                }
            )*
        }

        impl core::fmt::Debug for $bit_field {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                let mut f = f.debug_struct(stringify!($bit_field));

                // Generate field related output fields.
                $(
                    f.field(stringify!($field_get), &self.$field_get());
                )*

                // Generate flag related output fields.
                $(
                    for flag in <$flag_type>::iter() {
                        f.field(&alloc::format!("{:?}", flag), &self.0.bit(*flag as u8));
                    }
                )*

                f.finish()
            }
        }
    };
}

macro_rules! create_bit_field_type {
    ($name:ident: $int:ident) => {
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

            /// Returns a modified instance with the flag set to the specified value.
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

create_bit_field_type!(BitField8: u8);
create_bit_field_type!(BitField16: u16);
create_bit_field_type!(BitField32: u32);
create_bit_field_type!(BitField64: u64);
create_bit_field_type!(BitField128: u128);
create_bit_field_type!(BitFieldSize: usize);

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