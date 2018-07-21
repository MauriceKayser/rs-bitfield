//! # Bitfields for Rust
//!
//! A Rust macro to generate structures which behave like bitfields.
//!
//! ## Dependencies
//!
//! - Optionally a [from_primitive](https://github.com/mauricekayser/rs-from-primitive) like crate for enum conversions,
//! which generates from_*primitive_type* like functions.
//!
//! ## Simple example
//!
//! Imagine the following type which can store up to 8 flags in a `u8` value:
//!
//! ```rust
//! pub const IS_SYSTEM:    u8 = 1 << 0; // 1
//! pub const IS_DLL:       u8 = 1 << 1; // 2
//! pub const IS_X64:       u8 = 1 << 2; // 4
//! // ... up to 5 more flags ...
//!
//! fn do_stuff(information_flags: u8) { /* ... */ }
//!
//! // ...
//! do_stuff(IS_SYSTEM | IS_X64);
//! // ...
//! ```
//!
//! With the help of this crate this can be expressed as follows:
//!
//! ```rust
//! extern crate bitfield;
//! use bitfield::bitfield;
//!
//! /*
//! bitfield!(struct_visibility StructName(struct_base_type) {
//!     (flag_visibility flag_name: bit_position,)+
//! });
//! */
//! bitfield!(pub Information(u8) {
//!     pub system: 0,
//!     pub dll:    1,
//!     pub x64:    2,
//! });
//! ```
//!
//! This results in the following generated code:
//!
//! ```rust
//! pub struct Information(u8);
//!
//! #[derive(Default)]
//! pub struct InformationInit {
//!     system: bool,
//!     dll:    bool,
//!     x64:    bool,
//! }
//!
//! impl Information {
//!     #[inline]
//!     pub fn system(&self) -> bool {
//!         let max_bit_value = 1;
//!         let positioned_bits = self.0 >> 0;
//!         positioned_bits & max_bit_value == 1
//!     }
//!
//!     #[inline]
//!     pub fn set_system(&mut self, value: bool) {
//!         let positioned_bits = 1 << 0;
//!         let positioned_flags = (value as u8) << 0;
//!         let cleaned_flags = self.0 & !positioned_bits;
//!         self.0 = cleaned_flags | positioned_flags;
//!     }
//!
//!     #[inline]
//!     pub fn dll(&self) -> bool {
//!         let max_bit_value = 1;
//!         let positioned_bits = self.0 >> 1;
//!         positioned_bits & max_bit_value == 1
//!     }
//!
//!     #[inline]
//!     pub fn set_dll(&mut self, value: bool) {
//!         let positioned_bits = 1 << 1;
//!         let positioned_flags = (value as u8) << 1;
//!         let cleaned_flags = self.0 & !positioned_bits;
//!         self.0 = cleaned_flags | positioned_flags;
//!     }
//!
//!     // ... same for `x64`.
//!
//!     #[inline]
//!     pub fn new(init: InformationInit) -> Self {
//!         let mut s = Information(0);
//!
//!         s.set_system(init.system);
//!         s.set_dll(init.dll);
//!         s.set_x64(init.x64);
//!
//!         s
//!     }
//! }
//!
//! // It can now be constructed (f. e. with default values) and used like so:
//!
//! let mut info = Information::new(InformationInit {
//!     dll: true,
//!     ..Default::default()
//! });
//!
//! // ... code ...
//!
//! if !info.x64() {
//!     // ... code ...
//!     info.set_system(true);
//! }
//! ```
//!
//! ## Detailed Example
//!
//! This example is based on the 4. parameter `UINT uType` of Microsoft Windows
//! [user32.MessageBox function](https://msdn.microsoft.com/en-us/library/windows/desktop/ms645505.aspx) and not only stores
//! `bool`ean flags, but also `enum` values.
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
//! because several flags of the same "type group" like a button can be `|`-ed together
//! (f. e. `MB_BUTTON_ABORT_RETRY_IGNORE | MB_BUTTON_YES_NO`) which might result in some unexpected behaviour.
//! Checking if certain flags are set is also unnecessarily complicated.
//!
//! The previously mentioned "type groups" are saved in the `u32` value as follows:
//!
//! | Type            | Min. value (w/o 0) | Max. value | Storage bits                                            | Max. storable value             |
//! | --------------- | ------------------ | ---------- | ------------------------------------------------------- | ------------------------------- |
//! | `Button`        | 0x1                | 0x6        | 0b0000_0000_0000_0000_0000_0000_0000_0**XXX** (0 - 2)   | `((1 << 3) - 1) <<  0` = 0x7    |
//! | `Icon`          | 0x10               | 0x40       | 0b0000_0000_0000_0000_0000_0000_0**XXX**_0000 (4 - 6)   | `((1 << 3) - 1) <<  4` = 0x70   |
//! | `DefaultButton` | 0x100              | 0x300      | 0b0000_0000_0000_0000_0000_00**XX**_0000_0000 (8 - 9)   | `((1 << 2) - 1) <<  8` = 0x300  |
//! | `Modality`      | 0x1000             | 0x2000     | 0b0000_0000_0000_0000_00**XX**_0000_0000_0000 (12 - 13) | `((1 << 2) - 1) << 12` = 0x3000 |
//!
//! All of the "type groups" can be expressed by rebasing them (removing the trailing zeros):
//!
//! ```rust
//! #[macro_use]
//! extern crate from_primitive;
//!
//! #[repr(u32)]
//! #[derive(Debug, FromPrimitive, PartialEq)]
//! pub enum Button {
//!     #[default]
//!     Ok,
//!     OkCancel,
//!     AbortRetryIgnore,
//!     YesNoCancel,
//!     YesNo,
//!     RetryCancel,
//!     CancelTryContinue
//! }
//!
//! #[repr(u32)]
//! #[derive(Debug, FromPrimitive, PartialEq)]
//! pub enum DefaultButton {
//!     #[default]
//!     One,
//!     Two,
//!     Three,
//!     Four
//! }
//!
//! #[repr(u32)]
//! #[derive(Debug, FromPrimitive, PartialEq)]
//! pub enum Icon {
//!     #[default]
//!     None,
//!     Stop,
//!     Question,
//!     Exclamation,
//!     Information
//! }
//!
//! #[repr(u32)]
//! #[derive(Debug, FromPrimitive, PartialEq)]
//! pub enum Modality {
//!     #[default]
//!     Application,
//!     System,
//!     Task
//! }
//! ```
//!
//! Now the `bitfield` macro can be used as follows:
//!
//! ```rust
//! /*
//! bitfield!(struct_visibility StructName(struct_base_type) {
//!     (
//!         (flag_visibility flag_name: bit_position,) |
//!         (flag_visibility flag_name: flag_base_type(bit_position, bit_amount),)
//!     )+
//! });
//! */
//! bitfield!(pub Style(u32) {
//!     pub button:                 Button(0, 3),
//!     pub icon:                   Icon(4, 3),
//!     pub default_button:         DefaultButton(8, 2),
//!     pub modality:               Modality(12, 2),
//!     pub help:                   14,
//!     pub foreground:             16,
//!     pub default_desktop_only:   17,
//!     pub top_most:               18,
//!     pub right:                  19,
//!     pub right_to_left_reading:  20,
//!     pub service_notification:   21,
//! });
//! ```
//!
//! This results in the following generated code:
//!
//! ```rust
//! pub struct Style(u32);
//!
//! #[derive(Default)]
//! pub struct StyleInit {
//!     button:     Button,
//!     icon:       Icon,
//!     // ...
//!     help:       bool,
//!     foreground: bool,
//!     // ...
//! }
//!
//! impl Style {
//!     #[inline]
//!     pub fn button(&self) -> result::Result<Button, u32> {
//!         const MAX_BIT_VALUE: u32 = (1 << 3) - 1;
//!         let positioned_bits = self.0 >> 0;
//!         let value = positioned_bits & MAX_BIT_VALUE;
//!         let enum_value = Button::from_u32(value as u32);
//!         if enum_value.is_some() {
//!             Ok(enum_value.unwrap())
//!         } else { Err(value) }
//!     }
//!
//!     #[inline]
//!     pub fn set_button(&mut self, value: Button) {
//!         const MAX_BIT_VALUE: u32 = (1 << 3) - 1;
//!         const POSITIONED_BITS = MAX_BIT_VALUE << 0;
//!         let positioned_flags = (value as u32) << 0;
//!         let cleaned_flags = self.0 & !POSITIONED_BITS;
//!         self.0 = cleaned_flags | positioned_flags;
//!     }
//!
//!     #[inline]
//!     pub fn icon(&self) -> result::Result<Icon, u32> {
//!         const MAX_BIT_VALUE: u32 = (1 << 3) - 1;
//!         let positioned_bits = self.0 >> 4;
//!         let value = positioned_bits & MAX_BIT_VALUE;
//!         let enum_value = Icon::from_u32(value as u32);
//!         if enum_value.is_some() {
//!             Ok(enum_value.unwrap())
//!         } else { Err(value) }
//!     }
//!
//!     #[inline]
//!     pub fn set_icon(&mut self, value: Icon) {
//!         const MAX_BIT_VALUE: u32 = (1 << 3) - 1;
//!         const POSITIONED_BITS = MAX_BIT_VALUE << 4;
//!         let positioned_flags = (value as u32) << 4;
//!         let cleaned_flags = self.0 & !POSITIONED_BITS;
//!         self.0 = cleaned_flags | positioned_flags;
//!     }
//!
//!     // ...
//!
//!     #[inline]
//!     pub fn help(&self) -> bool {
//!         let max_bit_value: u32 = 1;
//!         let positioned_bits = self.0 >> 14;
//!         positioned_bits & max_bit_value == 1
//!     }
//!
//!     #[inline]
//!     pub fn set_help(&mut self, value: bool) {
//!         let positioned_bits: u32 = 1 << 14;
//!         let positioned_flags = (value as u32) << 14;
//!         let cleaned_flags = self.0 & !positioned_bits;
//!         self.0 = cleaned_flags | positioned_flags;
//!     }
//!
//!     // ...
//!
//!     #[inline]
//!     pub fn new(init: StyleInit) -> Self {
//!         let mut s = Style(0);
//!
//!         s.set_button(init.button);
//!         s.set_icon(init.icon);
//!         // ...
//!
//!         s.set_help(init.help);
//!         s.set_foreground(init.foreground);
//!         // ...
//!
//!         s
//!     }
//! }
//! ```
//!
//! It can now be constructed (f. e. with default values) and used like so:
//!
//! ```rust
//! let mut style = Style::new(StyleInit {
//!     button: Button::OkCancel,
//!     icon: Icon::Information,
//!     right: true,
//!     ..Default::default()
//! });
//!
//! // ... code ...
//!
//! if style.right() && style.button() == Button::Ok {
//!     // ... code ...
//!     style.set_button(Button::OkCancel);
//! }
//! ```
//!
//! ## TODO
//!
//! - Caculate whether biggest enum value fits in `bit_amount`.
//! - Allow `Expr` for flag offsets and ranges.
//! - Generate function `unused_bits() -> #base_type { /* ... */ }`
//! - Check for unnecessary `allow_overlap` attributes and attribute members.

