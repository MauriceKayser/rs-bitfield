//! This crate provides the three macros `bitfield`, `Field` and `Flags` (and the additional
//! convenience macro `FromPrimitive`) to interoperate with low level, typically operating system
//! related types which store data with sub-byte precision, like boolean flags or sub-byte fields,
//! in a type-safe, typical rust way.
//!
//! For more specific documentation look at the documentation of the macros, or at the files in
//! `examples/*`.

/// Generates an abstraction of a primitive type which tightly stores information in a bit field.
///
/// # Example
///
/// This 32 bit wide bit field stores 4 fields (3 with a size of 4, 1 with a size of 2 bits) and a
/// few flags:
///
/// ```rust
/// /// Layout:
/// ///
/// ///  31      27      23      19      15      11      7       3     0
/// /// ╔═══════╧═══════╧═══╤═╤═╪═╤═╤═╤═╪═╤═╤═══╪═══════╪═══════╪═══════╗
/// /// ║                   │S│R│R│T│D│F│ │H│Mod│DefBtn │Icon   │Button ║
/// /// ║                   │N│T│ │M│D│ │ │ │   │       │       │       ║ Styles
/// /// ║0 0 0 0 0 0 0 0 0 0│ │L│ │ │O│ │0│ │   │       │       │       ║
/// /// ╚═══════════════════╧═╧═╧═╧═╧═╧═╧═╧═╧═══╧═══════╧═══════╧═══════╝
/// ///          Button
/// ///          Icon
/// /// DefBtn = Default Button
/// /// Mod    = Modality
/// /// H      = Help
/// /// F      = Foreground
/// /// DDO    = Default Desktop Only
/// /// TM     = Top Most
/// /// R      = Right
/// /// RTL    = Right To Left Reading
/// /// SN     = Service Notification
/// #[bitfield::bitfield(32)]
/// struct Styles {
///     #[field(size = 4)] button: Button,
///     #[field(size = 4)] icon: Icon,
///     #[field(size = 4)] default_button: DefaultButton,
///     #[field(size = 2)] modality: Modality,
///     style: Style
/// }
///
/// #[derive(Clone, Copy, bitfield::Flags)]
/// #[repr(u8)]
/// enum Style {
///     Help = 14,
///     // Bit 15 is reserved.
///     Foreground = 16,
///     DefaultDesktopOnly,
///     TopMost,
///     Right,
///     RightToLeftReading,
///     ServiceNotification
///     // Bits 22 - 31 are reserved.
/// }
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum Button {
///     Ok,
///     OkCancel,
///     AbortRetryIgnore,
///     YesNoCancel,
///     YesNo,
///     RetryCancel,
///     CancelTryContinue
///     // Bits 7 - 15 are reserved.
/// }
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum DefaultButton {
///     One,
///     Two,
///     Three,
///     Four
///     // Bits 4 - 15 are reserved.
/// }
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum Icon {
///     None,
///     Stop,
///     Question,
///     Exclamation,
///     Information
///     // Bits 5 - 15 are reserved.
/// }
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum Modality {
///     Application,
///     System,
///     Task
///     // Bit 3 is reserved.
/// }
///
/// // Construction example:
///
/// # fn main() {
/// # mod user32 {
/// #     #[allow(non_snake_case)]
/// #     pub(super) fn MessageBoxW(_: usize, _: &str, _: &str, _: super::Styles) {}
/// # }
/// # let parent = 0;
/// let styles = Styles::new()
///     .set_button(Button::YesNo)
///     .set_icon(Icon::Question)
///     .set_style(Style::Help, true)
///     .set_style(Style::TopMost, true);
///
/// // Alternatively:
/// // let styles = Styles::new() + Button::YesNo + Icon::Question + Style::Help + Style::TopMost;
///
/// // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messageboxw
/// user32::MessageBoxW(parent, "Text", "Title", styles);
/// # }
/// ```
///
/// # 1. Macro usage
///
/// The following describes how to use the macro and which options it offers.
///
/// ## 1.1. Macro attribute
///
/// The macro attribute expects the amount of bits of the primitive type to abstract from, or
/// `size` if the platform dependent `usize` should be used, f. e. to describe a CPU register, as in
/// `examples/x86_debug_registers.rs`.
///
/// Examples:
///
/// ```rust,compile_fail
/// // Abstracts access to the bits in a `u8`.
/// #[bitfield::bitfield(8)]
/// // Abstracts access to the bits in a `u32`.
/// #[bitfield::bitfield(32)]
/// // Abstracts access to the bits in a `usize`.
/// #[bitfield::bitfield(size)]
/// // Abstracts access to the bits in a `core::num::NonZeroU32`, for usage with `Option<T>`, see
/// // `examples/windows_memory_protection.rs`. Caution: All UBs from `core::num::NonZeroU32` apply
/// // as well, f. e. constructing an instance with the value `0` (`T::new()`)!
/// #[bitfield::bitfield(NonZero32)]
/// ```
///
/// If fields or flags overlap, a compile time error will occur to warn the user of this crate about
/// a possible layout mistake:
///
/// ```rust,compile_fail
/// // error: `field_1` overlaps with field `field_0`.
/// #[bitfield::bitfield(16)]
/// struct BitField {
///     #[field(bit = 0)] field_0: u8, // Field from bits 0 - 7.
///     #[field(bit = 1)] field_1: u8  // Field from bits 1 - 8.
/// }
/// ```
///
/// If this is intentional, the comma separated identifier `allow_overlaps` can be appended after
/// the amount of bits to suppress this error, as is necessary in
/// `examples/windows_memory_protection.rs`.
///
/// ```rust
/// #[bitfield::bitfield(16, allow_overlaps)]
/// struct BitField {
///     #[field(bit = 0)] field_0: u8,
///     #[field(bit = 1)] field_1: u8
/// }
/// ```
///
/// If the displayed error is `attempt to compute "0_usize - 1_usize", which would overflow` then
/// the macro itself could not check the fields and flags for overlaps and generated code so the
/// compiler can check it instead. If this happens check `tests/ui/bitfield/*` for hints.
///
/// ```rust,compile_fail
/// // error: `field`, `Flag::Flag00000001` and `Flag::Flag00000010` overlap.
/// #[bitfield::bitfield(8)]
/// struct BitField {
///     #[field(size = 2)] field: Field,
///     flags: Flag
/// }
///
/// #[derive(bitfield::Field)]
/// #[repr(u8)]
/// enum Field {
///     Variant0,
///     Variant1,
///     Variant2,
///     Variant3
/// }
///
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag00000001,
///     Flag00000010,
///     Flag00000100,
///     Flag00001000,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
/// ```
///
/// ## 1.2. Struct types
///
/// There are two different ways to define a bit field with the `bitfield::bitfield` macro.
///
/// ### 1.2.1 Tuple structs
///
/// If a bit field only contains one field or only flags, a tuple struct can be used.
///
/// #### 1.2.1.1 Flags
///
/// Example (using the `bitfield::Flags` macro):
///
/// ```rust
/// /// Layout:
/// ///
/// ///  7       3     0
/// /// ╔═══════╧═══════╗
/// /// ║-----Flags-----║
/// /// ║               ║ BitField
/// /// ║               ║
/// /// ╚═══════════════╝
/// #[bitfield::bitfield(8)]
/// struct Flags(Flag);
///
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag00000001,
///     Flag00000010,
///     Flag00000100,
///     Flag00001000,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
/// ```
///
/// The names for the accessors described below are `has` and `set`.
///
/// #### 1.2.1.2 Field
///
/// Example (using the `bitfield::Field` macro):
///
/// ```rust
/// /// Layout:
/// ///
/// ///  7       3     0
/// /// ╔═════╤═╧═╤═════╗
/// /// ║     │Fld│     ║
/// /// ║     │   │     ║ BitField
/// /// ║0 0 0│   │0 0 0║
/// /// ╚═════╧═══╧═════╝
/// /// Fld = Field
/// #[bitfield::bitfield(8)]
/// struct BitField(#[field(3, 2)] Field);
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum Field {
///     Variant0,
///     Variant1,
///     Variant2,
///     Variant3
/// }
/// ```
///
/// The names for the accessors described below are `get` and `set`.
///
/// ### 1.2.2 Struct with named fields
///
/// If a bit field contains more than one field or a combination of fields and flags (or multiple
/// flag types if one is reused in multiple types for example, see
/// `example/windows_object_access.rs`), a struct with named fields must be used.
///
/// Example:
///
/// ```rust
/// /// Layout:
/// ///
/// ///  7       3     0
/// /// ╔═══════╧═╤═════╗
/// /// ║--Flags--│Field║
/// /// ║         │     ║ BitField
/// /// ║         │     ║
/// /// ╚═════════╧═════╝
/// #[bitfield::bitfield(8)]
/// struct BitField {
///     #[field(size = 3)] field: Field,
///     flags: Flag
/// }
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum Field {
///     Variant0,
///     Variant1,
///     Variant2,
///     Variant3,
///     Variant4,
///     Variant5,
///     Variant6,
///     Variant7
/// }
///
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag00001000 = 3,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
/// ```
///
/// The names for the accessors described below are `field` and `set_field` as well as `flags` and
/// `set_flags`.
///
/// ## 1.3. Bit field entries
///
/// As seen above, a bit field can host a combination of multi-bit wide fields and one-bit wide
/// boolean flags.
///
/// ### 1.3.1 Flags
///
/// Flags in a bit field must be `#[repr(u8)]` `enum` types. The `bitfield::Flags` proc-macro-derive
/// macro aids in implementing the necessary traits and methods. Unlike fields, flags can be used in
/// bit fields without any special attribute, just like a field in a normal `struct` type.
///
/// Example:
///
/// ```rust
/// /// Layout:
/// ///
/// ///  7       3     0
/// /// ╔═══════╧═══════╗
/// /// ║-----Flags-----║
/// /// ║               ║ BitField
/// /// ║               ║
/// /// ╚═══════════════╝
/// #[bitfield::bitfield(8)]
/// struct BitField(
///     // No explicit attribute needed.
///     Flag
/// );
///
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag00000001,
///     Flag00000010,
///     Flag00000100,
///     Flag00001000,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
/// ```
///
/// ### 1.3.2 Fields
///
/// A field in a bit field must be either a C-like `enum` type with a `#[repr(iX/uX)]` attribute, a
/// primitive integer type, or a `bool`. A `bool` should only be used in case a separate
/// flags enum is unnecessary, see `examples/vga_text_mode.rs`, otherwise using a flags enum should
/// be preferred, as it has advantages like enumerability. For C-like `enum` types, the
/// `bitfield::Field` proc-macro-derive macro aids in implementing the necessary traits and methods.
///
/// Unlike for flags, a `#[field]` attribute must be specified for fields in a bit field.
///
/// Example:
///
/// ```rust
/// /// Layout:
/// ///
/// ///  7       3     0
/// /// ╔═════╤═╧═╤═════╗
/// /// ║     │Fld│     ║
/// /// ║     │   │     ║ BitField
/// /// ║0 0 0│   │0 0 0║
/// /// ╚═════╧═══╧═════╝
/// /// Fld = Field
/// #[bitfield::bitfield(8)]
/// struct BitField(
///     // Explicit attribute needed.
///     #[field(3, 2)] Field
/// );
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum Field {
///     Variant0,
///     Variant1,
///     Variant2,
///     Variant3
/// }
/// ```
///
/// The position and the size of the field can be specified separately by using one of the following
/// notations:
///
/// - Position: `#[field(bit = VALUE)]` (see `field_01` in the following example)
/// - Size: `#[field(size = VALUE)]` (see `field_09`)
/// - Both: `#[field(BIT, SIZE)]` (see `field_14`)
///
/// If `bit` is not specified in any way, then the field is placed directly after the previous field
/// (deduced by its `bit + size`), see `field_09`.
///
/// If `size` is not specified in any way for fields of an unsigned primitive integer type or
/// `bool`, the full amount of necessary bits store the primitive type is used instead, see
/// `field_01`.
///
/// The `#[field]` attribute can be completely omitted for primitive types in the case that no
/// explicit position should be set and the full primitive type size should be used (see
/// `field_12`), as is done for the two `bool` fields in `examples/vga_text_mode.rs`.
///
/// Example:
///
/// ```rust
/// /// Layout:
/// ///
/// ///  15      11      7       3     0
/// /// ╔═══╤═╤═╪═════╤═╧═══════╧═════╤═╗
/// /// ║f14│ │f│fld09│----field_01---│ ║
/// /// ║   │ │1│     │               │ ║ BitField
/// /// ║   │0│2│     │               │0║
/// /// ╚═══╧═╧═╧═════╧═══════════════╧═╝
/// ///         field_01
/// /// fld09 = field_09
/// /// f12   = field_12
/// /// f14   = field_14
/// #[bitfield::bitfield(16)]
/// struct BitField {
///     // Bit 0 is unused.
///     #[field(bit  = 1)] field_01: u8,   // Field from bit  1 -  8 (implicit size =  8).
///     #[field(size = 3)] field_09: u8,   // Field from bit  9 - 11 (implicit bit  =  9).
///                        field_12: bool, // Implicit field 12 - 12 (implicit bit  = 12, size = 1).
///     // Bit 13 is unused.
///     #[field(14, 2)]    field_14: Field // Field from bit 14 - 15 (explicit bit and size).
/// }
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum Field {
///     Variant0,
///     Variant1,
///     Variant2,
///     Variant3
/// }
/// ```
///
/// ## 1.4. Implementations for the `core::fmt::{Debug, Display}` traits
///
/// Implementations for the `core::fmt::{Debug, Display}` traits can be generated by using the
/// `#[derive]` attribute. In this case the user of this crate needs to add `extern crate alloc;` to
/// the root of their crate, as the macros in this crate target `core` instead of `std` to generate
/// `#![no_std]`-compatible code, and formatting strings requires memory allocations.
///
/// Example:
///
/// ```rust
/// extern crate alloc;
///
/// #[bitfield::bitfield(8)]
/// #[derive(Debug)]
/// struct BitField {
///     #[field(size = 2)] field: Field,
///     flags: Flag
/// }
///
/// #[derive(Clone, Copy, Debug, bitfield::Field)]
/// #[repr(u8)]
/// enum Field {
///     Variant0,
///     Variant1,
///     Variant2,
///     Variant3
/// }
///
/// #[derive(Copy, Clone, Debug, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag00000100 = 2,
///     Flag00001000,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
/// ```
///
/// The implementation for `core::fmt::Display` can only be generated for bit fields with only one
/// field or flags (typically a tuple struct bit field).
///
/// Example for flags:
///
/// ```rust
/// extern crate alloc;
///
/// #[bitfield::bitfield(8)]
/// #[derive(Debug, Display)]
/// struct BitField(Flag);
///
/// #[derive(Copy, Clone, Debug, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag00000001,
///     Flag00000010,
///     Flag00000100,
///     Flag00001000,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
/// ```
///
/// Example for a field:
///
/// ```rust
/// extern crate alloc;
///
/// #[bitfield::bitfield(8)]
/// #[derive(Debug, Display)]
/// struct BitField(#[field(3, 2)] u8);
/// ```
///
/// # 2. Bit field features
///
/// The following type definition is generated for a bit field:
///
/// ```ignore
/// #[repr(transparent)]
/// struct #NAME(#PRIMITIVE_TYPE);
/// ```
///
/// ## 2.1. Initialization
///
/// For initialization purposes the following method is generated:
///
/// ### 2.1.1 Primitive type based bit field
///
/// ```ignore
/// /// Creates a new instance with all flags and fields cleared.
/// const fn new() -> Self;
/// ```
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(8)]
/// struct BitField(bool);
///
/// let field = BitField::new();
/// ```
///
/// ### 2.1.2 `NonZero` type based bit field
///
/// `NonZero` type based bit fields can not safely be initialized from `0`, so no `new` method is generated.
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(NonZero8)]
/// struct BitField(bool);
/// ```
///
/// ## 2.2. Accessors
///
/// All methods that change the state of a bit field do not actually change the bit field, but
/// return a changed copy of it, in a way that it can be constructed with the
/// [builder pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html). To access
/// fields and flags the following methods are generated, with the exception that for `NonZero`
/// type based bit fields, the methods that return `Self` in the examples below return `Option<Self>`
/// instead, when the internal value becomes `0`:
///
/// ### 2.2.1 Flags
///
/// For flags the following accessor methods are generated:
///
/// ```ignore
/// /// Returns `true` if the specified `flag` is set.
/// const fn #GETTER(&self, flag: #FLAG_TYPE) -> bool;
///
/// /// Returns `true` if all flags are set.
/// const fn #GETTER_all(&self) -> bool;
///
/// /// Returns `true` if any flag is set.
/// const fn #GETTER_any(&self) -> bool;
///
/// /// Creates a copy of the bit field with the new value for the specified flag.
/// const fn #SETTER(&self, flag: #FLAG_TYPE, value: bool) -> Self;
///
/// /// Creates a copy of the bit field with all flags set to `true`.
/// const fn #SETTER_all(&self) -> Self;
///
/// /// Creates a copy of the bit field with all flags set to `false`.
/// const fn #SETTER_none(&self) -> Self;
///
/// /// Creates a copy of the bit field with the value of the specified flag inverted.
/// const fn invert_#FLAG(&self, flag: #FLAG_TYPE) -> Self;
/// ```
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(8)]
/// struct BitField(Flag);
///
/// #[derive(Copy, Clone, Debug, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag00000001,
///     Flag00000010,
///     Flag00000100,
///     Flag00001000,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
///
/// let mut field = BitField::new();
/// assert!(!field.has(Flag::Flag00000001));
/// assert!(!field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!(!field.has_any());
///
/// field = field.set(Flag::Flag00000001, true).set(Flag::Flag00000010, true);
/// assert!( field.has(Flag::Flag00000001));
/// assert!( field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!( field.has_any());
///
/// field = field.set(Flag::Flag00000001, false);
/// assert!(!field.has(Flag::Flag00000001));
/// assert!( field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!( field.has_any());
///
/// field = field.set_all();
/// assert!( field.has(Flag::Flag00000001));
/// assert!( field.has(Flag::Flag00000010));
/// assert!( field.has(Flag::Flag00000100));
/// assert!( field.has_all());
/// assert!( field.has_any());
///
/// field = field.set_none();
/// assert!(!field.has(Flag::Flag00000001));
/// assert!(!field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!(!field.has_any());
///
/// field = field.invert(Flag::Flag00000001);
/// assert!( field.has(Flag::Flag00000001));
/// assert!(!field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!( field.has_any());
///
/// field = field.invert(Flag::Flag00000001);
/// assert!(!field.has(Flag::Flag00000001));
/// assert!(!field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!(!field.has_any());
/// ```
///
/// The `#SETTER_none` method is not generated at all, if a `NonZero` type based bit field has only one entry, as this would guarantee to return `None`.
///
/// ### 2.2.2 Fields
///
/// For fields the following accessor methods are generated:
///
/// #### 2.2.2.1 `bool`
///
/// For `bool` fields the following accessor methods are generated:
///
/// ```rust,ignore
/// /// Gets the value of the field.
/// const fn #GETTER(&self) -> bool;
///
/// /// Creates a copy of the bit field with the new value.
/// const fn #SETTER(&self, value: bool) -> Self;
///
/// /// Creates a copy of the bit field with the value of the field inverted.
/// const fn invert_#FIELD(&self, flag: #FLAG_TYPE) -> Self;
/// ```
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(8)]
/// struct BitField(bool);
///
/// let mut field = BitField::new();
/// assert!(!field.get());
///
/// field = field.set(true);
/// assert!( field.get());
///
/// field = field.set(false);
/// assert!(!field.get());
///
/// field = field.invert();
/// assert!( field.get());
///
/// field = field.invert();
/// assert!(!field.get());
/// ```
///
/// #### 2.2.2.2 Signed primitive integer types
///
/// Because the highest bit is used to store the sign, signed primitive integer types are only
/// supported if their full size is used, f. e. `size = 8` for `i8` or `size = 16` for `i16`, etc.
///
/// For signed primitive integer type fields the following accessor methods are generated:
///
/// ```rust,ignore
/// /// Gets the value of the field.
/// const fn #GETTER(&self) -> #PRIMITIVE_TYPE;
///
/// /// Creates a copy of the bit field with the new value.
/// const fn #SETTER(&self, value: #PRIMITIVE_TYPE) -> Self;
/// ```
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(16)]
/// struct BitField(#[field(bit = 8)] i8);
///
/// let mut field = BitField::new();
/// assert_eq!(field.get(), 0);
///
/// field = field.set(7);
/// assert_eq!(field.get(), 7);
///
/// field = field.set(-7);
/// assert_eq!(field.get(), -7);
/// ```
///
/// #### 2.2.2.3 Unsigned primitive integer types
///
/// For unsigned primitive integer type fields which are smaller than their full size, the following
/// accessor methods are generated:
///
/// ```rust,ignore
/// /// Gets the value of the field.
/// const fn #GETTER(&self) -> #PRIMITIVE_TYPE;
///
/// // NOTE: This can be solved with ranged integers when they land:
/// // https://github.com/rust-lang/rfcs/issues/671.
/// //
/// /// Creates a copy of the bit field with the new value.
/// ///
/// /// Returns `None` if `value` is bigger than the specified amount of
/// /// bits for the field can store.
/// const fn #SETTER(&self, value: #PRIMITIVE_TYPE) -> core::option::Option<Self>;
/// ```
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(8)]
/// struct BitField(#[field(size = 3)] u8);
///
/// let mut field = BitField::new();
/// assert_eq!(field.get(), 0);
///
/// field = field.set(7).unwrap();
/// assert_eq!(field.get(), 7);
///
/// assert!(field.set(8).is_none());
/// ```
///
/// A "signed primitive integer types"-like implementation is generated, which does not return a
/// `core::result::Result` from the setter, if the full size of the integer is used:
///
/// ```rust,ignore
/// /// Gets the value of the field.
/// const fn #GETTER(&self) -> #PRIMITIVE_TYPE;
///
/// /// Creates a copy of the bit field with the new value.
/// const fn #SETTER(&self, value: #PRIMITIVE_TYPE) -> Self; // No `Option<Self>` here.
/// ```
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(16)]
/// struct BitField(#[field(size = 8)] u8);
///
/// let mut field = BitField::new();
/// assert_eq!(field.get(), 0);
///
/// field = field.set(8);
/// assert_eq!(field.get(), 8);
/// ```
///
/// #### 2.2.2.4 Unsigned enumerations
///
/// For fields of enumerations with an unsigned primitive integer representation, the following
/// accessor methods are generated:
///
/// ```rust,ignore
/// // NOTE: This method is `const` only if the `#![feature(const_trait_impl)]` from
/// // https://github.com/rust-lang/rust/pull/68847 is used because `core::convert::TryFrom` is
/// // used under the hood to convert the primitve value to an enumeration variant.
/// //
/// /// Returns the primitive value encapsulated in the `Err` variant, if the value can
/// /// not be converted to the expected type.
/// (const) fn #GETTER(&self) -> core::result::Result<#FIELD_TYPE, #UNSIGNED_PRIMITIVE_TYPE>;
///
/// /// Creates a copy of the bit field with the new value.
/// const fn #SETTER(&self, value: #FIELD_TYPE) -> Self;
/// ```
///
/// The getter tries to convert the primitive integer type to an enumeration variant by executing
/// `core::convert::TryFrom<#UNSIGNED_PRIMITIVE_TYPE>::try_into(BITS_REPRESENTING_THE_FIELD)` where
/// `#UNSIGNED_PRIMITIVE_TYPE` is the smallest possible primitive integer type that can store the
/// field value, based on the `size` value in the `#[field]` attribute, f. e. `u8` for
/// `#[field(size = 1)]` to `#[field(size = 8)]`, or `u16` for `#[field(size = 9)]` to
/// `#[field(size = 16)]`, etc.
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(8)]
/// struct BitField(#[field(size = 3)] Field);
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, bitfield::Field)]
/// #[repr(u8)]
/// enum Field {
///     // 0 is unused.
///     One = 1,
///     Two,
///     Three
/// }
///
/// let mut field = BitField::new();
/// assert_eq!(field.get(), Err(0));
///
/// field = field.set(Field::One);
/// assert_eq!(field.get(), Ok(Field::One));
/// ```
///
/// #### 2.2.2.5 Signed enumerations
///
/// Because the highest bit is used to store the sign, fields of enumerations with a signed
/// primitive integer representation are only supported if their full size is used, f. e. `size = 8`
/// for `i8` or `size = 16` for `i16`, etc.
///
/// They are generally handled like fields of enumerations with an unsigned primitive integer type,
/// except that the `signed` keyword must be added in the `#[field]` attribute, which will cause the
/// following accessor methods to be generated:
///
/// ```rust,ignore
/// // NOTE: This method is `const` only if the `#![feature(const_trait_impl)]` from
/// // https://github.com/rust-lang/rust/pull/68847 is used because `core::convert::TryFrom` is
/// // used under the hood to convert the primitve value to an enumeration variant.
/// //
/// /// Returns the primitive value encapsulated in the `Err` variant, if the value can
/// /// not be converted to the expected type.
/// (const) fn #GETTER(&self) -> core::result::Result<#FIELD_TYPE, #SIGNED_PRIMITIVE_TYPE>;
///
/// /// Creates a copy of the bit field with the new value.
/// const fn #SETTER(&self, value: #FIELD_TYPE) -> Self;
/// ```
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(16)]
/// struct BitField(#[field(size = 8, signed)] Field);
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, bitfield::Field)]
/// #[repr(i8)]
/// enum Field {
///     MinusOne = -1,
///     // 0 is unused.
///     One = 1,
///     Two,
///     Three
/// }
///
/// let mut field = BitField::new();
/// assert_eq!(field.get(), Err(0));
///
/// field = field.set(Field::MinusOne);
/// assert_eq!(field.get(), Ok(Field::MinusOne));
///
/// field = field.set(Field::One);
/// assert_eq!(field.get(), Ok(Field::One));
/// ```
///
/// #### 2.2.2.6 Complete enumerations
///
/// As described previously, the getter of an enumeration value returns a
/// `core::result::Result<#FIELD_TYPE, #PRIMITIVE_TYPE>` by default. The `Result`
/// overhead can be avoided for enumerations which have a variant for all bit
/// combinations of a field.
///
/// Example:
///
/// ```rust
/// #[bitfield::bitfield(8)]
/// struct BitField {
///     #[field(size = 2)]           field_normal:     FieldComplete,
///     #[field(size = 2, complete)] field_complete:   FieldComplete,
///     #[field(size = 2)]           field_incomplete: FieldIncomplete,
/// //  #[field(size = 2, complete)] field_invalid:    FieldIncomplete,
/// }
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, bitfield::Field)]
/// #[repr(u8)]
/// enum FieldComplete {
///     Zero, // 0b00
///     One,  // 0b01
///     Two,  // 0b10
///     Three // 0b11
/// }
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, bitfield::Field)]
/// #[repr(u8)]
/// enum FieldIncomplete {
///     // 0 is unused.
///     One = 1, // 0b01
///     Two,     // 0b10
///     Three    // 0b11
/// }
///
/// let field = BitField::new();
/// assert_eq!(field.field_normal(),      Ok(FieldComplete::Zero));
/// assert_eq!(field.field_complete(),       FieldComplete::Zero );
/// assert_eq!(field.field_incomplete(), Err(         0         ));
/// ```
///
/// ### 2.2.3 `core::ops::*` implementations
///
/// Bit fields can be manipulated in a less verbose way than previously presented. For most fields
/// and flags, a few `core::ops::*` trait implementations are generated.
///
/// *Note*: These shortcuts can not yet be used in a `const` context until traits can be implemented
/// in a `const` way, see [RFC-2632](https://github.com/rust-lang/rfcs/pull/2632).
///
/// #### 2.2.3.1 Flags
///
/// For flags the `core::ops::*` implementations are *not* generated under these conditions:
/// - The flags are less visible than the bit field. Trait implementations of a `pub` bit field are
/// `pub` themselves and would leak access to less visible flags.
///
///     Negative example:
///
///     ```rust
///     #[bitfield::bitfield(8)]
///     pub struct BitField(pub(crate) Flag);
///     #
///     # #[derive(Copy, Clone, Debug, bitfield::Flags)]
///     # #[repr(u8)]
///     # enum Flag {
///     #    Flag00000001
///     # }
///     ```
/// - The primitive type is a `NonZero` variant. The explicit setter accessors return `Option<Self>`,
/// which is not possible for the trait implementations.
///
///     Negative example:
///
///     ```rust
///     #[bitfield::bitfield(NonZero8)]
///     struct BitField(Flag);
///     #
///     # #[derive(Copy, Clone, Debug, bitfield::Flags)]
///     # #[repr(u8)]
///     # enum Flag {
///     #    Flag00000001
///     # }
///     ```
///
/// If the right conditions are met, the following `core::ops::*` implementations are generated:
///
/// ```rust,ignore
/// core::ops::Add<#FLAG_TYPE>;
/// core::ops::AddAssign<#FLAG_TYPE>;
/// core::ops::BitXor<#FLAG_TYPE>;
/// core::ops::BitXorAssign<#FLAG_TYPE>;
/// core::ops::Sub<#FLAG_TYPE>;
/// core::ops::SubAssign<#FLAG_TYPE>;
/// ```
///
/// This allows flags to be enabled (`+`), disabled (`-`) and inverted (`^`) in the following ways:
///
/// ```rust
/// #[bitfield::bitfield(8)]
/// struct BitField(Flag);
///
/// #[derive(Copy, Clone, Debug, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag00000001,
///     Flag00000010,
///     Flag00000100,
///     Flag00001000,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
///
/// let mut field = BitField::new() + Flag::Flag00000001 ^ Flag::Flag00000010;
/// assert!( field.has(Flag::Flag00000001));
/// assert!( field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!( field.has_any());
///
/// field = field - Flag::Flag00000001;
/// field = field ^ Flag::Flag00000010;
/// assert!(!field.has(Flag::Flag00000001));
/// assert!(!field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!(!field.has_any());
///
/// field += Flag::Flag00000001;
/// field ^= Flag::Flag00000010;
/// assert!( field.has(Flag::Flag00000001));
/// assert!( field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!( field.has_any());
///
/// field = field - Flag::Flag00000001 ^ Flag::Flag00000010;
/// assert!(!field.has(Flag::Flag00000001));
/// assert!(!field.has(Flag::Flag00000010));
/// assert!(!field.has(Flag::Flag00000100));
/// assert!(!field.has_all());
/// assert!(!field.has_any());
/// ```
///
/// #### 2.2.3.2 Fields
///
/// For fields the `core::ops::*` implementations are *not* generated under these conditions:
/// - The field is less visible than the bit field. Trait implementations of a `pub` bit field are
/// `pub` themselves and would leak access to a less visible field.
///
///     Negative example:
///
///     ```rust
///     #[bitfield::bitfield(8)]
///     pub struct BitField(#[field(size = 3)] pub(crate) Field);
///     #
///     # #[derive(Copy, Clone, bitfield::Field)]
///     # #[repr(u8)]
///     # enum Field {
///     #    Zero
///     # }
///     ```
/// - The primitive type is a `NonZero` variant. The explicit setter accessors return `Option<Self>`,
/// which is not possible for the trait implementations.
///
///     Negative example:
///
///     ```rust
///     #[bitfield::bitfield(NonZero8)]
///     struct BitField(#[field(size = 3)] Field);
///     #
///     # #[derive(Copy, Clone, bitfield::Field)]
///     # #[repr(u8)]
///     # enum Field {
///     #    Zero
///     # }
///     ```
/// - The type of the field is used more than once in the bit field. The implementation can not know
/// which field to access.
///
///     Negative example:
///
///     ```rust
///     #[bitfield::bitfield(8)]
///     struct BitField {
///         #[field(size = 3)] f1: Field,
///         #[field(size = 3)] f2: Field
///     }
///     #
///     # #[derive(Clone, Copy, bitfield::Field)]
///     # #[repr(u8)]
///     # enum Field {
///     #     Zero
///     # }
///     ```
///
/// If the right conditions are met, the following `core::ops::*` implementations are generated:
///
/// ```rust,ignore
/// core::ops::Add<#FIELD_TYPE>;
/// core::ops::AddAssign<#FIELD_TYPE>;
/// ```
///
/// This allows the field value to be set (`+`, analogous to how flags are activated):
///
/// ```rust
/// #[bitfield::bitfield(8)]
/// struct BitField(#[field(size = 3)] Field);
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, bitfield::Field)]
/// #[repr(u8)]
/// enum Field {
///     // 0 is unused.
///     One = 1,
///     Two,
///     Three
/// }
///
/// let mut field = BitField::new() + Field::One;
/// assert_eq!(field.get(), Ok(Field::One));
///
/// field = field.set(Field::Two);
/// assert_eq!(field.get(), Ok(Field::Two));
///
/// field = field + Field::Three;
/// assert_eq!(field.get(), Ok(Field::Three));
///
/// field += Field::One;
/// assert_eq!(field.get(), Ok(Field::One));
/// ```
#[proc_macro_attribute]
pub fn bitfield(
    attribute: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    bitfield_impl::bitfield::BitField::parse(attribute.into(), item.into()).map_or_else(
        |error| error.to_compile_error(),
        |field| field.into()
    ).into()
}

