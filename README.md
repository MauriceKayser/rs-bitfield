# Bitfields for Rust

A Rust macro to generate structures which behave like bitfields.

## Dependencies

- Optionally a [from_primitive](https://github.com/mauricekayser/rs-from-primitive) like crate for enum conversions,
which generates from_*primitive_type* like functions.

## Simple example

Imagine the following type which can store up to 8 flags in a `u8` value:

```rust
pub const IS_SYSTEM:    u8 = 1 << 0; // 1
pub const IS_DLL:       u8 = 1 << 1; // 2
pub const IS_X64:       u8 = 1 << 2; // 4
// ... up to 5 more flags ...

fn do_stuff(information_flags: u8) { /* ... */ }

// ...
do_stuff(IS_SYSTEM | IS_X64);
// ...
```

With the help of this crate this can be expressed as follows:

```rust
extern crate bitfield;
use bitfield::bitfield;

/*
bitfield!(struct_visibility StructName(struct_base_type) {
    (flag_visibility flag_name: bit_position,)+
});
*/
bitfield!(pub Information(u8) {
    pub system: 0,
    pub dll:    1,
    pub x64:    2,
});
```

This results in the following generated code:

```rust
pub struct Information(u8);

#[derive(Default)]
pub struct InformationInit {
    system: bool,
    dll:    bool,
    x64:    bool,
}

impl Information {
    #[inline]
    pub fn system(&self) -> bool {
        let max_bit_value = 1;
        let positioned_bits = self.0 >> 0;
        positioned_bits & max_bit_value == 1
    }

    #[inline]
    pub fn set_system(&mut self, value: bool) {
        let positioned_bits = 1 << 0;
        let positioned_flags = (value as u8) << 0;
        let cleaned_flags = self.0 & !positioned_bits;
        self.0 = cleaned_flags | positioned_flags;
    }

    #[inline]
    pub fn dll(&self) -> bool {
        let max_bit_value = 1;
        let positioned_bits = self.0 >> 1;
        positioned_bits & max_bit_value == 1
    }

    #[inline]
    pub fn set_dll(&mut self, value: bool) {
        let positioned_bits = 1 << 1;
        let positioned_flags = (value as u8) << 1;
        let cleaned_flags = self.0 & !positioned_bits;
        self.0 = cleaned_flags | positioned_flags;
    }

    // ... same for `x64`.

    #[inline]
    pub fn new(init: InformationInit) -> Self {
        let mut s = Information(0);

        s.set_system(init.system);
        s.set_dll(init.dll);
        s.set_x64(init.x64);

        s
    }
}

// It can now be constructed (f. e. with default values) and used like so:

let mut info = Information::new(InformationInit {
    dll: true,
    ..Default::default()
});

// ... code ...

if !info.x64() {
    // ... code ...
    info.set_system(true);
}
```

## Detailed Example

This example is based on the 4. parameter `UINT uType` of Microsoft Windows
[user32.MessageBox function](https://msdn.microsoft.com/en-us/library/windows/desktop/ms645505.aspx) and not only stores
`bool`ean flags, but also `enum` values.

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
because several flags of the same "type group" like a button can be `|`-ed together
(f. e. `MB_BUTTON_ABORT_RETRY_IGNORE | MB_BUTTON_YES_NO`) which might result in some unexpected behaviour.
Checking if certain flags are set is also unnecessarily complicated.

The previously mentioned "type groups" are saved in the `u32` value as follows:

| Type            | Min. value (w/o 0) | Max. value | Storage bits                                            | Max. storable value             |
| --------------- | ------------------ | ---------- | ------------------------------------------------------- | ------------------------------- |
| `Button`        | 0x1                | 0x6        | 0b0000_0000_0000_0000_0000_0000_0000_0**XXX** (0 - 2)   | `((1 << 3) - 1) <<  0` = 0x7    |
| `Icon`          | 0x10               | 0x40       | 0b0000_0000_0000_0000_0000_0000_0**XXX**_0000 (4 - 6)   | `((1 << 3) - 1) <<  4` = 0x70   |
| `DefaultButton` | 0x100              | 0x300      | 0b0000_0000_0000_0000_0000_00**XX**_0000_0000 (8 - 9)   | `((1 << 2) - 1) <<  8` = 0x300  |
| `Modality`      | 0x1000             | 0x2000     | 0b0000_0000_0000_0000_00**XX**_0000_0000_0000 (12 - 13) | `((1 << 2) - 1) << 12` = 0x3000 |

All of the "type groups" can be expressed by rebasing them (removing the trailing zeros):

```rust
#[macro_use]
extern crate from_primitive;

#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum Button {
    #[default]
    Ok,
    OkCancel,
    AbortRetryIgnore,
    YesNoCancel,
    YesNo,
    RetryCancel,
    CancelTryContinue
}

#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum DefaultButton {
    #[default]
    One,
    Two,
    Three,
    Four
}

#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum Icon {
    #[default]
    None,
    Stop,
    Question,
    Exclamation,
    Information
}

#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum Modality {
    #[default]
    Application,
    System,
    Task
}
```

Now the `bitfield` macro can be used as follows:

```rust
/*
bitfield!(struct_visibility StructName(struct_base_type) {
    (
        (flag_visibility flag_name: bit_position,) |
        (flag_visibility flag_name: flag_base_type(bit_position, bit_amount),)
    )+
});
*/
bitfield!(pub Style(u32) {
    pub button:                 Button(0, 3),
    pub icon:                   Icon(4, 3),
    pub default_button:         DefaultButton(8, 2),
    pub modality:               Modality(12, 2),
    pub help:                   14,
    pub foreground:             16,
    pub default_desktop_only:   17,
    pub top_most:               18,
    pub right:                  19,
    pub right_to_left_reading:  20,
    pub service_notification:   21,
});
```

This results in the following generated code:

```rust
pub struct Style(u32);

#[derive(Default)]
pub struct StyleInit {
    button:     Button,
    icon:       Icon,
    // ...
    help:       bool,
    foreground: bool,
    // ...
}

impl Style {
    #[inline]
    pub fn button(&self) -> result::Result<Button, u32> {
        const MAX_BIT_VALUE: u32 = (1 << 3) - 1;
        let positioned_bits = self.0 >> 0;
        let value = positioned_bits & MAX_BIT_VALUE;
        let enum_value = Button::from_u32(value as u32);
        if enum_value.is_some() {
            Ok(enum_value.unwrap())
        } else { Err(value) }
    }

    #[inline]
    pub fn set_button(&mut self, value: Button) {
        const MAX_BIT_VALUE: u32 = (1 << 3) - 1;
        const POSITIONED_BITS = MAX_BIT_VALUE << 0;
        let positioned_flags = (value as u32) << 0;
        let cleaned_flags = self.0 & !POSITIONED_BITS;
        self.0 = cleaned_flags | positioned_flags;
    }

    #[inline]
    pub fn icon(&self) -> result::Result<Icon, u32> {
        const MAX_BIT_VALUE: u32 = (1 << 3) - 1;
        let positioned_bits = self.0 >> 4;
        let value = positioned_bits & MAX_BIT_VALUE;
        let enum_value = Icon::from_u32(value as u32);
        if enum_value.is_some() {
            Ok(enum_value.unwrap())
        } else { Err(value) }
    }

    #[inline]
    pub fn set_icon(&mut self, value: Icon) {
        const MAX_BIT_VALUE: u32 = (1 << 3) - 1;
        const POSITIONED_BITS = MAX_BIT_VALUE << 4;
        let positioned_flags = (value as u32) << 4;
        let cleaned_flags = self.0 & !POSITIONED_BITS;
        self.0 = cleaned_flags | positioned_flags;
    }

    // ...

    #[inline]
    pub fn help(&self) -> bool {
        let max_bit_value: u32 = 1;
        let positioned_bits = self.0 >> 14;
        positioned_bits & max_bit_value == 1
    }

    #[inline]
    pub fn set_help(&mut self, value: bool) {
        let positioned_bits: u32 = 1 << 14;
        let positioned_flags = (value as u32) << 14;
        let cleaned_flags = self.0 & !positioned_bits;
        self.0 = cleaned_flags | positioned_flags;
    }

    // ...

    #[inline]
    pub fn new(init: StyleInit) -> Self {
        let mut s = Style(0);

        s.set_button(init.button);
        s.set_icon(init.icon);
        // ...

        s.set_help(init.help);
        s.set_foreground(init.foreground);
        // ...

        s
    }
}
```

It can now be constructed (f. e. with default values) and used like so:

```rust
let mut style = Style::new(StyleInit {
    button: Button::OkCancel,
    icon: Icon::Information,
    right: true,
    ..Default::default()
});

// ... code ...

if style.right() && style.button() == Button::Ok {
    // ... code ...
    style.set_button(Button::OkCancel);
}
```

## Overlap Example

Some systems reuse/overlap bits for different meanings, for example Microsoft Windows
[Memory Protection Constants](https://docs.microsoft.com/en-us/windows/desktop/Memory/memory-protection-constants)
type has 2x2 flags which overlap: `PAGE_TARGETS_INVALID` with `PAGE_TARGETS_NO_UPDATE` and
`PAGE_ENCLAVE_THREAD_CONTROL` with `PAGE_ENCLAVE_UNVALIDATED`.
By default overlapping fields will result in a `panic`, but it can be disabled by explicitly
indicating that the overlapping is wanted:

```rust
bitfield!(pub MemoryProtection(u32) {
    pub no_access:              0,

    pub read_only:              1,
    pub read_write:             2,
    pub write_copy:             3,

    pub execute:                4,
    pub execute_read:           5,
    pub execute_read_write:     6,
    pub execute_write_copy:     7,

    pub guard:                  8,
    pub no_cache:               9,
    pub write_combine:          10,

    #[allow_overlap(targets_no_update)]
    pub targets_invalid:        30,
    #[allow_overlap(targets_invalid)]
    pub targets_no_update:      30,

    #[allow_overlap(enclave_unvalidated)]
    pub enclave_thread_control: 31,
    #[allow_overlap(enclave_thread_control)]
    pub enclave_unvalidated:    31,
});
```

This behaviour also works for enum values with more than one bit in size.

## TODO

- Calculate whether the biggest enum value fits in `bit_amount`.
- Allow `Expr` for flag offsets and ranges.
- Generate function `unused_bits() -> #base_type { /* ... */ }`
- Check for unnecessary `allow_overlap` attributes and attribute members.