# Bit fields for Rust

Provides macros and types which simplify bit level access to primitive types in Rust.

## Dependencies

None, the types work in a `#[no_std]` environment.

## Description

A bit field can store simple boolean flags, as well as values of multiple bits in size. This
crate provides types based on the primitive types `u8`, `u16`, `u32`, `u64`, `u128` and `usize`,
which simplify bit level access to values of those primitive types.

## Simple example

Imagine the following type which can store up to 16 boolean flags in a `u16` value:

```rust
pub const IS_SYSTEM:    u16 = 1 << 0; // 1
pub const IS_LIBRARY:   u16 = 1 << 1; // 2
// Undefined:                 1 << 2; // 4
pub const IS_X64:       u16 = 1 << 3; // 8
// ... up to 12 more flags ...

extern "C" fn bla() -> u16;
extern "C" fn foo(executable_flags: u16);

// Usage

let mut executable_flags = bla();

// Add the system and x64 flags.
executable_flags |= IS_SYSTEM | IS_X64;

// Execute `foo` if the library flag is set.
if (executable_flags & IS_LIBRARY) != 0 {
    foo(executable_flags);
}
```

With the help of this crate this can be expressed in a type safe way as follows:

```rust
// Implementation

extern crate alloc;

bitfield::bit_field!(
    ExecutableFlags: u16;
    flags:
        has + set: ExecutableFlag
);

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum ExecutableFlag {
    System,
    Library,
    X64 = 3
}

/// Instead of manually implementing this, `#[derive(enum_extensions::Iterator)]` of the
/// [enum_extensions](https://github.com/MauriceKayser/rs-enum_extensions)
/// crate can be used for automatic generation.
impl ExecutableFlag {
    const fn iter() -> &'static [Self] {
        &[ExecutableFlag::System, ExecutableFlag::Library, ExecutableFlag::X64]
    }
}

// Usage

extern "C" fn bla() -> ExecutableFlags;
extern "C" fn foo(executable_flags: ExecutableFlags);

// Usage

let executable_flags = bla().set(ExecutableFlag::System).set(ExecutableFlag::X64);

if executable_flags.has(ExecutableFlag::Library) {
    foo(executable_flags);
}
```

## Detailed Example

This example is based on the 4. parameter `UINT uType` of Microsoft Windows
[user32.MessageBox function](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messagebox)
which not only stores boolean flags, but also fields with more than one bit in size.

A Microsoft Visual C++ `UINT` is a `u32` in Rust. So all constants for the parameter `uType` can be written as follows:

```rust
// Buttons
const MB_BUTTON_OK:                  u32 = 0;
const MB_BUTTON_OK_CANCEL:           u32 = 1;
const MB_BUTTON_ABORT_RETRY_IGNORE:  u32 = 2;
const MB_BUTTON_YES_NO_CANCEL:       u32 = 3;
const MB_BUTTON_YES_NO:              u32 = 4;
const MB_BUTTON_RETRY_CANCEL:        u32 = 5;
const MB_BUTTON_CANCEL_TRY_CONTINUE: u32 = 6;

// Icons
const MB_ICON_NONE:                  u32 = 0x00;
const MB_ICON_ERROR:                 u32 = 0x10;
const MB_ICON_QUESTION:              u32 = 0x20;
const MB_ICON_EXCLAMATION:           u32 = 0x30;
const MB_ICON_INFORMATION:           u32 = 0x40;

// Default buttons
const MB_DEFAULT_BUTTON1:            u32 = 0x000;
const MB_DEFAULT_BUTTON2:            u32 = 0x100;
const MB_DEFAULT_BUTTON3:            u32 = 0x200;
const MB_DEFAULT_BUTTON4:            u32 = 0x300;

// Modality
const MB_MODALITY_APPLICATION:       u32 = 0x0000;
const MB_MODALITY_SYSTEM:            u32 = 0x1000;
const MB_MODALITY_TASK:              u32 = 0x2000;

// Other flags
const MB_HELP:                       u32 = 1 << 14;
const MB_FOREGROUND:                 u32 = 1 << 16;
const MB_DEFAULT_DESKTOP_ONLY:       u32 = 1 << 17;
const MB_TOP_MOST:                   u32 = 1 << 18;
const MB_RIGHT:                      u32 = 1 << 19;
const MB_RIGHT_TO_LEFT_READING:      u32 = 1 << 20;
const MB_SERVICE_NOTIFICATION:       u32 = 1 << 21;
```

One problem is that `u32` is not type safe like an `enum` value, another is that the usage of an `u32` is error prone,
because several "flags" of the same "type group" (called a "field"), like a button, can be `|`-ed together
(f. e. `MB_BUTTON_ABORT_RETRY_IGNORE | MB_BUTTON_YES_NO`) which might result in some unexpected behaviour.
Checking if certain fields, like a button, have a specific value is also unnecessarily complicated (bit fiddling
operators like `>>` and `&` etc. are necessary).

The previously mentioned fields are stored in the `u32` value as follows:

| Type            | Min. def. value (`> 0`) | Max. def. value | Storage bits                                            | Max. storable value             |
| --------------- | ----------------------- | --------------- | ------------------------------------------------------- | ------------------------------- |
| `Button`        | 0x1                     | 0x6             | 0b0000_0000_0000_0000_0000_0000_0000_**XXXX** (0 - 4)   | `((1 << 4) - 1) <<  0` = 0x7    |
| `Icon`          | 0x10                    | 0x40            | 0b0000_0000_0000_0000_0000_0000_**XXXX**_0000 (4 - 8)   | `((1 << 4) - 1) <<  4` = 0x70   |
| `DefaultButton` | 0x100                   | 0x300           | 0b0000_0000_0000_0000_0000_**XXXX**_0000_0000 (8 - 12)  | `((1 << 4) - 1) <<  8` = 0x700  |
| `Modality`      | 0x1000                  | 0x2000          | 0b0000_0000_0000_0000_00**XX**_0000_0000_0000 (12 - 13) | `((1 << 2) - 1) << 12` = 0x3000 |

All of the fields can be expressed by shifting them to the right (removing the trailing zeros):

```rust
#[repr(u8)]
enum Button {
    Ok,
    OkCancel,
    AbortRetryIgnore,
    YesNoCancel,
    YesNo,
    RetryCancel,
    CancelTryContinue
    // Value `7` is unused.
}