/// Generates all necessary trait implementations and methods for a C-like `enum` type to be used as
/// a field in the `bitfield::bitfield` macro.
///
/// The type must implement `core::clone::Clone` and `core::marker::Copy`.
///
/// The following methods are generated:
///
/// ```ignore
/// /// Returns true if the enumeration is represented by a signed primitive type.
/// const fn is_signed() -> bool;
///
/// /// Returns an array containing all enumeration variants in the defined order.
/// const fn iter() -> &'static [Self];
/// ```
///
/// A `core::convert::TryFrom<#REPR_TYPE>` implementation with `Error = #REPR_TYPE` is generated.
///
/// Example:
///
/// ```rust
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(u8)]
/// enum UnsignedField {
///     Variant1 = 1,
///     Variant2,
///     Variant5 = 5
/// }
///
/// #[derive(Clone, Copy, bitfield::Field)]
/// #[repr(i8)]
/// enum SignedField {
///     VariantMinus1 = -1,
///     Variant1 = 1
/// }
/// ```
///
/// Generates:
///
/// ```rust
/// # #[derive(Clone, Copy)]
/// # #[repr(u8)]
/// # enum UnsignedField {
/// #     Variant1 = 1,
/// #     Variant2,
/// #     Variant5 = 5
/// # }
/// # #[derive(Clone, Copy)]
/// # #[repr(i8)]
/// # enum SignedField {
/// #     VariantMinus1 = -1,
/// #     Variant1 = 1
/// # }
/// impl UnsignedField {
///     /// Returns true if the enumeration is represented by a signed primitive type.
///     #[inline(always)]
///     const fn is_signed() -> bool {
///         false
///     }
///
///     /// Returns an array containing all enumeration variants in the defined order.
///     #[inline(always)]
///     const fn iter() -> &'static [Self] {
///         &[Self::Variant1, Self::Variant2, Self::Variant5]
///     }
/// }
///
/// impl ::core::convert::TryFrom<u8> for UnsignedField {
///     type Error = u8;
///
///     #[allow(non_upper_case_globals)]
///     #[inline(always)]
///     fn try_from(value: u8) -> ::core::result::Result<
///         Self, <Self as ::core::convert::TryFrom<u8>>::Error
///     > {
///         const Variant1: u8 = UnsignedField::Variant1 as u8;
///         const Variant2: u8 = UnsignedField::Variant2 as u8;
///         const Variant5: u8 = UnsignedField::Variant5 as u8;
///
///         match value {
///             Variant1 | Variant2 | Variant5 => ::core::result::Result::Ok(unsafe {
///                 *(&value as *const u8 as *const Self)
///             }),
///             _ => ::core::result::Result::Err(value)
///         }
///     }
/// }
///
/// impl SignedField {
///     /// Returns true if the enumeration is represented by a signed primitive type.
///     #[inline(always)]
///     const fn is_signed() -> bool {
///         true
///     }
///
///     /// Returns an array containing all enumeration variants in the defined order.
///     #[inline(always)]
///     const fn iter() -> &'static [Self] {
///         &[Self::VariantMinus1, Self::Variant1]
///     }
/// }
///
/// impl ::core::convert::TryFrom<i8> for SignedField {
///     type Error = i8;
///
///     #[allow(non_upper_case_globals)]
///     #[inline(always)]
///     fn try_from(value: i8) -> ::core::result::Result<
///         Self, <Self as ::core::convert::TryFrom<i8>>::Error
///     > {
///         const VariantMinus1: i8 = SignedField::VariantMinus1 as i8;
///         const Variant1: i8 = SignedField::Variant1 as i8;
///
///         match value {
///             VariantMinus1 | Variant1 => ::core::result::Result::Ok(unsafe {
///                 *(&value as *const i8 as *const Self)
///             }),
///             _ => ::core::result::Result::Err(value)
///         }
///     }
/// }
/// ```
#[proc_macro_derive(Field)]
pub fn field(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bitfield_impl::field::Field::parse(item.into())
        .map(|field| field.into())
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

/// Generates all necessary trait implementations and methods for a `#[repr(u8)]` `enum` type to be
/// used as flags in the `bitfield::bitfield` macro.
///
/// The type must implement `core::clone::Clone` and `core::marker::Copy`.
///
/// The enum variant discriminators must be equal to the bit position the flag is supposed to be at,
/// not the actual shifted flag:
///
/// ```rust
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum CorrectExplicit {
///     Flag00000001 = 0,
///     Flag00000010 = 1,
///     Flag00000100 = 2,
///     Flag00001000 = 3,
///     Flag00010000 = 4,
///     Flag00100000 = 5,
///     Flag01000000 = 6,
///     Flag10000000 = 7
/// }
///
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum CorrectImplicit {
///     Flag00000001,
///     Flag00000010,
///     Flag00000100,
///     Flag00001000,
///     Flag00010000,
///     Flag00100000,
///     Flag01000000,
///     Flag10000000
/// }
///
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum IncorrectBinary {
///     Flag00000001 = 0b00000001,
///     Flag00000010 = 0b00000010,
///     Flag00000100 = 0b00000100,
///     Flag00001000 = 0b00001000,
///     Flag00010000 = 0b00010000,
///     Flag00100000 = 0b00100000,
///     Flag01000000 = 0b01000000,
///     Flag10000000 = 0b10000000
/// }
///
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum IncorrectHex {
///     Flag00000001 = 0x01,
///     Flag00000010 = 0x02,
///     Flag00000100 = 0x04,
///     Flag00001000 = 0x08,
///     Flag00010000 = 0x10,
///     Flag00100000 = 0x20,
///     Flag01000000 = 0x40,
///     Flag10000000 = 0x80
/// }
///
/// #[derive(Copy, Clone, bitfield::Flags)]
/// #[repr(u8)]
/// enum IncorrectShift {
///     Flag00000001 = 1 << 0,
///     Flag00000010 = 1 << 1,
///     Flag00000100 = 1 << 2,
///     Flag00001000 = 1 << 3,
///     Flag00010000 = 1 << 4,
///     Flag00100000 = 1 << 5,
///     Flag01000000 = 1 << 6,
///     Flag10000000 = 1 << 7
/// }
/// ```
///
/// The following methods are generated:
///
/// ```ignore
/// /// Returns an array containing all enumeration variants in the defined order.
/// const fn iter() -> &'static [Self];
///
/// /// Returns the flag with the highest bit value.
/// const fn max() -> Self;
/// ```
///
/// Example:
///
/// ```rust
/// #[derive(Clone, Copy, bitfield::Flags)]
/// #[repr(u8)]
/// enum Flag {
///     Flag1 = 1,
///     Flag2,
///     Flag5 = 5
/// }
/// ```
///
/// Generates:
///
/// ```rust
/// # #[derive(Clone, Copy)]
/// # #[repr(u8)]
/// # enum Flag {
/// #     Flag1 = 1,
/// #     Flag2,
/// #     Flag5 = 5
/// # }
/// #
/// impl Flag {
///     /// Returns an array containing all enumeration variants in the defined order.
///     #[inline(always)]
///     const fn iter() -> &'static [Self] {
///         &[Self::Flag1, Self::Flag2, Self::Flag5]
///     }
///
///     /// Returns the flag with the highest bit value.
///     #[inline(always)]
///     const fn max() -> Self {
///         let mut i = 0;
///         let mut max = Self::Flag1;
///
///         while i < Self::iter().len() {
///             let current = Self::iter()[i];
///             if current as u8 > max as u8 {
///                 max = current;
///             }
///
///             i += 1;
///         }
///
///         max
///     }
/// }
/// ```
#[proc_macro_derive(Flags)]
pub fn flags(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bitfield_impl::flags::Flags::parse(item.into())
        .map(|flags| flags.into())
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

/// Usage of this macro is not necessary to create a bit field, it is only exposed for convenience
/// for crates which need a way to convert primitive integer types to enum variants, similar to how
/// enum fields are converted in bit fields generated by this crate.
///
/// Generates a `core::convert::TryFrom<#REPR_TYPE>` implementation with `Error = #REPR_TYPE` for a
/// C-like `enum`.
///
/// The type must implement `core::clone::Clone` and `core::marker::Copy`.
///
/// Example:
///
/// ```rust
/// #[derive(Clone, Copy, bitfield::FromPrimitive)]
/// #[repr(u8)]
/// enum Field {
///     Variant1 = 1,
///     Variant2,
///     Variant5 = 5
/// }
/// ```
///
/// Generates:
///
/// ```rust
/// # #[derive(Clone, Copy)]
/// # #[repr(u8)]
/// # enum Field {
/// #     Variant1 = 1,
/// #     Variant2,
/// #     Variant5 = 5
/// # }
/// impl ::core::convert::TryFrom<u8> for Field {
///     type Error = u8;
///
///     #[allow(non_upper_case_globals)]
///     #[inline(always)]
///     fn try_from(value: u8) -> ::core::result::Result<
///         Self, <Self as ::core::convert::TryFrom<u8>>::Error
///     > {
///         const Variant1: u8 = Field::Variant1 as u8;
///         const Variant2: u8 = Field::Variant2 as u8;
///         const Variant5: u8 = Field::Variant5 as u8;
///
///         match value {
///             Variant1 | Variant2 | Variant5 => ::core::result::Result::Ok(unsafe {
///                 *(&value as *const u8 as *const Self)
///             }),
///             _ => ::core::result::Result::Err(value)
///         }
///     }
/// }
/// ```
#[proc_macro_derive(FromPrimitive)]
pub fn from_primitive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bitfield_impl::enumeration::Enumeration::parse(item.into())
        .map(|enumeration| enumeration.generate_try_from())
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}