#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

// Macro

use syn::{Attribute, Ident, LitInt, Visibility};

#[proc_macro]
pub fn bitfield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let result = syn::parse(input);

    match result {
        Ok(field) => parse_bitfield(&field),
        Err(error) => panic!("Couldn't parse: {:?}!", error)
    }
}

// Parser

struct BitField {
    pub attributes: Vec<Attribute>,
    pub visibility: Option<Visibility>,
    pub name: Ident,
    pub base_type: Ident,
    pub fields: Vec<Field>
}

enum Field {
    Flag(FieldFlag),
    Type(FieldType)
}

// struct Foo2<'a> { x: &'a bool }
struct FieldFlag {
    pub attributes: Vec<Attribute>,
    pub visibility: Option<Visibility>,
    pub name: Ident,
    pub bit: LitInt
}

struct FieldType {
    pub attributes: Vec<Attribute>,
    pub visibility: Option<Visibility>,
    pub name: Ident,
    pub typ: Ident,
    pub bit: LitInt,
    pub amount: LitInt
}

impl syn::synom::Synom for BitField {
    named!(parse -> Self, do_parse!(
        attributes: many0!(Attribute::parse_outer) >>
        visibility: option!(syn!(Visibility)) >>
        name: syn!(Ident) >>
        base_type: parens!(syn!(Ident)) >>
        fields: braces!(
            // TODO: Use `synom::separated_nonempty_list!(punct!(","), ...)` instead of `syn::many0!(...)`.
            many0!(alt!(
                do_parse!(
                    attributes: many0!(Attribute::parse_outer) >>
                    visibility: option!(syn!(Visibility)) >>
                    name: syn!(Ident) >>
                    punct!(:) >>
                    bit: syn!(LitInt) >>
                    punct!(,) >>

                    (Field::Flag(FieldFlag { attributes, visibility, name, bit }))
                ) |
                do_parse!(
                    attributes: many0!(Attribute::parse_outer) >>
                    visibility: option!(syn!(Visibility)) >>
                    name: syn!(Ident) >>
                    punct!(:) >>
                    typ: syn!(Ident) >>
                    numbers: parens!(do_parse!(
                        bit: syn!(LitInt) >>
                        punct!(,) >>
                        amount: syn!(LitInt) >>

                        (bit, amount)
                    )) >>
                    punct!(,) >>

                    (Field::Type(FieldType { attributes, visibility, name, typ, bit: (numbers.1).0, amount: (numbers.1).1 }))
                )
            ))
        ) >>

        (BitField { attributes, visibility, name, base_type: base_type.1, fields: fields.1 })
    ));
}