#[repr(u8)]
enum DefaultButton {
    One,
    Two,
    Three,
    Four
    // Values `4` - `7` are unused.
}

#[repr(u8)]
enum Icon {
    None,
    Error,
    Question,
    Warning,
    Information
    // Values `5` - `7` are unused.
}

#[repr(u8)]
enum Modality {
    Application,
    System,
    Task
    // Value `3` is unused.
}
```

The 32-bit wide `Styles` bit field representing this structure can be generated like this:

```rust
extern crate alloc;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Button {
    Ok,
    OkCancel,
    AbortRetryIgnore,
    YesNoCancel,
    YesNo,
    RetryCancel,
    CancelTryContinue
    // Value `7` is unused.
}

/// Instead of manually implementing this, `#[derive(enum_extensions::FromPrimitive)]` of the
/// [enum_extensions](https://github.com/MauriceKayser/rs-enum_extensions)
/// crate can be used for automatic generation.
impl core::convert::TryFrom<u8> for Button {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == Self::Ok                  as u8 => Ok(Self::Ok),
            v if v == Self::OkCancel            as u8 => Ok(Self::OkCancel),
            v if v == Self::AbortRetryIgnore    as u8 => Ok(Self::AbortRetryIgnore),
            v if v == Self::YesNoCancel         as u8 => Ok(Self::YesNoCancel),
            v if v == Self::YesNo               as u8 => Ok(Self::YesNo),
            v if v == Self::RetryCancel         as u8 => Ok(Self::RetryCancel),
            v if v == Self::CancelTryContinue   as u8 => Ok(Self::CancelTryContinue),
            _ => Err(value),
        }
    }
}

#[derive(Copy, Clone, Debug, enum_extensions::FromPrimitive)]
#[repr(u8)]
enum DefaultButton {
    One,
    Two,
    Three,
    Four
    // Values `4` - `7` are unused.
}

#[derive(Copy, Clone, Debug, enum_extensions::FromPrimitive)]
#[repr(u8)]
enum Icon {
    None,
    Error,
    Question,
    Warning,
    Information
    // Values `5` - `7` are unused.
}

#[derive(Copy, Clone, Debug, enum_extensions::FromPrimitive)]
#[repr(u8)]
enum Modality {
    Application,
    System,
    Task
    // Value `3` is unused.
}

#[derive(Clone, Copy, Debug, enum_extensions::Iterator)]
#[repr(u8)]
enum Style {
    Help = 14,
    SetForeground = 16,
    DefaultDesktopOnly,
    TopMost,
    Right,
    RightToLeftReading,
    ServiceNotification
}

bitfield::bit_field!(
    /// MessageBox styles, see [user32.MessageBox function](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messagebox).
    Styles: u32;
    flags:
        // Flags spanning bits 14-21.
        has            + set:                Style;
    fields:
        // Field spanning bits 0-3.
        button         + set_button:         Button[u8:        0,  4]
        // Field spanning bits 4-7.
        icon           + set_icon:           Icon[u8:          4,  4]
        // Field spanning bits 8-11.
        default_button + set_default_button: DefaultButton[u8: 8,  4]
        // Field spanning bits 12-13.
        modality       + set_modality:       Modality[u8:      12, 2]
);
```

It can now be constructed and used as follows:

```rust
let styles = Styles::new()
    .set_button(Button::OkCancel)
    .set_icon(Icon::Information)
    .set(Style::Right, true)
    .set(Style::TopMost, true);

// `Button == Button` requires `#[derive(PartialEq)]` for `Button`.
if styles.has(Style::Help) && styles.button() == Ok(Button::OkCancel) {
    let result = user32::MessageBoxW(/* ... */, styles.set_button(Button::YesNo));
}
```

## TODO

- Bounds checking has to wait until [Allow panicking in constants](https://github.com/rust-lang/rust/issues/51999)
is merged.
- Update documentation if [RFC-2632](https://github.com/rust-lang/rfcs/pull/2632) is done.