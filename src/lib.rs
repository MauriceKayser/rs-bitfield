#![no_std]
#![feature(plugin)]
#![plugin(interpolate_idents)]

pub extern crate num;

/**
 * # Bitfields for Rust
 * 
 * A Rust macro to generate structures which behave like bitfields.
 * 
 * ## Dependencies
 * 
 * - Rust's [nightly distribution](https://doc.rust-lang.org/book/nightly-rust.html) because of: [interpolate_idents](https://crates.io/crates/interpolate_idents).
 * - Rust's [core crate](https://doc.rust-lang.org/core/index.html) because of: [core::mem::transmute_copy](https://doc.rust-lang.org/core/mem/fn.transmute_copy.html) for primitive to enum conversion, and [core::default::Default](https://doc.rust-lang.org/beta/core/default/trait.Default.html).
 * - [num](https://rust-num.github.io/num/num/index.html) crate because of: [num::traits::FromPrimitive](https://rust-num.github.io/num/num/trait.FromPrimitive.html) conversion traits.
 * 
 * ## Simple example
 * 
 * Imagine the following type which can store up to 8 flags in a `u8` value:
 * 
 * ```rust
 * const IS_VISIBLE:    u8 = 1 << 0; // 1
 * const IS_INVINCIBLE: u8 = 1 << 1; // 2
 * const IS_PLAYER:     u8 = 1 << 2; // 4
 * // ... up to 5 more flags ...
 * 
 * fn do_stuff(information_flags: u8) { /* ... */ }
 * 
 * // ...
 * do_stuff(IS_VISIBLE | IS_PLAYER);
 * // ...
 * ```
 * 
 * With the help of this crate this can be expressed as follows:
 * 
 * ```rust
 * /*
 * bitfield! {
 *     base type, struct name;
 * 
 *     (, flag name, flag position;)*
 * }
 * */
 * bitfield! {
 *     u8, Information;
 *     , visible,    0;
 *     , invincible, 1;
 *     , player,     2;
 * }
 * ```
 * 
 * This results in the following generated code:
 * 
 * ```rust
 * pub struct Information(u8);
 * 
 * #[derive(Default)]
 * pub struct InformationInit {
 *     visible:    bool,
 *     invincible: bool,
 *     player:     bool,
 * }
 * 
 * impl Information {
 *     #[inline]
 *     pub fn visible(&self) -> bool {
 *         let max_bit_value = 1;
 *         let positioned_bits = self.0 >> 0;
 *         positioned_bits & max_bit_value == 1
 *     }
 * 
 *     #[inline]
 *     pub fn set_visible(&mut self, value: bool) {
 *         let positioned_bits = 1 << 0;
 *         let positioned_flags = (value as u8) << 0;
 *         let cleaned_flags = self.0 & !positioned_bits;
 *         self.0 = cleaned_flags | positioned_flags;
 *     }
 * 
 *     #[inline]
 *     pub fn invincible(&self) -> bool {
 *         let max_bit_value = 1;
 *         let positioned_bits = self.0 >> 1;
 *         positioned_bits & max_bit_value == 1
 *     }
 * 
 *     #[inline]
 *     pub fn set_invincible(&mut self, value: bool) {
 *         let positioned_bits = 1 << 1;
 *         let positioned_flags = (value as u8) << 1;
 *         let cleaned_flags = self.0 & !positioned_bits;
 *         self.0 = cleaned_flags | positioned_flags;
 *     }
 * 
 *     // ... same for `player`.
 * 
 *     #[inline]
 *     pub fn new(init: InformationInit) -> Self {
 *         let mut s = Information(0);
 * 
 *         s.set_visible(init.visible);
 *         s.set_invincible(init.invincible);
 *         s.set_player(init.player);
 * 
 *         s
 *     }
 * }
 * 
 * // It can now be constructed (f. e. with default values) and used like so:
 * 
 * let mut info = Information::new(InformationInit {
 *     player: true,
 *     ..Default::default()
 * });
 * 
 * // ... code ...
 * 
 * if !info.visible() {
 *     // ... code ...
 *     info.set_visible(true);
 * }
 * ```
 * 
 * ## Detailed Example
 * 
 * This example is based on the 4. parameter `UINT uType` of Microsoft Windows [user32.MessageBox function](https://msdn.microsoft.com/en-us/library/windows/desktop/ms645505.aspx) and not only stores `bool`ean flags, but also `enum` values.
 * 
 * A Microsoft Visual C++ `UINT` is a Rust `u32`. So all constants for the parameter `uType` can be written as follows:
 * 
 * ```rust
 * // Buttons
 * const MB_BUTTON_OK:                  u32 = 0;
 * const MB_BUTTON_OK_CANCEL:           u32 = 1;
 * const MB_BUTTON_ABORT_RETRY_IGNORE:  u32 = 2;
 * const MB_BUTTON_YES_NO_CANCEL:       u32 = 3;
 * const MB_BUTTON_YES_NO:              u32 = 4;
 * const MB_BUTTON_RETRY_CANCEL:        u32 = 5;
 * const MB_BUTTON_CANCEL_TRY_CONTINUE: u32 = 6;
 * 
 * // Icons
 * const MB_ICON_NONE:                  u32 = 0x00;
 * const MB_ICON_ERROR:                 u32 = 0x10;
 * const MB_ICON_QUESTION:              u32 = 0x20;
 * const MB_ICON_EXCLAMATION:           u32 = 0x30;
 * const MB_ICON_INFORMATION:           u32 = 0x40;
 * 
 * // Default buttons
 * const MB_DEFAULT_BUTTON1:            u32 = 0x000;
 * const MB_DEFAULT_BUTTON2:            u32 = 0x100;
 * const MB_DEFAULT_BUTTON3:            u32 = 0x200;
 * const MB_DEFAULT_BUTTON4:            u32 = 0x300;
 * 
 * // Modality
 * const MB_MODALITY_APPLICATION:       u32 = 0x0000;
 * const MB_MODALITY_SYSTEM:            u32 = 0x1000;
 * const MB_MODALITY_TASK:              u32 = 0x2000;
 * 
 * // Other flags
 * const MB_HELP:                       u32 = 1 << 14;
 * const MB_FOREGROUND:                 u32 = 1 << 16;
 * const MB_DEFAULT_DESKTOP_ONLY:       u32 = 1 << 17;
 * const MB_TOP_MOST:                   u32 = 1 << 18;
 * const MB_RIGHT:                      u32 = 1 << 19;
 * const MB_RIGHT_TO_LEFT_READING:      u32 = 1 << 20;
 * const MB_SERVICE_NOTIFICATION:       u32 = 1 << 21;
 * ```
 * 
 * One problem is that `u32` is not type safe like an `enum` value, another is that the usage of an `u32` is error prone, because several flags of the same "type group" like a button can be `|`-ed together (f. e. `MB_BUTTON_ABORT_RETRY_IGNORE | MB_BUTTON_YES_NO`) which might result in some unexpected behaviour. Checking if certain flags are set is also unnecessarily complicated.
 * 
 * The previously mentioned "type groups" are saved in the `u32` value as follows:
 * 
 * | Type            | Min. value (w/o 0) | Max. value | Storage bits                                            | Max. storable value             |
 * | --------------- | ------------------ | ---------- | ------------------------------------------------------- | ------------------------------- |
 * | `Button`        | 0x1                | 0x6        | 0b0000_0000_0000_0000_0000_0000_0000_0**XXX** (0 - 2)   | `((1 << 3) - 1) <<  0` = 0x7    |
 * | `Icon`          | 0x10               | 0x40       | 0b0000_0000_0000_0000_0000_0000_0**XXX**_0000 (4 - 6)   | `((1 << 3) - 1) <<  4` = 0x70   |
 * | `DefaultButton` | 0x100              | 0x300      | 0b0000_0000_0000_0000_0000_00**XX**_0000_0000 (8 - 9)   | `((1 << 2) - 1) <<  8` = 0x300  |
 * | `Modality`      | 0x1000             | 0x2000     | 0b0000_0000_0000_0000_00**XX**_0000_0000_0000 (12 - 13) | `((1 << 2) - 1) << 12` = 0x3000 |
 * 
 * All of the "type groups" can be expressed by rebasing them (removing the trailing zeros):
 * 
 * ```rust
 * #[repr(u32)]
 * #[derive(Debug, PartialEq)]
 * pub enum Button {
 *     Ok,
 *     OkCancel,
 *     AbortRetryIgnore,
 *     YesNoCancel,
 *     YesNo,
 *     RetryCancel,
 *     CancelTryContinue
 * }
 * 
 * impl Default for Button { fn default() -> Self { Button::Ok } }
 * enum_from_primitive!(Button, |x| x <= Button::CancelTryContinue as u64);
 * 
 * #[repr(u32)]
 * #[derive(Debug, PartialEq)]
 * pub enum DefaultButton {
 *     One,
 *     Two,
 *     Three,
 *     Four
 * }
 * 
 * impl Default for DefaultButton { fn default() -> Self { DefaultButton::One } }
 * enum_from_primitive!(DefaultButton, |x| x <= DefaultButton::Four as u64);
 * 
 * #[repr(u32)]
 * #[derive(Debug, PartialEq)]
 * pub enum Icon {
 *     None,
 *     Stop,
 *     Question,
 *     Exclamation,
 *     Information
 * }
 * 
 * impl Default for Icon { fn default() -> Self { Icon::None } }
 * enum_from_primitive!(Icon, |x| x <= Icon::Information as u64);
 * 
 * #[repr(u32)]
 * #[derive(Debug, PartialEq)]
 * pub enum Modality {
 *     Application,
 *     System,
 *     Task
 * }
 * 
 * impl Default for Modality { fn default() -> Self { Modality::Application } }
 * enum_from_primitive!(Modality, |x| x <= Modality::Task as u64);
 * ```
 * 
 * Now the `bitfield` macro can be used as follows:
 * 
 * ```rust
 * /*
 * bitfield! {
 *     base type, struct name;
 * 
 *     (         , flag name, flag position;)*
 *     (enum type, flag name, flag position, amount of bits;)*
 * }
 * */
 * bitfield! {
 *     u32, Style;
 *     ,              help,                  14;
 *     ,              foreground,            16;
 *     ,              default_desktop_only,  17;
 *     ,              top_most,              18;
 *     ,              right,                 19;
 *     ,              right_to_left_reading, 20;
 *     ,              service_notification,  21;
 *     Button,        button,                 0, 3;
 *     Icon,          icon,                   4, 3;
 *     DefaultButton, default_button,         8, 2;
 *     Modality,      modality,              12, 2;
 * }
 * ```
 * 
 * This results in the following generated code:
 * 
 * ```rust
 * pub struct Style(u32);
 * 
 * #[derive(Default)]
 * pub struct StyleInit {
 *     help:       bool,
 *     foreground: bool,
 *     // ...
 *     button:     Button,
 *     icon:       Icon,
 *     // ...
 * }
 * 
 * impl Style {
 *     #[inline]
 *     pub fn help(&self) -> bool {
 *         let max_bit_value = 1;
 *         let positioned_bits = self.0 >> 14;
 *         positioned_bits & max_bit_value == 1
 *     }
 * 
 *     #[inline]
 *     pub fn set_help(&mut self, value: bool) {
 *         let positioned_bits: u32 = 1 << 0;
 *         let positioned_flags = (value as u32) << 14;
 *         let cleaned_flags = self.0 & !positioned_bits;
 *         self.0 = cleaned_flags | positioned_flags;
 *     }
 * 
 *     // ...
 * 
 *     #[inline]
 *     pub fn button(&self) -> core::result::Result<Button, u32> {
 *         let max_bit_value: u32 = (1 << 3) - 1;
 *         let positioned_bits = self.0 >> 0;
 *         let value = positioned_bits & max_bit_value;
 *         let enum_value = Button::from_u64(value as u64);
 *         if enum_value.is_some() { Ok(enum_value.unwrap()) } else { Err(value) }
 *     }
 * 
 *     #[inline]
 *     pub fn set_button(&mut self, value: Button) {
 *         let max_bit_value: u32 = (1 << 3) - 1;
 *         let positioned_bits = max_bit_value << 0;
 *         let positioned_flags = (value as u32) << 0;
 *         let cleaned_flags = self.0 & !positioned_bits;
 *         self.0 = cleaned_flags | positioned_flags;
 *     }
 * 
 *     #[inline]
 *     pub fn icon(&self) -> core::result::Result<Icon, u32> {
 *         let max_bit_value: u32 = (1 << 3) - 1;
 *         let positioned_bits = self.0 >> 4;
 *         let value = positioned_bits & max_bit_value;
 *         let enum_value = Icon::from_u64(value as u64);
 *         if enum_value.is_some() { Ok(enum_value.unwrap()) } else { Err(value) }
 *     }
 * 
 *     #[inline]
 *     pub fn set_icon(&mut self, value: Icon) {
 *         let max_bit_value: u32 = (1 << 3) - 1;
 *         let positioned_bits = max_bit_value << 4;
 *         let positioned_flags = (value as u32) << 4;
 *         let cleaned_flags = self.0 & !positioned_bits;
 *         self.0 = cleaned_flags | positioned_flags;
 *     }
 * 
 *     // ...
 * 
 *     #[inline]
 *     pub fn new(init: StyleInit) -> Self {
 *         let mut s = Style(0);
 * 
 *         s.set_help(init.help);
 *         s.set_foreground(init.foreground);
 *         // ...
 * 
 *         s.set_button(init.button);
 *         s.set_icon(init.icon);
 *         // ...
 * 
 *         s
 *     }
 * }
 * ```
 * 
 * It can now be constructed (f. e. with default values) and used like so:
 * 
 * ```rust
 * let mut style = Style::new(StyleInit {
 *     button: Button::OkCancel,
 *     icon: Icon::Information,
 *     right: true,
 *     ..Default::default()
 * });
 * 
 * // ... code ...
 * 
 * if style.right() && style.button() == Button::Ok {
 *     // ... code ...
 *     style.set_button(Button::OkCancel);
 * }
 * ```
 */

/**
 * # Description
 * 
 * Macro which converts a primitive value to an enum value.
 * 
 * ## Usage
 * 
 * ```rust
 * #[repr(u32)]
 * enum Information {
 *     Info1,
 *     Info2
 * }
 * 
 * let info = enum_from_primitive!(Information, |x| x <= Information::Info2 as u64);
 * ```
 */
#[macro_export]
macro_rules! enum_from_primitive {
	($enum_type: ty, $validator: expr) => {
		impl $crate::num::traits::FromPrimitive for $enum_type {
			fn from_i64(n: i64) -> Option<Self> { Self::from_u64(n as u64) }
			fn from_u64(n: u64) -> Option<Self> {
				if $validator(n) { Some(unsafe { core::mem::transmute_copy(&n) }) } else { None }
			}
		}
	};
}

/**
 * # Description
 * 
 * Macro which creates a bitfield.
 * 
 * ## Usage
 * 
 * ```rust
 * bitfield! {
 *     base type, struct name;
 * 
 *     (         , flag name, flag position;)*
 *     (enum type, flag name, flag position, amount of bits;)*
 * }
 * ```
 */
#[macro_export]
macro_rules! bitfield {
	(
		$base_type: ty, $type_name: ident;
		$(
			, $bool_name: ident, $bool_position: expr;
		)*
		$(
			$flag_type: ident, $flag_name: ident, $flag_position: expr, $flag_amount: expr/*, $flag_validator: expr*/;
		)*
	) => {
		interpolate_idents! {
			pub struct $type_name($base_type);

			#[derive(Default)]
			pub struct [$type_name Init] {
				$(
					$bool_name: bool,
				)*

				$(
					$flag_name: $flag_type,
				)*
			}

			impl $type_name {
				$(
					#[inline]
					pub fn $bool_name(&self) -> bool {
						let max_bit_value: $base_type = 1;
						let positioned_bits = self.0 >> $bool_position;
						positioned_bits & max_bit_value == 1
					}

					#[inline]
					pub fn [set_ $bool_name](&mut self, value: bool) {
						let positioned_bits: $base_type = 1 << $bool_position;
						let positioned_flags = (value as $base_type) << $bool_position;
						let cleaned_flags = self.0 & !positioned_bits;
						self.0 = cleaned_flags | positioned_flags;
					}
				)*

				$(
					#[inline]
					pub fn $flag_name(&self) -> core::result::Result<$flag_type, $base_type> {
						use $crate::num::traits::FromPrimitive;

						let max_bit_value = (1 << $flag_amount) - 1;
						let positioned_bits = self.0 >> $flag_position;
						let value = positioned_bits & max_bit_value;
						let enum_value = $flag_type::from_u64(value as u64);
						if enum_value.is_some() { Ok(enum_value.unwrap()) } else { Err(value) }
					}

					#[inline]
					pub fn [set_ $flag_name](&mut self, value: $flag_type) {
						let max_bit_value = (1 << $flag_amount) - 1;
						let positioned_bits = max_bit_value << $flag_position;
						let positioned_flags = (value as $base_type) << $flag_position;
						let cleaned_flags = self.0 & !positioned_bits;
						self.0 = cleaned_flags | positioned_flags;
					}
				)*

				#[inline]
				pub fn new(init: [$type_name Init]) -> Self {
					let mut s = $type_name(0);

					$(
						s.[set_ $bool_name](init.$bool_name);
					)*

					$(
						s.[set_ $flag_name](init.$flag_name);
					)*

					s
				}
			}
		}
	};
}

#[cfg(test)]
mod tests {
	use core;

	#[allow(dead_code)]
	#[repr(u32)]
	#[derive(Debug, PartialEq)]
	pub enum Button {
		Ok,
		OkCancel,
		AbortRetryIgnore,
		YesNoCancel,
		YesNo,
		RetryCancel,
		CancelTryContinue
	}

	impl Default for Button { fn default() -> Self { Button::Ok } }
	enum_from_primitive!(Button, |x| x <= Button::CancelTryContinue as u64);

	#[allow(dead_code)]
	#[repr(u32)]
	#[derive(Debug, PartialEq)]
	pub enum DefaultButton {
		One,
		Two,
		Three,
		Four
	}

	impl Default for DefaultButton { fn default() -> Self { DefaultButton::One } }
	enum_from_primitive!(DefaultButton, |x| x <= DefaultButton::Four as u64);

	#[allow(dead_code)]
	#[repr(u32)]
	#[derive(Debug, PartialEq)]
	pub enum Icon {
		None,
		Stop,
		Question,
		Exclamation,
		Information
	}

	impl Default for Icon { fn default() -> Self { Icon::None } }
	enum_from_primitive!(Icon, |x| x <= Icon::Information as u64);

	#[allow(dead_code)]
	#[repr(u32)]
	#[derive(Debug, PartialEq)]
	pub enum Modality {
		Application,
		System,
		Task
	}

	impl Default for Modality { fn default() -> Self { Modality::Application } }
	enum_from_primitive!(Modality, |x| x <= Modality::Task as u64);

	bitfield! {
		u32, Style;
		,				help,					14;
		,				foreground,				16;
		,				default_desktop_only,	17;
		,				top_most,				18;
		,				right,					19;
		,				right_to_left_reading,	20;
		,				service_notification,	21;
		Button,			button,					 0, 3;
		Icon,			icon,					 4, 3;
		DefaultButton,	default_button,			 8, 2;
		Modality,		modality,				12, 2;
	}

	#[test]
	fn test_style() {
		// Test defaults.
		let mut style = Style(0);

		assert_eq!(style.button().ok(),			Some(Button::Ok));
		assert_eq!(style.icon().ok(),			Some(Icon::None));
		assert_eq!(style.default_button().ok(),	Some(DefaultButton::One));
		assert_eq!(style.modality().ok(),		Some(Modality::Application));
		assert!(!style.help());
		assert!(!style.foreground());
		assert!(!style.default_desktop_only());
		assert!(!style.top_most());
		assert!(!style.right());
		assert!(!style.right_to_left_reading());
		assert!(!style.service_notification());

		// Test overflow.
		style = Style(0xFFFFFFFF);

		assert_eq!(style.button().err(),		Some((1 << 3) - 1));
		assert_eq!(style.icon().err(),			Some((1 << 3) - 1));
		assert_eq!(style.default_button().ok(),	Some(DefaultButton::Four));
		assert_eq!(style.modality().err(),		Some((1 << 2) - 1));
		assert!(style.help());
		assert!(style.foreground());
		assert!(style.default_desktop_only());
		assert!(style.top_most());
		assert!(style.right());
		assert!(style.right_to_left_reading());
		assert!(style.service_notification());

		/* Test max.
		 * 
		 * Flags:	    SR_RTDF_xHMM_xxDD_xIII_xBBB
 		 * Mean:	    TT_TTTT_FT2._FF3._F4.._F6..
		 */
		style = Style(0b11_1111_0110_0011_0100_0110);

		assert_eq!(style.button().ok(),			Some(Button::CancelTryContinue));
		assert_eq!(style.icon().ok(),			Some(Icon::Information));
		assert_eq!(style.default_button().ok(),	Some(DefaultButton::Four));
		assert_eq!(style.modality().ok(),		Some(Modality::Task));
		assert!(style.help());
		assert!(style.foreground());
		assert!(style.default_desktop_only());
		assert!(style.top_most());
		assert!(style.right());
		assert!(style.right_to_left_reading());
		assert!(style.service_notification());

		// Test setters.
		style = Style(0);

		style.set_button(Button::CancelTryContinue);
		assert_eq!(style.button().ok(),					Some(Button::CancelTryContinue));
		style.set_button(Button::Ok);
		assert_eq!(style.button().ok(),					Some(Button::Ok));

		style.set_icon(Icon::Information);
		assert_eq!(style.icon().ok(),					Some(Icon::Information));
		style.set_icon(Icon::None);
		assert_eq!(style.icon().ok(),					Some(Icon::None));

		style.set_default_button(DefaultButton::Four);
		assert_eq!(style.default_button().ok(),			Some(DefaultButton::Four));
		style.set_default_button(DefaultButton::One);
		assert_eq!(style.default_button().ok(),			Some(DefaultButton::One));

		style.set_modality(Modality::Task);
		assert_eq!(style.modality().ok(),				Some(Modality::Task));
		style.set_modality(Modality::Application);
		assert_eq!(style.modality().ok(),				Some(Modality::Application));

		style.set_help(true);
		assert!(style.help());
		style.set_help(false);
		assert!(!style.help());

		style.set_foreground(true);
		assert!(style.foreground());
		style.set_foreground(false);
		assert!(!style.foreground());

		style.set_default_desktop_only(true);
		assert!(style.default_desktop_only());
		style.set_default_desktop_only(false);
		assert!(!style.default_desktop_only());

		style.set_top_most(true);
		assert!(style.top_most());
		style.set_top_most(false);
		assert!(!style.top_most());

		style.set_right(true);
		assert!(style.right());
		style.set_right(false);
		assert!(!style.right());

		style.set_right_to_left_reading(true);
		assert!(style.right_to_left_reading());
		style.set_right_to_left_reading(false);
		assert!(!style.right_to_left_reading());

		style.set_service_notification(true);
		assert!(style.service_notification());
		style.set_service_notification(false);
		assert!(!style.service_notification());

		// Test StyleInit defaults.
		style = Style::new(StyleInit { ..Default::default() });

		assert_eq!(style.button().ok(),			Some(Button::Ok));
		assert_eq!(style.icon().ok(),			Some(Icon::None));
		assert_eq!(style.default_button().ok(),	Some(DefaultButton::One));
		assert_eq!(style.modality().ok(),		Some(Modality::Application));
		assert!(!style.help());
		assert!(!style.foreground());
		assert!(!style.default_desktop_only());
		assert!(!style.top_most());
		assert!(!style.right());
		assert!(!style.right_to_left_reading());
		assert!(!style.service_notification());

		// Test StyleInit max.
		style = Style::new(StyleInit {
			help:					true,
			foreground:				true,
			default_desktop_only:	true,
			top_most:				true,
			right:					true,
			right_to_left_reading:	true,
			service_notification:	true,
			button:					Button::CancelTryContinue,
			icon:					Icon::Information,
			default_button:			DefaultButton::Four,
			modality:				Modality::Task
		});

		assert_eq!(style.button().ok(),			Some(Button::CancelTryContinue));
		assert_eq!(style.icon().ok(),			Some(Icon::Information));
		assert_eq!(style.default_button().ok(),	Some(DefaultButton::Four));
		assert_eq!(style.modality().ok(),		Some(Modality::Task));
		assert!(style.help());
		assert!(style.foreground());
		assert!(style.default_desktop_only());
		assert!(style.top_most());
		assert!(style.right());
		assert!(style.right_to_left_reading());
		assert!(style.service_notification());
	}
}