// Converter

struct FieldBounds {
    pub name: Ident,
    pub bit: u64,
    pub amount: u64,
    pub overlap: Option<Vec<Ident>>
}

fn parse_bitfield(input: &BitField) -> proc_macro::TokenStream {
    if input.fields.len() == 0 {
        panic!("The bitfield must contain fields!");
    }

    let attributes = &input.attributes;
    let base_type = input.base_type;
    let name = input.name;
    let visibility = &input.visibility;

    let mut new_initializer = quote::Tokens::new();
    let mut initializer = quote::Tokens::new();
    let mut implementation = quote::Tokens::new();

    let mut field_bounds = Vec::new();

    // Check for duplicate names.
    for i in 0..input.fields.len() - 1 {
        let mut name_left: Ident;
        match &input.fields[i] {
            Field::Flag(flag) => {
                name_left = flag.name;
            }
            Field::Type(typ) => {
                name_left = typ.name;
            }
        }

        for j in i + 1..input.fields.len() {
            let mut name_right: Ident;
            match &input.fields[j] {
                Field::Flag(flag) => {
                    name_right = flag.name;
                }
                Field::Type(typ) => {
                    name_right = typ.name;
                }
            }

            if name_left == name_right {
                panic!("Duplicate field name: \"{}\"!", name_left);
            }
        }
    }

    // Process all fields.
    for field in input.fields.iter() {
        match field {
            Field::Flag(flag) => {
                let setter = Ident::from(format!("set_{}", flag.name.as_ref()));

                let mut attributes = flag.attributes.to_vec();
                let bit = &flag.bit;
                let name = flag.name;
                let visibility = &flag.visibility;

                let base_type_string: String = base_type.as_ref().chars().skip(1).collect();
                let available_bits = base_type_string.parse::<u64>();
                match available_bits {
                    Ok(value) => {
                        if bit.value() > value {
                            panic!("\"{}\": {} is outside of the valid range of {} bits!", name, bit.value(), value);
                        }
                    },
                    _ => panic!("\"{}\" is an unsupported primitive type!", base_type)
                }

                let overlap = extract_overlap(&mut attributes);
                let attributes = &attributes;

                field_bounds.push(FieldBounds { name, bit: bit.value(), amount: 1, overlap });

                new_initializer.append_all(quote! {
                    s.#setter(init.#name);
                });

                initializer.append_all(quote! {
                    #(#attributes)*
                    #name: bool,
                });

                implementation.append_all(quote! {
                    #(#attributes)*
                    #[inline]
                    #visibility fn #name(&self) -> bool {
                        let max_bit_value: #base_type = 1;
                        let positioned_bits = self.0 >> #bit;
                        positioned_bits & max_bit_value == 1
                    }

                    #(#attributes)*
                    #[inline]
                    #visibility fn #setter(&mut self, value: bool) {
                        let positioned_bits: #base_type = 1 << #bit;
                        let positioned_flags = (value as #base_type) << #bit;
                        let cleaned_flags = self.0 & !positioned_bits;
                        self.0 = cleaned_flags | positioned_flags;
                    }
                });
            },
            Field::Type(typ) => {
                let setter = Ident::from(format!("set_{}", typ.name.as_ref()));
                let from = Ident::from(format!("from_{}", base_type.as_ref()));

                let amount = &typ.amount;
                let mut attributes = typ.attributes.to_vec();
                let base_type = input.base_type;
                let bit = &typ.bit;
                let name = typ.name;
                let visibility = &typ.visibility;
                let typ = typ.typ;

                let base_type_string: String = base_type.as_ref().chars().skip(1).collect();
                let available_bits = base_type_string.parse::<u64>();
                match available_bits {
                    Ok(value) => {
                        if bit.value() + amount.value() > value {
                            panic!("\"{}\": {} + {} = {} is outside of the valid range of {} bits!", name, bit.value(), amount.value(), bit.value() + amount.value(), value);
                        }
                    },
                    _ => panic!("\"{}\" is an unsupported primitive type!", base_type)
                }

                let overlap = extract_overlap(&mut attributes);
                let attributes = &attributes;

                field_bounds.push(FieldBounds { name, bit: bit.value(), amount: amount.value(), overlap });

                new_initializer.append_all(quote! {
                    s.#setter(init.#name);
                });

                initializer.append_all(quote! {
                    #(#attributes)*
                    #name: #typ,
                });

                implementation.append_all(quote! {
                    #(#attributes)*
                    #[inline]
                    #visibility fn #name(&self) -> result::Result<#typ, #base_type> {
                        const MAX_BIT_VALUE: #base_type = (1 << #amount) - 1;
                        let positioned_bits = self.0 >> #bit;
                        let value = positioned_bits & MAX_BIT_VALUE;
                        let enum_value = #typ::#from(value as #base_type);
                        if enum_value.is_some() {
                            Ok(enum_value.unwrap())
                        } else { Err(value) }
                    }

                    #(#attributes)*
                    #[inline]
                    #visibility fn #setter(&mut self, value: #typ) {
                        const MAX_BIT_VALUE: #base_type = (1 << #amount) - 1;
                        const POSITIONED_BITS: #base_type = MAX_BIT_VALUE << #bit;
                        let positioned_flags = (value as #base_type) << #bit;
                        let cleaned_flags = self.0 & !POSITIONED_BITS;
                        self.0 = cleaned_flags | positioned_flags;
                    }
                });
            }
        }
    }

    // Check overlapping fields.
    let bounds = field_bounds.len();
    match bounds {
        0 => panic!("The bitfield must contain fields!"),
        1 => (),
        _ => {
            // Sort fields by 1. bit position and 2. bit amount.
            field_bounds.sort_by(|left, right| {
                let ordering = left.bit.cmp(&right.bit);
                match ordering {
                    std::cmp::Ordering::Less | std::cmp::Ordering::Greater => ordering,
                    std::cmp::Ordering::Equal => left.amount.cmp(&right.amount)
                }
            });

            // Check overlap attributes.
            for i in 0..bounds - 1 {
                let bounds_left: &FieldBounds = field_bounds.get(i).unwrap();

                for j in i + 1..bounds {
                    let bounds_right: &FieldBounds = field_bounds.get(j).unwrap();

                    match check_overlap(&bounds_left, &bounds_right) {
                        OverlapStatus::NoOverlap => break,
                        OverlapStatus::AllowedOverlap => (),
                        OverlapStatus::Overlap => panic!(format!("{} overlaps with {}! Did you forget an allow_overlap attribute?", bounds_left.name, bounds_right.name))
                    }
                }
            }
        }
    }

    // Create struct, initializer and struct implementation.
    let name_initializer = Ident::from(input.name.to_string() + "Init");

    let content: quote::Tokens = quote! {
        #(#attributes)*
        #visibility struct #name(#base_type);

        #[derive(Default)]
        #visibility struct #name_initializer {
            #initializer
        }

        impl #name {
            #[inline]
            pub fn new(init: #name_initializer) -> Self {
                let mut s = #name(0);

                #new_initializer

                s
            }

            #implementation
        }
    };

    content.into()
}

enum OverlapStatus {
    NoOverlap,
    AllowedOverlap,
    Overlap
}

/**
 * None        => No Overlap,
 * Some(false) => Overlap, with `allow_overlap` attribute.
 * Some(true)  => Overlap, without `allow_overlap` attribute!
 */
fn check_overlap(bounds_left: &FieldBounds, bounds_right: &FieldBounds) -> OverlapStatus {
    let ordering = bounds_left.bit.cmp(&bounds_right.bit);
    match ordering {
        std::cmp::Ordering::Less => {
            if bounds_left.bit + bounds_left.amount <= bounds_right.bit { return OverlapStatus::NoOverlap; }
            check_overlap_names(bounds_left, bounds_right)
        }
        std::cmp::Ordering::Equal => check_overlap_names(bounds_left, bounds_right),
        std::cmp::Ordering::Greater => panic!(format!("Wrong order of {} and {}! THIS SHOULD NEVER HAPPEN!", bounds_left.name, bounds_right.name))
    }
}

fn check_overlap_names(bounds_left: &FieldBounds, bounds_right: &FieldBounds) -> OverlapStatus {
    match bounds_left.overlap {
        None => return OverlapStatus::Overlap,
        Some(ref overlap) => {
            let mut has_attribute = false;
            for attribute_name in overlap.iter() {
                if bounds_right.name == attribute_name {
                    has_attribute = true;
                    break;
                }
            }
            if !has_attribute { return OverlapStatus::Overlap; }
        }
    }

    match bounds_right.overlap {
        None => return OverlapStatus::Overlap,
        Some(ref overlap) => {
            let mut has_attribute = false;
            for attribute_name in overlap.iter() {
                if bounds_left.name == attribute_name {
                    has_attribute = true;
                    break;
                }
            }
            if !has_attribute { return OverlapStatus::Overlap; }
        }
    }

    OverlapStatus::AllowedOverlap
}

fn extract_overlap(attributes: &mut Vec<Attribute>) -> Option<Vec<Ident>> {
    let mut result = Vec::new();

    let mut i: usize = 0;
    while i < attributes.len() {
        if attributes[i].path.segments.len() != 1 || &attributes[i].path.segments[0].ident != "allow_overlap" {
            i += 1;
            continue;
        }

        match attributes[i].interpret_meta() {
            None => panic!("Couldn't parse allow_overlap attribute!"),
            Some(meta) => {
                match meta {
                    syn::Meta::List(meta_list) => {
                        for meta_item in meta_list.nested {
                            match meta_item {
                                syn::NestedMeta::Meta(meta) => {
                                    match meta {
                                        syn::Meta::Word(ident) => {
                                            result.push(ident);
                                        },
                                        _ => panic!("Only identifiers are allowed as overlapping names!")
                                    }
                                },
                                _ => panic!("Only identifiers are allowed as overlapping names!")
                            }
                        }
                    },
                    _ => panic!("Must specify allowed overlapping names: #[allow_overlap(name1, name2, ...)]!")
                }
            }
        }
        attributes.remove(i);
    }

    if result.len() == 0 { return None; }
    Some(result)
}