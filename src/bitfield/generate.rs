//! Contains code to generate bit fields.

use syn::spanned::Spanned;

impl super::BitField {
    /// Returns the narrowest primitive size for a field.
    fn field_primitive_size(size: u8) -> u8 {
        const SIZES: &[u8] = &[8, 16, 32, 64, 128];

        for s in SIZES {
            if size <= *s { return *s; }
        }

        unimplemented!("field size {} > {}", size, SIZES.last().unwrap());
    }

    /// Generates the accessors for a single entry.
    fn generate_accessor(
        &self,
        entry: &super::Entry,
        getter: &syn::Ident,
        setter: &syn::Ident
    ) -> proc_macro2::TokenStream {
        let ty_size = &self.attr.size;
        let attrs = &entry.attrs;
        let vis = &entry.vis;
        let ty = &entry.ty;

        if let Some(field) = &entry.field
        {
            let bit = field.bit.as_ref().unwrap().base10_parse::<u8>().unwrap();
            let size = field.size.as_ref().unwrap().base10_parse::<u8>().unwrap();

            // Conversion for signed fields.
            let primitive_size = super::BitField::field_primitive_size(size);
            let conversion = if field.signed.is_some() || ty.get_ident().map(
                |ident| crate::primitive::is_signed_primitive(ident)
            ).unwrap_or_default() {
                quote::ToTokens::to_token_stream(&syn::parse_str::<syn::Expr>(
                    &format!("u{} as {}", primitive_size, ty_size)
                ).unwrap())
            } else { quote::quote! { #ty_size } };

            // Special handling for primitive types.
            if let Some(ty) = ty.get_ident() {
                if crate::primitive::is_bool(ty) {
                    return quote::quote! {
                        #(#attrs)*
                        /// Gets the value of the field.
                        #[inline(always)]
                        #vis const fn #getter(&self) -> #ty {
                            self._bit(#bit)
                        }

                        #(#attrs)*
                        /// Creates a copy of the bit field with the new value.
                        #[inline(always)]
                        #[must_use = "leaves `self` unmodified and returns a modified variant"]
                        #vis const fn #setter(&self, value: #ty) -> Self {
                            self._set_bit(#bit, value)
                        }
                    };
                } else if crate::primitive::is_signed_primitive(ty) {
                    return quote::quote! {
                        #(#attrs)*
                        /// Gets the value of the field.
                        #[inline(always)]
                        #vis const fn #getter(&self) -> #ty {
                            self._field(#bit, #size) as #ty
                        }

                        #(#attrs)*
                        /// Creates a copy of the bit field with the new value.
                        #[inline(always)]
                        #[must_use = "leaves `self` unmodified and returns a modified variant"]
                        #vis const fn #setter(&self, value: #ty) -> Self {
                            self._set_field(#bit, #size, value as #conversion)
                        }
                    };
                } else if crate::primitive::is_unsigned_primitive(ty) {
                    return if crate::primitive::primitive_bits(ty).unwrap() != size {
                        // Fields with a size < bits_of(FieldPrimitive).
                        quote::quote! {
                            #(#attrs)*
                            /// Gets the value of the field.
                            #[inline(always)]
                            #vis const fn #getter(&self) -> #ty {
                                self._field(#bit, #size) as #ty
                            }

                            // TODO: Use ranged integers when they land: https://github.com/rust-lang/rfcs/issues/671.
                            #(#attrs)*
                            /// Creates a copy of the bit field with the new value.
                            ///
                            /// Returns `None` if `value` is bigger than the specified amount of
                            /// bits for the field can store.
                            #[inline(always)]
                            #[must_use = "leaves `self` unmodified and returns a modified variant"]
                            #vis const fn #setter(&self, value: #ty) -> Option<Self> {
                                if value >= (1 as #ty).wrapping_shl(#size as u32) {
                                    return None;
                                }

                                Some(self._set_field(#bit, #size, value as #ty_size))
                            }
                        }
                    } else {
                        // Fields with a size == bits_of(FieldPrimitive).
                        quote::quote! {
                            #(#attrs)*
                            /// Gets the value of the field.
                            #[inline(always)]
                            #vis const fn #getter(&self) -> #ty {
                                self._field(#bit, #size) as #ty
                            }

                            #(#attrs)*
                            /// Creates a copy of the bit field with the new value.
                            #[inline(always)]
                            #[must_use = "leaves `self` unmodified and returns a modified variant"]
                            #vis const fn #setter(&self, value: #ty) -> Self {
                                self._set_field(#bit, #size, value as #ty_size)
                            }
                        }
                    }
                }
            }

            // Handling for non-primitive types.

            // Generate the minimal primitive type the field needs.
            let primitive = syn::Ident::new(
                &format!("{}{}",
                    if field.signed.is_none() { 'u' } else { 'i' },
                    primitive_size
                ), field.size.span()
            );

            quote::quote! {
                #(#attrs)*
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                // TODO: Remove when https://github.com/rust-lang/rfcs/pull/2632 is merged.
                #[cfg(const_trait_impl)]
                #vis const fn #getter(&self) -> core::result::Result<#ty, #primitive> {
                    core::convert::TryFrom::try_from(self._field(#bit, #size) as #primitive)
                }

                // TODO: Remove when https://github.com/rust-lang/rfcs/pull/2632 is merged.
                #(#attrs)*
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                #vis fn #getter(&self) -> core::result::Result<#ty, #primitive> {
                    core::convert::TryFrom::try_from(self._field(#bit, #size) as #primitive)
                }

                #(#attrs)*
                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                #vis const fn #setter(&self, value: #ty) -> Self {
                    self._set_field(#bit, #size, value as #conversion)
                }
            }
        } else {
            let getter_mask = syn::Ident::new(
                &format!("{}_mask", getter.to_string()), getter.span()
            );
            let getter_all = syn::Ident::new(
                &format!("{}_all", getter.to_string()), getter.span()
            );
            let getter_any = syn::Ident::new(
                &format!("{}_any", getter.to_string()), getter.span()
            );
            let setter_all = syn::Ident::new(
                &format!("{}_all", setter.to_string()), setter.span()
            );
            let setter_none = syn::Ident::new(
                &format!("{}_none", setter.to_string()), setter.span()
            );

            quote::quote! {
                #(#attrs)*
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                #vis const fn #getter(&self, flag: #ty) -> bool {
                    self._bit(flag as u8)
                }

                #(#attrs)*
                /// Returns a bit mask of all possible flags.
                #[inline(always)]
                #vis const fn #getter_mask() -> #ty_size {
                    let mut mask = 0;

                    let mut i = 0;
                    while i < #ty::iter().len() {
                        mask |= 1 << (#ty::iter()[i] as #ty_size);

                        i += 1;
                    }

                    mask
                }

                #(#attrs)*
                /// Returns `true` if all flags are set.
                #[inline(always)]
                #vis const fn #getter_all(&self) -> bool {
                    (self.0 & Self::#getter_mask()) == Self::#getter_mask()
                }

                #(#attrs)*
                /// Returns `true` if any flag is set.
                #[inline(always)]
                #vis const fn #getter_any(&self) -> bool {
                    (self.0 & Self::#getter_mask()) != 0
                }

                #(#attrs)*
                /// Creates a copy of the bit field with the new value for the specified flag.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                #vis const fn #setter(&self, flag: #ty, value: bool) -> Self {
                    self._set_bit(flag as u8, value)
                }

                #(#attrs)*
                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                #vis const fn #setter_all(&self) -> Self {
                    Self(self.0 | Self::#getter_mask())
                }

                #(#attrs)*
                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                #vis const fn #setter_none(&self) -> Self {
                    Self(self.0 & !Self::#getter_mask())
                }
            }
        }
    }

    /// Generates the getters and setters for all fields and flags.
    fn generate_accessors(&self) -> proc_macro2::TokenStream {
        let fields = match &self.data {
            super::Data::Named(entries) => {
                let mut fields = vec!();

                for entry in entries {
                    fields.push(Self::generate_accessor(
                        &self, &entry.entry, &entry.ident, &syn::Ident::new(
                            &format!("set_{}", &entry.ident), entry.ident.span()
                        )
                    ));
                }

                fields
            },

            super::Data::Tuple(entry) => {
                vec!(Self::generate_accessor(
                    &self, entry, &syn::Ident::new(
                        if entry.field.is_some() { "get" } else { "has" },
                        entry.ty.span()
                    ), &syn::Ident::new("set", entry.ty.span())
                ))
            }
        };

        if fields.is_empty() {
            return proc_macro2::TokenStream::new();
        }

        let ident = &self.ident;

        quote::quote! {
            impl #ident {
                #(#fields)*
            }
        }
    }

    /// Generates the accessors that directly work on the primitive bit field type.
    fn generate_accessors_low(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let ty_size = &self.attr.size;

        quote::quote! {
            impl #ident {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0 >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0 & !(1 << position);

                    Self(cleared | ((value as #ty_size) << position))
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> #ty_size {
                    let shifted = self.0 >> position;

                    let rest = size as #ty_size % (core::mem::size_of::<#ty_size>() * 8) as #ty_size;
                    let bit = (rest > 0) as #ty_size;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as #ty_size);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: #ty_size) -> Self {
                    let rest = size as #ty_size % (core::mem::size_of::<#ty_size>() * 8) as #ty_size;
                    let bit = (rest > 0) as #ty_size;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as #ty_size);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        }
    }

    /// Generates constant assertions about field type sizes and flag overlaps.
    fn generate_assertions(&self) -> proc_macro2::TokenStream {
        /// Generates a constant assertion for a constant expression.
        fn generate_assertion(name: &syn::Ident, expression: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
            // TODO: Wait for constant assertions with friendlier error messages.
            // Adapted from https://crates.io/crates/static_assertions.
            quote::quote! {
                const #name: [(); 0 - #expression as usize] = [];
            }
        }

        let ty_size = &self.attr.size;

        let entries = self.data.entries();
        let assertions = entries.iter().enumerate().map(|(i, entry)| {
            let ty = &entry.ty;
            if let Some(field) = &entry.field {
                let bit = field.bit.as_ref().unwrap();
                let size = field.size.as_ref().unwrap();

                // `bits_of(FieldType)` must not be < `field.size`.
                let field_assertion = generate_assertion(&syn::Ident::new(&format!(
                    "_TYPE_IN_FIELD_{}_IS_SMALLER_THAN_THE_SPECIFIED_SIZE_OF_{}_BITS", i, &size
                ), size.span()), quote::quote! {
                    !{ const ASSERT: bool = core::mem::size_of::<#ty>() * 8 >= #size; ASSERT }
                });

                // Only generate this check for non-primitive types.
                let signed_size_assertion = ty.get_ident()
                    .map(|ident| !crate::primitive::is_primitive(ident))
                    .unwrap_or_default()
                    .then(|| {
                        // If `FieldType` is signed, the `field.size` must be exactly `bits_of(FieldType)`.
                        Some(generate_assertion(&syn::Ident::new(&format!(
                            "_SIGNED_TYPE_IN_FIELD_{}_CAN_NEVER_BE_NEGATIVE", i
                        ), size.span()), quote::quote! {
                            !{
                                const ASSERT: bool =
                                    !#ty::is_signed() || core::mem::size_of::<#ty>() * 8 == #size;
                                ASSERT
                            }
                        }))
                    }).unwrap_or_default();

                // Only generate the next assertions if this can not be checked in the parsing phase,
                // aka. when the primitive base type is `usize`.
                if self.attr.bits.is_some() {
                    return quote::quote! {
                        #field_assertion

                        #signed_size_assertion
                    }
                }

                // `bits_of(BitField)` must not be < `bits_of(Field) + size_of(Field)`.
                let size_assertion = generate_assertion(&syn::Ident::new(&format!(
                    "_FIELD_{}_EXCEEDS_THE_BIT_FIELD_SIZE", i
                ), field.span), quote::quote! {
                    !{
                        const ASSERT: bool = core::mem::size_of::<#ty_size>() * 8 >= #bit + #size;
                        ASSERT
                    }
                });

                // `bits_of(BitField)` must not be == `size_of(Field)`.
                let size_not_equal_assertion = generate_assertion(&syn::Ident::new(&format!(
                    "_FIELD_{}_HAS_THE_SIZE_OF_THE_WHOLE_BIT_FIELD", i
                ), field.span), quote::quote! {
                    !{
                        const ASSERT: bool = core::mem::size_of::<#ty_size>() * 8 != #size;
                        ASSERT
                    }
                });

                quote::quote! {
                    #field_assertion

                    #signed_size_assertion

                    #size_assertion

                    #size_not_equal_assertion
                }
            } else {
                // `bits_of(Flags)` must be `bits_of(u8)`.
                let size_assertion = generate_assertion(&syn::Ident::new(&format!(
                    "_FLAGS_IN_FIELD_{}_MUST_BE_REPR_U8", i
                ), entry.ty.span()), quote::quote! {
                    !{ const ASSERT: bool = core::mem::size_of::<#ty>() == 1; ASSERT }
                });

                // `Flags::max()` must not be >= `bits_of(BitField)`.
                let max_assertion = generate_assertion(&syn::Ident::new(&format!(
                    "_FLAGS_IN_FIELD_{}_EXCEED_THE_BIT_FIELD_SIZE", i
                ), entry.ty.span()), quote::quote! { !{
                    const ASSERT: bool =
                        core::mem::size_of::<#ty_size>() * 8 > #ty::max() as usize;
                    ASSERT
                } });

                let flag_assertions = entries.iter().enumerate().map(
                    |(inner_i, inner)|
                {
                    if i == inner_i { return proc_macro2::TokenStream::new(); }

                    // Do not check flag overlapping if overlapping is allowed.
                    if self.attr.allow_overlaps.is_some() {
                        return proc_macro2::TokenStream::new();
                    }

                    let f1 = &entry.ty;
                    let f2 = &inner.ty;

                    if let Some(field) = &inner.field {
                        let name = syn::Ident::new(&format!(
                            "_FLAGS_IN_FIELD_{}_OVERLAP_WITH_FIELD_{}", i, inner_i
                        ), f2.span());

                        let fn_name = syn::Ident::new(
                            &name.to_string().to_ascii_lowercase(), name.span()
                        );

                        let assertion = generate_assertion(
                            &name, quote::quote! { Self::#fn_name() }
                        );

                        let bit = field.bit.as_ref().unwrap().base10_parse::<u8>().unwrap();
                        let size = field.size.as_ref().unwrap().base10_parse::<u8>().unwrap();

                        quote::quote! {
                            const fn #fn_name() -> bool {
                                let flags = #f1::iter();

                                let mut i = 0;
                                while i < flags.len() {
                                    let flag = flags[i] as u8;
                                    if flag >= #bit && flag < #bit + #size {
                                        return true;
                                    }

                                    i += 1;
                                }

                                false
                            }

                            #assertion
                        }
                    } else {
                        // Skip flags overlap flags check for previously defined flags, as they
                        // already check for flags that are defined after them.
                        if i > inner_i { return proc_macro2::TokenStream::new(); }

                        let name = syn::Ident::new(&format!(
                            "_FLAGS_IN_FIELD_{}_OVERLAP_WITH_FLAGS_IN_FIELD_{}", i, inner_i
                        ), f2.span());

                        let fn_name = syn::Ident::new(
                            &name.to_string().to_ascii_lowercase(), name.span()
                        );

                        let assertion = generate_assertion(
                            &name, quote::quote! { Self::#fn_name() }
                        );

                        quote::quote! {
                            const fn #fn_name() -> bool {
                                let f1 = #f1::iter();
                                let f2 = #f2::iter();

                                let mut i1 = 0;
                                while i1 < f1.len() {
                                    let mut i2 = 0;
                                    while i2 < f2.len() {
                                        if (f1[i1] as u32) == (f2[i2] as u32) {
                                            return true;
                                        }

                                        i2 += 1;
                                    }

                                    i1 += 1;
                                }

                                false
                            }

                            #assertion
                        }
                    }
                });

                quote::quote! {
                    #size_assertion

                    #max_assertion

                    #(#flag_assertions)*
                }
            }
        });

        if assertions.len() == 0 { return proc_macro2::TokenStream::new(); }

        let ident = &self.ident;

        quote::quote! {
            impl #ident {
                #(#assertions)*
            }
        }
    }

    /// Generates a debug/display sequence for fields.
    fn generate_print_field(
        entry: &super::Entry, getter: &syn::Ident, print: proc_macro2::TokenStream
    ) -> proc_macro2::TokenStream {
        entry.ty.get_ident().and_then(|ty| crate::primitive::is_primitive(ty).then(
            || quote::quote! {
                let value = self.#getter();
                #print
            }
        )).unwrap_or_else(|| quote::quote! {
            let value = self.#getter();
            if let core::result::Result::Ok(value) = value {
                #print
            } else {
                #print
            }
        })
    }

    /// Generates the `core::fmt::Debug` implementation, if `#[derive(Debug)]` is specified.
    /// Expects all flags to expose a `fn iter() -> &'static [Self]`, all flags and fields to
    /// implement `core::fmt::Debug`, and all flags to implement `core::marker::Copy` and
    /// `core::clone::Clone`.
    ///
    /// Since field getters return a result, an `Ok` value will be unwrapped before it is printed,
    /// to hide the `Ok(<value>)` around the `<value>`.
    fn generate_debug(&self) -> proc_macro2::TokenStream {
        if self.debug.is_none() { return proc_macro2::TokenStream::new(); }

        let ident = &self.ident;

        let fields = match &self.data {
            super::Data::Named(entries) => {
                let mut fields = vec!();

                for entry in entries {
                    let ident = &entry.ident;

                    fields.push(if entry.entry.field.is_some() {
                        // Display fields as a normal struct field.
                        super::BitField::generate_print_field(&entry.entry, ident, quote::quote! {
                            s.field(core::stringify!(#ident), &value);
                        })
                    } else {
                        // Display flags as sub structure with a `bool` field for each flag.
                        let self_ident = &self.ident;
                        let ident = &entry.ident;
                        let ty = &entry.entry.ty;
                        let ty_name = &entry.entry.ty.segments.last().unwrap().ident;

                        quote::quote! {{
                            struct BitFieldDebugImplementor<'a>(&'a #self_ident);

                            impl<'a> core::fmt::Debug for BitFieldDebugImplementor<'a> {
                                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                                    let mut s = f.debug_struct(core::stringify!(#ty_name));

                                    for flag in <#ty>::iter() {
                                        s.field(&alloc::format!("{:?}", flag), &self.0 . #ident(*flag));
                                    }

                                    s.finish()
                                }
                            }

                            s.field(core::stringify!(#ident), &BitFieldDebugImplementor(&self));
                        }}
                    });
                }

                fields
            },

            super::Data::Tuple(entry) => {
                let ty = &entry.ty;

                vec!(if entry.field.is_some()
                {
                    // Display fields as a normal struct field.
                    let ident = &ty.segments.last().unwrap().ident;
                    let getter = syn::Ident::new("get", ty.span());

                    super::BitField::generate_print_field(entry, &getter, quote::quote! {
                        s.field(core::stringify!(#ident), &value);
                    })
                } else {
                    // Display each flag as a `bool` field.
                    quote::quote! {
                        for flag in <#ty>::iter() {
                            s.field(&alloc::format!("{:?}", flag), &self.has(*flag));
                        }
                    }
                })
            }
        };

        quote::quote! {
            impl core::fmt::Debug for #ident {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut s = f.debug_struct(core::stringify!(#ident));

                    #(#fields)*

                    s.finish()
                }
            }
        }
    }

    /// If `#[derive(Display)]` is specified this generates the `core::fmt::Display` implementation,
    /// for tuple bit fields, or named bit fields that only host flags, otherwise an empty
    /// `TokenStream` is generated. Expects all flags to expose a `fn iter() -> &'static [Self]` and
    /// implement `core::marker::Copy`, `core::clone::Clone`, and all flags and fields to implement
    /// `core::fmt::Debug`.
    fn generate_display(&self) -> proc_macro2::TokenStream {
        /// Generates the implementation for a single entry (named or tuple struct).
        fn generate_display_for_entry(
            ident: &syn::Ident, getter: &syn::Ident, entry: &super::Entry
        ) -> proc_macro2::TokenStream {
            let ty = &entry.ty;

            let implementation = if entry.field.is_some()
            {
                // Display the field value.
                super::BitField::generate_print_field(entry, getter, quote::quote! {
                    f.write_str(&alloc::format!("{:?}", value))
                })
            } else {
                // Display all set flags joined with `" | "`, or "-" if no flag is set at all.
                quote::quote! {
                    let mut flags = alloc::vec::Vec::new();

                    for flag in <#ty>::iter() {
                        if self.#getter(*flag) {
                            flags.push(alloc::format!("{:?}", flag));
                        }
                    }

                    let flags = flags.join(" | ");

                    f.write_str(if flags.len() > 0 { &flags } else { "-" })
                }
            };

            quote::quote! {
                impl core::fmt::Display for #ident {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        #implementation
                    }
                }
            }
        }

        if self.display.is_none() { return proc_macro2::TokenStream::new(); }

        let ident = &self.ident;

        match &self.data {
            super::Data::Named(entries) => {
                if entries.len() == 0 {
                    // Do not generate `Display` for bit fields with no fields or flags at all.
                    // Should have been checked in `parse::validate_display`.
                    panic!("can not generate `Display` for empty bit fields");
                } else if entries.len() == 1 {
                    let first = entries.first().unwrap();
                    generate_display_for_entry(ident, &first.ident, &first.entry)
                } else {
                    // Do not generate `Display` for bit fields with non-flags.
                    // Should have been checked in `parse::validate_display`.
                    for entry in entries {
                        if entry.entry.field.is_some() {
                            panic!("can not generate `Display` for bit fields with non-flag fields");
                        }
                    }

                    let iterators = entries.iter().map(|c| {
                        let ty = &c.entry.ty;
                        let ident = &c.ident;

                        let format_data = if entries.len() > 1 {
                            quote::quote! { "{}::{:?}", core::any::type_name::<#ty>(), flag }
                        } else {
                            quote::quote! { "{:?}", flag }
                        };

                        quote::quote! {
                            for flag in <#ty>::iter() {
                                if self.#ident(*flag) {
                                    flags.push(alloc::format!(#format_data));
                                }
                            }
                        }
                    });

                    quote::quote! {
                        impl core::fmt::Display for #ident {
                            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                                let mut flags = alloc::vec::Vec::new();

                                #(#iterators)*

                                let flags = flags.join(" | ");

                                f.write_str(
                                    if flags.len() > 0 { &flags } else { "-" }
                                )
                            }
                        }
                    }
                }
            },

            super::Data::Tuple(entry) => {
                let getter = syn::Ident::new(
                    if entry.field.is_some() { "get" } else { "has" },
                    entry.ty.span()
                );
                generate_display_for_entry(ident, &getter, entry)
            }
        }
    }

    /// Generates the main bit field implementation.
    fn generate_impl(&self) -> proc_macro2::TokenStream {
        let vis = &self.vis;
        let ident = &self.ident;

        quote::quote! {
            impl #ident {
                /// Creates a new instance with all flags and fields cleared.
                #[inline(always)]
                #vis const fn new() -> Self {
                    Self(0)
                }
            }
        }
    }

    /// Generates the main bit field structure.
    fn generate_struct(&self) -> proc_macro2::TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let ident = &self.ident;
        let size = &self.attr.size;

        quote::quote! {
            #[repr(C)]
            #(#attrs)*
            #vis struct #ident(#size);
        }
    }
}

/// Generates the user code for the parsed bit field.
impl core::convert::Into<proc_macro2::TokenStream> for super::BitField {
    fn into(self) -> proc_macro2::TokenStream {
        let field = self.generate_struct();
        let implementation = self.generate_impl();
        let accessors_low = self.generate_accessors_low();
        let accessors = self.generate_accessors();
        let assertions = self.generate_assertions();
        let debug = self.generate_debug();
        let display = self.generate_display();

        quote::quote! {
            #field
            #implementation
            #accessors_low
            #accessors
            #assertions
            #debug
            #display
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use syn::spanned::Spanned;

    macro_rules! assert_accessor {
        ($attribute:expr, $item:expr, $result:expr) => {{
            let bitfield = parse_valid!($attribute, $item);

            if let super::super::Data::Tuple(entry) = &bitfield.data {
                let getter = syn::Ident::new("test_get", entry.ty.span());
                let setter = syn::Ident::new("test_set", entry.ty.span());

                assert_eq!(
                    bitfield.generate_accessor(&entry, &getter, &setter).to_string(),
                    $result.to_string()
                );
            } else { panic!("expected tuple struct") }
        }};
    }

    macro_rules! assert_compare {
        ($generator:ident, $attribute:expr, $item:expr, $result:expr) => {{
            let bitfield = parse_valid!($attribute, $item).$generator().to_string();
            let expected = $result.to_string();

            assert_eq!(&bitfield, &expected);
        }};
    }

    // Test macros.

    #[test]
    #[should_panic]
    fn test_assert_accessor() {
        assert_accessor!("8", "struct A(A);", quote::quote! {});
    }

    #[test]
    #[should_panic]
    fn test_assert_compare() {
        assert_compare!(generate_struct, "8", "struct A(A);", quote::quote! {});
    }

    // Test generation.

    #[test]
    fn field_primitive_size() {
        const SIZES: &[(u8, u8)] = &[
            ( 0,   8), ( 1,   8), (  7,   8), (  8,   8),
            ( 9,  16), (10,  16), ( 15,  16), ( 16,  16),
            (17,  32), (18,  32), ( 31,  32), ( 32,  32),
            (33,  64), (34,  64), ( 63,  64), ( 64,  64),
            (65, 128), (66, 128), (127, 128), (128, 128)
        ];

        for size in SIZES {
            assert_eq!(BitField::field_primitive_size(size.0), size.1);
        }
    }

    #[test]
    fn accessor_attrs() {
        assert_accessor!("8", "struct A(#[some_attribute1] #[some_attribute2] A);", quote::quote! {
            #[some_attribute1]
            #[some_attribute2]
            /// Returns `true` if the specified `flag` is set.
            #[inline(always)]
            const fn test_get(&self, flag: A) -> bool {
                self._bit(flag as u8)
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Returns a bit mask of all possible flags.
            #[inline(always)]
            const fn test_get_mask() -> u8 {
                let mut mask = 0;

                let mut i = 0;
                while i < A::iter().len() {
                    mask |= 1 << (A::iter()[i] as u8);

                    i += 1;
                }

                mask
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Returns `true` if all flags are set.
            #[inline(always)]
            const fn test_get_all(&self) -> bool {
                (self.0 & Self::test_get_mask()) == Self::test_get_mask()
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Returns `true` if any flag is set.
            #[inline(always)]
            const fn test_get_any(&self) -> bool {
                (self.0 & Self::test_get_mask()) != 0
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with the new value for the specified flag.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, flag: A, value: bool) -> Self {
                self._set_bit(flag as u8, value)
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                Self(self.0 | Self::test_get_mask())
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                Self(self.0 & !Self::test_get_mask())
            }
        });

        assert_accessor!(
            "8", "struct A(#[some_attribute1] #[some_attribute2] #[field(0, 1)] A);", quote::quote! {
                #[some_attribute1]
                #[some_attribute2]
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn test_get(&self) -> core::result::Result<A, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                #[some_attribute1]
                #[some_attribute2]
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn test_get(&self) -> core::result::Result<A, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                #[some_attribute1]
                #[some_attribute2]
                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: A) -> Self {
                    self._set_field(0u8, 1u8, value as u8)
                }
            }
        );
    }

    #[test]
    fn accessor_vis() {
        assert_accessor!("8", "struct A(pub A);", quote::quote! {
            /// Returns `true` if the specified `flag` is set.
            #[inline(always)]
            pub const fn test_get(&self, flag: A) -> bool {
                self._bit(flag as u8)
            }

            /// Returns a bit mask of all possible flags.
            #[inline(always)]
            pub const fn test_get_mask() -> u8 {
                let mut mask = 0;

                let mut i = 0;
                while i < A::iter().len() {
                    mask |= 1 << (A::iter()[i] as u8);

                    i += 1;
                }

                mask
            }

            /// Returns `true` if all flags are set.
            #[inline(always)]
            pub const fn test_get_all(&self) -> bool {
                (self.0 & Self::test_get_mask()) == Self::test_get_mask()
            }

            /// Returns `true` if any flag is set.
            #[inline(always)]
            pub const fn test_get_any(&self) -> bool {
                (self.0 & Self::test_get_mask()) != 0
            }

            /// Creates a copy of the bit field with the new value for the specified flag.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            pub const fn test_set(&self, flag: A, value: bool) -> Self {
                self._set_bit(flag as u8, value)
            }

            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            pub const fn test_set_all(&self) -> Self {
                Self(self.0 | Self::test_get_mask())
            }

            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            pub const fn test_set_none(&self) -> Self {
                Self(self.0 & !Self::test_get_mask())
            }
        });

        assert_accessor!(
            "8", "struct A(#[field(0, 1)] pub A);", quote::quote! {
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                pub const fn test_get(&self) -> core::result::Result<A, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                pub fn test_get(&self) -> core::result::Result<A, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                pub const fn test_set(&self, value: A) -> Self {
                    self._set_field(0u8, 1u8, value as u8)
                }
            }
        );
    }

    #[test]
    fn accessor_ty() {
        assert_accessor!("8", "struct A(B);", quote::quote! {
            /// Returns `true` if the specified `flag` is set.
            #[inline(always)]
            const fn test_get(&self, flag: B) -> bool {
                self._bit(flag as u8)
            }

            /// Returns a bit mask of all possible flags.
            #[inline(always)]
            const fn test_get_mask() -> u8 {
                let mut mask = 0;

                let mut i = 0;
                while i < B::iter().len() {
                    mask |= 1 << (B::iter()[i] as u8);

                    i += 1;
                }

                mask
            }

            /// Returns `true` if all flags are set.
            #[inline(always)]
            const fn test_get_all(&self) -> bool {
                (self.0 & Self::test_get_mask()) == Self::test_get_mask()
            }

            /// Returns `true` if any flag is set.
            #[inline(always)]
            const fn test_get_any(&self) -> bool {
                (self.0 & Self::test_get_mask()) != 0
            }

            /// Creates a copy of the bit field with the new value for the specified flag.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, flag: B, value: bool) -> Self {
                self._set_bit(flag as u8, value)
            }

            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                Self(self.0 | Self::test_get_mask())
            }

            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                Self(self.0 & !Self::test_get_mask())
            }
        });

        assert_accessor!(
            "8", "struct A(#[field(0, 1)] B);", quote::quote! {
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn test_get(&self) -> core::result::Result<B, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn test_get(&self) -> core::result::Result<B, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: B) -> Self {
                    self._set_field(0u8, 1u8, value as u8)
                }
            }
        );
    }

    #[test]
    fn accessor_field() {
        assert_accessor!("32", "struct A(A);", quote::quote! {
            /// Returns `true` if the specified `flag` is set.
            #[inline(always)]
            const fn test_get(&self, flag: A) -> bool {
                self._bit(flag as u8)
            }

            /// Returns a bit mask of all possible flags.
            #[inline(always)]
            const fn test_get_mask() -> u32 {
                let mut mask = 0;

                let mut i = 0;
                while i < A::iter().len() {
                    mask |= 1 << (A::iter()[i] as u32);

                    i += 1;
                }

                mask
            }

            /// Returns `true` if all flags are set.
            #[inline(always)]
            const fn test_get_all(&self) -> bool {
                (self.0 & Self::test_get_mask()) == Self::test_get_mask()
            }

            /// Returns `true` if any flag is set.
            #[inline(always)]
            const fn test_get_any(&self) -> bool {
                (self.0 & Self::test_get_mask()) != 0
            }

            /// Creates a copy of the bit field with the new value for the specified flag.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, flag: A, value: bool) -> Self {
                self._set_bit(flag as u8, value)
            }

            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                Self(self.0 | Self::test_get_mask())
            }

            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                Self(self.0 & !Self::test_get_mask())
            }
        });

        assert_accessor!(
            "32", "struct A(#[field(0, 1)] A);", quote::quote! {
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn test_get(&self) -> core::result::Result<A, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn test_get(&self) -> core::result::Result<A, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: A) -> Self {
                    self._set_field(0u8, 1u8, value as u32)
                }
            }
        );

        assert_accessor!(
            "32", "struct A(#[field(1, 9)] A);", quote::quote! {
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn test_get(&self) -> core::result::Result<A, u16> {
                    core::convert::TryFrom::try_from(self._field(1u8, 9u8) as u16)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn test_get(&self) -> core::result::Result<A, u16> {
                    core::convert::TryFrom::try_from(self._field(1u8, 9u8) as u16)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: A) -> Self {
                    self._set_field(1u8, 9u8, value as u32)
                }
            }
        );

        assert_accessor!("8", "struct A(#[field(2, 1)] bool);", quote::quote! {
            /// Gets the value of the field.
            #[inline(always)]
            const fn test_get(&self) -> bool {
                self._bit(2u8)
            }

            /// Creates a copy of the bit field with the new value.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, value: bool) -> Self {
                self._set_bit(2u8, value)
            }
        });

        assert_accessor!("8", "struct A(#[field(3, 2)] u8);", quote::quote! {
            /// Gets the value of the field.
            #[inline(always)]
            const fn test_get(&self) -> u8 {
                self._field(3u8, 2u8) as u8
            }

            /// Creates a copy of the bit field with the new value.
            ///
            /// Returns `None` if `value` is bigger than the specified amount of
            /// bits for the field can store.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, value: u8) -> Option<Self> {
                if value >= (1 as u8).wrapping_shl(2u8 as u32) { return None; }

                Some(self._set_field(3u8, 2u8, value as u8))
            }
        });

        assert_accessor!("16", "struct A(#[field(3, 8)] u8);", quote::quote! {
            /// Gets the value of the field.
            #[inline(always)]
            const fn test_get(&self) -> u8 {
                self._field(3u8, 8u8) as u8
            }

            /// Creates a copy of the bit field with the new value.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, value: u8) -> Self {
                self._set_field(3u8, 8u8, value as u16)
            }
        });
    }

    #[test]
    fn accessor_signed() {
        assert_accessor!(
            "32", "struct A(#[field(0, 8)] A);", quote::quote! {
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn test_get(&self) -> core::result::Result<A, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 8u8) as u8)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn test_get(&self) -> core::result::Result<A, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 8u8) as u8)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: A) -> Self {
                    self._set_field(0u8, 8u8, value as u32)
                }
            }
        );

        assert_accessor!(
            "32", "struct A(#[field(0, 8, signed)] A);", quote::quote! {
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn test_get(&self) -> core::result::Result<A, i8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 8u8) as i8)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn test_get(&self) -> core::result::Result<A, i8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 8u8) as i8)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: A) -> Self {
                    self._set_field(0u8, 8u8, value as u8 as u32)
                }
            }
        );

        assert_accessor!(
            "32", "struct A(u8);", quote::quote! {
                /// Gets the value of the field.
                #[inline(always)]
                const fn test_get(&self) -> u8 {
                    self._field(0u8, 8u8) as u8
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: u8) -> Self {
                    self._set_field(0u8, 8u8, value as u32)
                }
            }
        );

        assert_accessor!(
            "32", "struct A(i8);", quote::quote! {
                /// Gets the value of the field.
                #[inline(always)]
                const fn test_get(&self) -> i8 {
                    self._field(0u8, 8u8) as i8
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: i8) -> Self {
                    self._set_field(0u8, 8u8, value as u8 as u32)
                }
            }
        );
    }

    #[test]
    fn accessors() {
        assert_compare!(generate_accessors, "8", "struct A(B);", quote::quote! {
            impl A {
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                const fn has(&self, flag: B) -> bool {
                    self._bit(flag as u8)
                }

                /// Returns a bit mask of all possible flags.
                #[inline(always)]
                const fn has_mask() -> u8 {
                    let mut mask = 0;

                    let mut i = 0;
                    while i < B::iter().len() {
                        mask |= 1 << (B::iter()[i] as u8);

                        i += 1;
                    }

                    mask
                }

                /// Returns `true` if all flags are set.
                #[inline(always)]
                const fn has_all(&self) -> bool {
                    (self.0 & Self::has_mask()) == Self::has_mask()
                }

                /// Returns `true` if any flag is set.
                #[inline(always)]
                const fn has_any(&self) -> bool {
                    (self.0 & Self::has_mask()) != 0
                }

                /// Creates a copy of the bit field with the new value for the specified flag.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set(&self, flag: B, value: bool) -> Self {
                    self._set_bit(flag as u8, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_all(&self) -> Self {
                    Self(self.0 | Self::has_mask())
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_none(&self) -> Self {
                    Self(self.0 & !Self::has_mask())
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A(#[field(0, 1)] B);", quote::quote! {
            impl A {
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn get(&self) -> core::result::Result<B, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn get(&self) -> core::result::Result<B, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set(&self, value: B) -> Self {
                    self._set_field(0u8, 1u8, value as u8)
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A {}", quote::quote! {});

        assert_compare!(generate_accessors, "8", "struct A { b: B }", quote::quote! {
            impl A {
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                const fn b(&self, flag: B) -> bool {
                    self._bit(flag as u8)
                }

                /// Returns a bit mask of all possible flags.
                #[inline(always)]
                const fn b_mask() -> u8 {
                    let mut mask = 0;

                    let mut i = 0;
                    while i < B::iter().len() {
                        mask |= 1 << (B::iter()[i] as u8);

                        i += 1;
                    }

                    mask
                }

                /// Returns `true` if all flags are set.
                #[inline(always)]
                const fn b_all(&self) -> bool {
                    (self.0 & Self::b_mask()) == Self::b_mask()
                }

                /// Returns `true` if any flag is set.
                #[inline(always)]
                const fn b_any(&self) -> bool {
                    (self.0 & Self::b_mask()) != 0
                }

                /// Creates a copy of the bit field with the new value for the specified flag.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b(&self, flag: B, value: bool) -> Self {
                    self._set_bit(flag as u8, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_all(&self) -> Self {
                    Self(self.0 | Self::b_mask())
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_none(&self) -> Self {
                    Self(self.0 & !Self::b_mask())
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A {#[field(0, 1)] b: B}", quote::quote! {
            impl A {
                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn b(&self) -> core::result::Result<B, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn b(&self) -> core::result::Result<B, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b(&self, value: B) -> Self {
                    self._set_field(0u8, 1u8, value as u8)
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A {b: B, #[field(0, 1)] c: C}", quote::quote! {
            impl A {
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                const fn b(&self, flag: B) -> bool {
                    self._bit(flag as u8)
                }

                /// Returns a bit mask of all possible flags.
                #[inline(always)]
                const fn b_mask() -> u8 {
                    let mut mask = 0;

                    let mut i = 0;
                    while i < B::iter().len() {
                        mask |= 1 << (B::iter()[i] as u8);

                        i += 1;
                    }

                    mask
                }

                /// Returns `true` if all flags are set.
                #[inline(always)]
                const fn b_all(&self) -> bool {
                    (self.0 & Self::b_mask()) == Self::b_mask()
                }

                /// Returns `true` if any flag is set.
                #[inline(always)]
                const fn b_any(&self) -> bool {
                    (self.0 & Self::b_mask()) != 0
                }

                /// Creates a copy of the bit field with the new value for the specified flag.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b(&self, flag: B, value: bool) -> Self {
                    self._set_bit(flag as u8, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_all(&self) -> Self {
                    Self(self.0 | Self::b_mask())
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_none(&self) -> Self {
                    Self(self.0 & !Self::b_mask())
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(const_trait_impl)]
                const fn c(&self) -> core::result::Result<C, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                /// not be converted to the expected type.
                #[inline(always)]
                #[cfg(not(const_trait_impl))]
                fn c(&self) -> core::result::Result<C, u8> {
                    core::convert::TryFrom::try_from(self._field(0u8, 1u8) as u8)
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_c(&self, value: C) -> Self {
                    self._set_field(0u8, 1u8, value as u8)
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A(#[field(0, 1)] u8);", quote::quote! {
            impl A {
                /// Gets the value of the field.
                #[inline(always)]
                const fn get(&self) -> u8 {
                    self._field(0u8, 1u8) as u8
                }

                /// Creates a copy of the bit field with the new value.
                ///
                /// Returns `None` if `value` is bigger than the specified amount of
                /// bits for the field can store.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set(&self, value: u8) -> Option<Self> {
                    if value >= (1 as u8).wrapping_shl(1u8 as u32) { return None; }

                    Some(self._set_field(0u8, 1u8, value as u8))
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A { #[field(0, 1)] b: u8 }", quote::quote! {
            impl A {
                /// Gets the value of the field.
                #[inline(always)]
                const fn b(&self) -> u8 {
                    self._field(0u8, 1u8) as u8
                }

                /// Creates a copy of the bit field with the new value.
                ///
                /// Returns `None` if `value` is bigger than the specified amount of
                /// bits for the field can store.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b(&self, value: u8) -> Option<Self> {
                    if value >= (1 as u8).wrapping_shl(1u8 as u32) { return None; }

                    Some(self._set_field(0u8, 1u8, value as u8))
                }
            }
        });
    }

    #[test]
    fn accessors_low() {
        assert_compare!(generate_accessors_low, "8", "struct A(B);", quote::quote! {
            impl A {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0 >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0 & !(1 << position);

                    Self(cleared | ((value as u8) << position))
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u8 {
                    let shifted = self.0 >> position;

                    let rest = size as u8 % (core::mem::size_of::<u8>() * 8) as u8;
                    let bit = (rest > 0) as u8;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as u8);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u8) -> Self {
                    let rest = size as u8 % (core::mem::size_of::<u8>() * 8) as u8;
                    let bit = (rest > 0) as u8;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as u8);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });

        assert_compare!(generate_accessors_low, "16", "struct B(C);", quote::quote! {
            impl B {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0 >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0 & !(1 << position);

                    Self(cleared | ((value as u16) << position))
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u16 {
                    let shifted = self.0 >> position;

                    let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                    let bit = (rest > 0) as u16;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as u16);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u16) -> Self {
                    let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                    let bit = (rest > 0) as u16;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as u16);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });

        assert_compare!(generate_accessors_low, "32", "struct C(D);", quote::quote! {
            impl C {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0 >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0 & !(1 << position);

                    Self(cleared | ((value as u32) << position))
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u32 {
                    let shifted = self.0 >> position;

                    let rest = size as u32 % (core::mem::size_of::<u32>() * 8) as u32;
                    let bit = (rest > 0) as u32;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as u32);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u32) -> Self {
                    let rest = size as u32 % (core::mem::size_of::<u32>() * 8) as u32;
                    let bit = (rest > 0) as u32;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as u32);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });

        assert_compare!(generate_accessors_low, "64", "struct D(E);", quote::quote! {
            impl D {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0 >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0 & !(1 << position);

                    Self(cleared | ((value as u64) << position))
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u64 {
                    let shifted = self.0 >> position;

                    let rest = size as u64 % (core::mem::size_of::<u64>() * 8) as u64;
                    let bit = (rest > 0) as u64;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as u64);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u64) -> Self {
                    let rest = size as u64 % (core::mem::size_of::<u64>() * 8) as u64;
                    let bit = (rest > 0) as u64;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as u64);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });

        assert_compare!(generate_accessors_low, "128", "struct E(F);", quote::quote! {
            impl E {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0 >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0 & !(1 << position);

                    Self(cleared | ((value as u128) << position))
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u128 {
                    let shifted = self.0 >> position;

                    let rest = size as u128 % (core::mem::size_of::<u128>() * 8) as u128;
                    let bit = (rest > 0) as u128;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as u128);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u128) -> Self {
                    let rest = size as u128 % (core::mem::size_of::<u128>() * 8) as u128;
                    let bit = (rest > 0) as u128;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as u128);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });
    }

    #[test]
    fn assertions() {
        assert_compare!(generate_assertions, "8", "struct A {}", quote::quote! {});

        let check_1 = quote::quote! {
            impl A {
                const _TYPE_IN_FIELD_0_IS_SMALLER_THAN_THE_SPECIFIED_SIZE_OF_9_BITS: [(); 0 - !{
                    const ASSERT: bool = core::mem::size_of::<B>() * 8 >= 9;
                    ASSERT
                } as usize] = [];

                const _SIGNED_TYPE_IN_FIELD_0_CAN_NEVER_BE_NEGATIVE: [(); 0 - !{
                    const ASSERT: bool = !B::is_signed() || core::mem::size_of::<B>() * 8 == 9;
                    ASSERT
                } as usize] = [];
            }
        };
        assert_compare!(generate_assertions, "16", "struct A(#[field(0, 9)] B);", check_1);
        assert_compare!(
            generate_assertions, "16, allow_overlaps", "struct A(#[field(0, 9)] B);", check_1
        );

        let check_2 = quote::quote! {
            impl A {
                const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                    const ASSERT: bool = core::mem::size_of::<B>() == 1;
                    ASSERT
                } as usize] = [];

                const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                    const ASSERT: bool = core::mem::size_of::<u16>() * 8 > B::max() as usize;
                    ASSERT
                } as usize] = [];
            }
        };
        assert_compare!(generate_assertions, "16", "struct A(B);", check_2);
        assert_compare!(generate_assertions, "16, allow_overlaps", "struct A(B);", check_2);

        let check_3 = quote::quote! {
            const _TYPE_IN_FIELD_0_IS_SMALLER_THAN_THE_SPECIFIED_SIZE_OF_2_BITS: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<B>() * 8 >= 2;
                ASSERT
            } as usize] = [];

            const _SIGNED_TYPE_IN_FIELD_0_CAN_NEVER_BE_NEGATIVE: [(); 0 - !{
                const ASSERT: bool = !B::is_signed() || core::mem::size_of::<B>() * 8 == 2;
                ASSERT
            } as usize] = [];

            const _FLAGS_IN_FIELD_1_MUST_BE_REPR_U8: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<C>() == 1;
                ASSERT
            } as usize] = [];

            const _FLAGS_IN_FIELD_1_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<u8>() * 8 > C::max() as usize;
                ASSERT
            } as usize] = [];
        };
        assert_compare!(
            generate_assertions, "8", "struct A { #[field(0, 2)] b: B, c: C }", quote::quote! {
                impl A {
                    #check_3

                    const fn _flags_in_field_1_overlap_with_field_0() -> bool {
                        let flags = C::iter();

                        let mut i = 0;
                        while i < flags.len () {
                            let flag = flags[i] as u8;
                            if flag >= 0u8 && flag < 0u8 + 2u8 { return true; }

                            i += 1;
                        }

                        false
                    }

                    const _FLAGS_IN_FIELD_1_OVERLAP_WITH_FIELD_0: [();
                        0 - Self::_flags_in_field_1_overlap_with_field_0() as usize
                    ] = [];
                }
            }
        );
        assert_compare!(
            generate_assertions, "8, allow_overlaps", "struct A { #[field(0, 2)] b: B, c: C }",
            quote::quote! { impl A { #check_3 } }
        );

        let check_4 = quote::quote! {
            const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<B>() == 1;
                ASSERT
            } as usize] = [];

            const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<u8>() * 8 > B::max() as usize;
                ASSERT
            } as usize] = [];
        };
        let check_5 = quote::quote! {
            const _FLAGS_IN_FIELD_1_MUST_BE_REPR_U8: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<C>() == 1;
                ASSERT
            } as usize] = [];

            const _FLAGS_IN_FIELD_1_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<u8>() * 8 > C::max() as usize;
                ASSERT
            } as usize] = [];
        };
        assert_compare!(
            generate_assertions, "8", "struct A { b: B, c: C }", quote::quote! {
                impl A {
                    #check_4

                    const fn _flags_in_field_0_overlap_with_flags_in_field_1() -> bool {
                        let f1 = B::iter();
                        let f2 = C::iter();

                        let mut i1 = 0;
                        while i1 < f1.len() {
                            let mut i2 = 0;
                            while i2 < f2.len() {
                                if (f1[i1] as u32) == (f2[i2] as u32) { return true; }

                                i2 += 1;
                            }

                            i1 += 1;
                        }

                        false
                    }

                    const _FLAGS_IN_FIELD_0_OVERLAP_WITH_FLAGS_IN_FIELD_1: [();
                        0 - Self::_flags_in_field_0_overlap_with_flags_in_field_1() as usize
                    ] = [];

                    #check_5
                }
            }
        );
        assert_compare!(
            generate_assertions, "8, allow_overlaps", "struct A { b: B, c: C }", quote::quote! {
                impl A {
                    #check_4
                    #check_5
                }
            }
        );

        assert_compare!(
            generate_assertions, "size", "struct A(B);", quote::quote! {
                impl A {
                    const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<B>() == 1;
                        ASSERT
                    } as usize] = [];

                    const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<usize>() * 8 > B::max() as usize;
                        ASSERT
                    } as usize] = [];
                }
            }
        );
        assert_compare!(
            generate_assertions, "size", "struct A(#[field(4, 3)] B);", quote::quote! {
                impl A {
                    const _TYPE_IN_FIELD_0_IS_SMALLER_THAN_THE_SPECIFIED_SIZE_OF_3_BITS: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<B>() * 8 >= 3;
                        ASSERT
                    } as usize] = [];

                    const _SIGNED_TYPE_IN_FIELD_0_CAN_NEVER_BE_NEGATIVE: [(); 0 - !{
                        const ASSERT: bool = !B::is_signed() || core::mem::size_of::<B>() * 8 == 3;
                        ASSERT
                    } as usize] = [];

                    const _FIELD_0_EXCEEDS_THE_BIT_FIELD_SIZE: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<usize>() * 8 >= 4 + 3;
                        ASSERT
                    } as usize] = [];

                    const _FIELD_0_HAS_THE_SIZE_OF_THE_WHOLE_BIT_FIELD: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<usize>() * 8 != 3;
                        ASSERT
                    } as usize] = [];
                }
            }
        );
    }

    #[test]
    fn debug() {
        assert_compare!(generate_debug, "8", "struct A(B);", quote::quote! {});

        assert_compare!(generate_debug, "8", "#[derive(Debug)] struct A(B);", quote::quote! {
            impl core::fmt::Debug for A {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut s = f.debug_struct(core::stringify!(A));

                    for flag in <B>::iter() {
                        s.field(&alloc::format!("{:?}", flag), &self.has(*flag));
                    }

                    s.finish()
                }
            }
        });

        let b_debug = quote::quote! {
            impl core::fmt::Debug for A {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut s = f.debug_struct(core::stringify!(A));

                    let value = self.get();

                    if let core::result::Result::Ok(value) = value {
                        s.field(core::stringify!(B), &value);
                    } else {
                        s.field(core::stringify!(B), &value);
                    }

                    s.finish()
                }
            }
        };
        assert_compare!(
            generate_debug, "8", "#[derive(Debug)] struct A(#[field(0, 1)] B);", &b_debug
        );
        assert_compare!(
            generate_debug, "8", "#[derive(Debug)] struct A(#[field(0, 1)] super::B);", &b_debug
        );

        assert_compare!(generate_debug, "8", "#[derive(Debug)] struct A {}", quote::quote! {
            impl core::fmt::Debug for A {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut s = f.debug_struct(core::stringify!(A));

                    s.finish()
                }
            }
        });

        assert_compare!(
            generate_debug, "8", "#[derive(Debug)] struct A { #[field(0, 1)] b: B }",
            quote::quote! {
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        let value = self.b();

                        if let core::result::Result::Ok(value) = value {
                            s.field(core::stringify!(b), &value);
                        } else {
                            s.field(core::stringify!(b), &value);
                        }

                        s.finish()
                    }
                }
            }
        );

        assert_compare!(
            generate_debug, "8", "#[derive(Debug)] struct A { b: super::B }",
            quote::quote! {
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        {
                            struct BitFieldDebugImplementor<'a>(&'a A);

                            impl<'a> core::fmt::Debug for BitFieldDebugImplementor<'a> {
                                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                                    let mut s = f.debug_struct(core::stringify!(B));

                                    for flag in <super::B>::iter() {
                                        s.field(&alloc::format!("{:?}", flag), &self.0.b(*flag));
                                    }

                                    s.finish()
                                }
                            }

                            s.field(core::stringify!(b), &BitFieldDebugImplementor(&self));
                        }

                        s.finish()
                    }
                }
            }
        );

        assert_compare!(
            generate_debug, "8", "#[derive(Debug)] struct A { b: B, #[field(0, 1)] c: C, d: D }",
            quote::quote! {
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        {
                            struct BitFieldDebugImplementor<'a>(&'a A);

                            impl<'a> core::fmt::Debug for BitFieldDebugImplementor<'a> {
                                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                                    let mut s = f.debug_struct(core::stringify!(B));

                                    for flag in <B>::iter() {
                                        s.field(&alloc::format!("{:?}", flag), &self.0.b(*flag));
                                    }

                                    s.finish()
                                }
                            }

                            s.field(core::stringify!(b), &BitFieldDebugImplementor(&self));
                        }

                        let value = self.c();

                        if let core::result::Result::Ok(value) = value {
                            s.field(core::stringify!(c), &value);
                        } else {
                            s.field(core::stringify!(c), &value);
                        }

                        {
                            struct BitFieldDebugImplementor<'a>(&'a A);

                            impl <'a> core::fmt::Debug for BitFieldDebugImplementor<'a> {
                                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                                    let mut s = f.debug_struct(core::stringify!(D));

                                    for flag in <D>::iter() {
                                        s.field(&alloc::format!("{:?}", flag), &self.0.d(*flag));
                                    }

                                    s.finish()
                                }
                            }

                            s.field(core::stringify!(d), &BitFieldDebugImplementor(&self));
                        }

                        s.finish()
                    }
                }
            }
        );

        assert_compare!(
            generate_debug, "8", "#[derive(Debug)] struct A { b: B, c: C }",
            quote::quote! {
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        {
                            struct BitFieldDebugImplementor<'a>(&'a A);

                            impl<'a> core::fmt::Debug for BitFieldDebugImplementor<'a> {
                                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                                    let mut s = f.debug_struct(core::stringify!(B));

                                    for flag in <B>::iter() {
                                        s.field(&alloc::format!("{:?}", flag), &self.0.b(*flag));
                                    }

                                    s.finish()
                                }
                            }

                            s.field(core::stringify!(b), &BitFieldDebugImplementor(&self));
                        }

                        {
                            struct BitFieldDebugImplementor<'a>(&'a A);

                            impl <'a> core::fmt::Debug for BitFieldDebugImplementor<'a> {
                                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                                    let mut s = f.debug_struct(core::stringify!(C));

                                    for flag in <C>::iter() {
                                        s.field(&alloc::format!("{:?}", flag), &self.0.c(*flag));
                                    }

                                    s.finish()
                                }
                            }

                            s.field(core::stringify!(c), &BitFieldDebugImplementor(&self));
                        }

                        s.finish()
                    }
                }
            }
        );

        assert_compare!(
            generate_debug, "8", "#[derive(Debug)] struct A(#[field(0, 1)] bool);",
            quote::quote! {
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        let value = self.get();
                        s.field(core::stringify!(bool), &value);

                        s.finish()
                    }
                }
            }
        );

        assert_compare!(
            generate_debug, "32", "#[derive(Debug)] struct A(#[field(0, 16)] u16);",
            quote::quote! {
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        let value = self.get();
                        s.field(core::stringify!(u16), &value);

                        s.finish()
                    }
                }
            }
        );

        assert_compare!(
            generate_debug, "8", "#[derive(Debug)] struct A { #[field(2, 1)] b: bool }",
            quote::quote! {
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        let value = self.b();
                        s.field(core::stringify!(b), &value);

                        s.finish()
                    }
                }
            }
        );

        assert_compare!(
            generate_debug, "32", "#[derive(Debug)] struct A { #[field(0, 16)] b: u16 }",
            quote::quote! {
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        let value = self.b();
                        s.field(core::stringify!(b), &value);

                        s.finish()
                    }
                }
            }
        );
    }

    #[test]
    fn display() {
        assert_compare!(generate_display, "8", "struct A(B);", quote::quote! {});

        assert_compare!(generate_display, "8", "#[derive(Display)] struct A(B);", quote::quote! {
            impl core::fmt::Display for A {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let mut flags = alloc::vec::Vec::new();

                    for flag in <B>::iter() {
                        if self.has(*flag) {
                            flags.push(alloc::format!("{:?}", flag));
                        }
                    }

                    let flags = flags.join(" | ");

                    f.write_str(
                        if flags.len() > 0 { &flags } else { "-" }
                    )
                }
            }
        });

        assert_compare!(
            generate_display, "8", "#[derive(Display)] struct A(#[field(0, 1)] B);", quote::quote! {
                impl core::fmt::Display for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let value = self.get();
                        if let core::result::Result::Ok(value) = value {
                            f.write_str(&alloc::format!("{:?}", value))
                        } else {
                            f.write_str(&alloc::format!("{:?}", value))
                        }
                    }
                }
            }
        );

        assert_compare!(
            generate_display, "8", "#[derive(Display)] struct A { #[field(0, 1)] b: B }",
            quote::quote! {
                impl core::fmt::Display for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let value = self.b();

                        if let core::result::Result::Ok(value) = value {
                            f.write_str(&alloc::format!("{:?}", value))
                        } else {
                            f.write_str(&alloc::format!("{:?}", value))
                        }
                    }
                }
            }
        );

        assert_compare!(
            generate_display, "8", "#[derive(Display)] struct A { b: super::B }", quote::quote! {
                impl core::fmt::Display for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut flags = alloc::vec::Vec::new();

                        for flag in <super::B>::iter() {
                            if self.b(*flag) {
                                flags.push(alloc::format!("{:?}", flag));
                            }
                        }

                        let flags = flags.join(" | ");

                        f.write_str(
                            if flags.len() > 0 { &flags } else { "-" }
                        )
                    }
                }
            }
        );

        assert_compare!(
            generate_display, "8", "#[derive(Display)] struct A { b: B, c: C }", quote::quote! {
                impl core::fmt::Display for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut flags = alloc::vec::Vec::new();

                        for flag in <B>::iter() {
                            if self.b(*flag) {
                                flags.push(alloc::format!(
                                    "{}::{:?}", core::any::type_name::<B>(), flag
                                ));
                            }
                        }

                        for flag in <C>::iter() {
                            if self.c(*flag) {
                                flags.push(alloc::format!(
                                    "{}::{:?}", core::any::type_name::<C>(), flag
                                ));
                            }
                        }

                        let flags = flags.join(" | ");

                        f.write_str(
                            if flags.len() > 0 { &flags } else { "-" }
                        )
                    }
                }
            }
        );

        assert_compare!(
            generate_display, "8", "#[derive(Display)] struct A(#[field(0, 1)] bool);",
            quote::quote! {
                impl core::fmt::Display for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let value = self.get();
                        f.write_str(&alloc::format!("{:?}", value))
                    }
                }
            }
        );

        assert_compare!(
            generate_display, "32", "#[derive(Display)] struct A(#[field(0, 16)] u16);",
            quote::quote! {
                impl core::fmt::Display for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let value = self.get();
                        f.write_str(&alloc::format!("{:?}", value))
                    }
                }
            }
        );

        assert_compare!(
            generate_display, "8", "#[derive(Display)] struct A { #[field(0, 1)] b: bool }",
            quote::quote! {
                impl core::fmt::Display for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let value = self.b();
                        f.write_str(&alloc::format!("{:?}", value))
                    }
                }
            }
        );

        assert_compare!(
            generate_display, "32", "#[derive(Display)] struct A { #[field(0, 16)] b: u16 }",
            quote::quote! {
                impl core::fmt::Display for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let value = self.b();
                        f.write_str(&alloc::format!("{:?}", value))
                    }
                }
            }
        );
    }

    #[test]
    #[should_panic]
    fn display_empty() {
        assert_compare!(generate_display, "8", "#[derive(Display)] struct A {}", quote::quote! {});
    }

    #[test]
    #[should_panic]
    fn display_non_flags() {
        assert_compare!(
            generate_display, "8", "#[derive(Display)] struct A { b: B, #[field(0, 1)] c: C }",
            quote::quote! {}
        );
    }

    #[test]
    fn implementation() {
        assert_compare!(generate_impl, "8", "struct A(A);", quote::quote! {
            impl A {
                /// Creates a new instance with all flags and fields cleared.
                #[inline(always)]
                const fn new() -> Self {
                    Self(0)
                }
            }
        });
    }

    #[test]
    fn struct_bit() {
        assert_compare!(generate_struct, "8", "struct A(A);", quote::quote! {
            #[repr(C)]
            struct A(u8);
        });

        assert_compare!(generate_struct, "16", "struct A(A);", quote::quote! {
            #[repr(C)]
            struct A(u16);
        });
    }

    #[test]
    fn struct_attrs() {
        assert_compare!(
            generate_struct, "8", "#[some_attribute1] #[some_attribute2] struct A(A);",
            quote::quote! {
                #[repr(C)]
                #[some_attribute1]
                #[some_attribute2]
                struct A(u8);
            }
        );
    }

    #[test]
    fn struct_vis() {
        assert_compare!(generate_struct, "8", "struct A(A);", quote::quote! {
            #[repr(C)]
            struct A(u8);
        });

        assert_compare!(generate_struct, "8", "pub struct A(A);", quote::quote! {
            #[repr(C)]
            pub struct A(u8);
        });
    }

    #[test]
    fn struct_ident() {
        assert_compare!(generate_struct, "8", "struct A(A);", quote::quote! {
            #[repr(C)]
            struct A(u8);
        });

        assert_compare!(generate_struct, "8", "struct B(A);", quote::quote! {
            #[repr(C)]
            struct B(u8);
        });
    }

    #[test]
    fn everything() {
        assert_eq!(
            Into::<proc_macro2::TokenStream>::into(parse_valid!(
                "16", "/** D1 */ #[derive(Debug)] pub(crate) struct A { /** D2 */ pub(crate) b: B, /** D3 */ #[field(7, 3)] pub c: C, /** D4 */ d: D }"
            )).to_string(),
            quote::quote! {
                // field
                #[repr(C)]
                #[doc = " D1 "]
                pub(crate) struct A(u16);

                // implementation
                impl A {
                    /// Creates a new instance with all flags and fields cleared.
                    #[inline(always)]
                    pub(crate) const fn new() -> Self {
                        Self(0)
                    }
                }

                // accessors_low
                impl A {
                    /// Returns a boolean value whether the specified flag is set.
                    #[inline(always)]
                    const fn _bit(&self, position: u8) -> bool {
                        ((self.0 >> position) & 1) != 0
                    }

                    /// Returns a modified instance with the flag set to the specified value.
                    #[inline(always)]
                    const fn _set_bit(&self, position: u8, value: bool) -> Self {
                        let cleared = self.0 & !(1 << position);
                        Self(cleared | ((value as u16) << position))
                    }

                    /// Returns a field (subset of bits) from the internal value.
                    #[inline(always)]
                    const fn _field(&self, position: u8, size: u8) -> u16 {
                        let shifted = self.0 >> position;

                        let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                        let bit = (rest > 0) as u16;

                        let limit = bit.wrapping_shl(rest as u32);
                        let mask = limit.wrapping_sub((size > 0) as u16);
                        let result = shifted & mask;

                        result
                    }

                    /// Returns a modified variant with the field set to the specified value.
                    #[inline(always)]
                    const fn _set_field(&self, position: u8, size: u8, value: u16) -> Self {
                        let rest = size as u16 % (core::mem::size_of::<u16> () * 8) as u16;
                        let bit = (rest > 0) as u16;

                        let limit = bit.wrapping_shl(rest as u32);
                        let negative_mask = limit.wrapping_sub((size > 0) as u16);
                        let positioned_used_bits = negative_mask << position;
                        let positioned_mask = !positioned_used_bits;
                        let cleared = self.0 & positioned_mask;

                        let shifted_value = value << position;

                        let result = cleared | shifted_value;

                        Self(result)
                    }
                }

                // accessors
                impl A {
                    #[doc = " D2 "]
                    /// Returns `true` if the specified `flag` is set.
                    #[inline(always)]
                    pub(crate) const fn b(&self, flag: B) -> bool {
                        self._bit(flag as u8)
                    }

                    #[doc = " D2 "]
                    /// Returns a bit mask of all possible flags.
                    #[inline(always)]
                    pub(crate) const fn b_mask() -> u16 {
                        let mut mask = 0;

                        let mut i = 0;
                        while i < B::iter().len() {
                            mask |= 1 << (B::iter()[i] as u16);

                            i += 1;
                        }

                        mask
                    }

                    #[doc = " D2 "]
                    /// Returns `true` if all flags are set.
                    #[inline(always)]
                    pub(crate) const fn b_all(&self) -> bool {
                        (self.0 & Self::b_mask()) == Self::b_mask()
                    }

                    #[doc = " D2 "]
                    /// Returns `true` if any flag is set.
                    #[inline(always)]
                    pub(crate) const fn b_any(&self) -> bool {
                        (self.0 & Self::b_mask()) != 0
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with the new value for the specified flag.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn set_b(&self, flag: B, value: bool) -> Self {
                        self._set_bit(flag as u8, value)
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with all flags set.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn set_b_all(&self) -> Self {
                        Self(self.0 | Self::b_mask())
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with all flags cleared.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn set_b_none(&self) -> Self {
                        Self(self.0 & !Self::b_mask())
                    }

                    #[doc = " D3 "]
                    /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                    /// not be converted to the expected type.
                    #[inline(always)]
                    #[cfg(const_trait_impl)]
                    pub const fn c(&self) -> core::result::Result<C, u8> {
                        core::convert::TryFrom::try_from(self._field(7u8, 3u8) as u8)
                    }

                    #[doc = " D3 "]
                    /// Returns the primitive value encapsulated in the `Err` variant, if the value can
                    /// not be converted to the expected type.
                    #[inline(always)]
                    #[cfg(not(const_trait_impl))]
                    pub fn c(&self) -> core::result::Result<C, u8> {
                        core::convert::TryFrom::try_from(self._field(7u8, 3u8) as u8)
                    }

                    #[doc = " D3 "]
                    /// Creates a copy of the bit field with the new value.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub const fn set_c(&self, value: C) -> Self {
                        self._set_field(7u8, 3u8, value as u16)
                    }

                    #[doc = " D4 "]
                    /// Returns `true` if the specified `flag` is set.
                    #[inline(always)]
                    const fn d(&self, flag: D) -> bool {
                        self._bit(flag as u8)
                    }

                    #[doc = " D4 "]
                    /// Returns a bit mask of all possible flags.
                    #[inline(always)]
                    const fn d_mask() -> u16 {
                        let mut mask = 0;

                        let mut i = 0;
                        while i < D::iter().len() {
                            mask |= 1 << (D::iter()[i] as u16);

                            i += 1;
                        }

                        mask
                    }

                    #[doc = " D4 "]
                    /// Returns `true` if all flags are set.
                    #[inline(always)]
                    const fn d_all(&self) -> bool {
                        (self.0 & Self::d_mask()) == Self::d_mask()
                    }

                    #[doc = " D4 "]
                    /// Returns `true` if any flag is set.
                    #[inline(always)]
                    const fn d_any(&self) -> bool {
                        (self.0 & Self::d_mask()) != 0
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with the new value for the specified flag.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn set_d(&self, flag: D, value: bool) -> Self {
                        self._set_bit(flag as u8, value)
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with all flags set.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn set_d_all(&self) -> Self {
                        Self(self.0 | Self::d_mask())
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with all flags cleared.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn set_d_none(&self) -> Self {
                        Self(self.0 & !Self::d_mask())
                    }
                }

                // assertions
                impl A {
                    const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<B>() == 1; ASSERT
                    } as usize] = [];

                    const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<u16>() * 8 > B::max() as usize;
                        ASSERT
                    } as usize] = [];

                    const fn _flags_in_field_0_overlap_with_field_1() -> bool {
                        let flags = B::iter();

                        let mut i = 0;
                        while i < flags.len() {
                            let flag = flags[i] as u8;
                            if flag >= 7u8 && flag < 7u8 + 3u8 {
                                return true;
                            }

                            i += 1;
                        }

                        false
                    }

                    const _FLAGS_IN_FIELD_0_OVERLAP_WITH_FIELD_1: [();
                        0 - Self::_flags_in_field_0_overlap_with_field_1() as usize
                    ] = [];

                    const fn _flags_in_field_0_overlap_with_flags_in_field_2() -> bool {
                        let f1 = B::iter();
                        let f2 = D::iter();

                        let mut i1 = 0;
                        while i1 < f1.len() {
                            let mut i2 = 0; while i2 < f2.len() {
                                if (f1 [i1] as u32) == (f2 [i2] as u32) {
                                    return true;
                                }

                                i2 += 1;
                            }

                            i1 += 1;
                        }

                        false
                    }

                    const _FLAGS_IN_FIELD_0_OVERLAP_WITH_FLAGS_IN_FIELD_2: [();
                        0 - Self::_flags_in_field_0_overlap_with_flags_in_field_2() as usize
                    ] = [];

                    const _TYPE_IN_FIELD_1_IS_SMALLER_THAN_THE_SPECIFIED_SIZE_OF_3_BITS: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<C>() * 8 >= 3; ASSERT
                    } as usize] = [];

                    const _SIGNED_TYPE_IN_FIELD_1_CAN_NEVER_BE_NEGATIVE: [(); 0 - !{
                        const ASSERT: bool = !C::is_signed() || core::mem::size_of::<C>() * 8 == 3;
                        ASSERT
                    } as usize] = [];

                    const _FLAGS_IN_FIELD_2_MUST_BE_REPR_U8: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<D>() == 1; ASSERT
                    } as usize] = [];

                    const _FLAGS_IN_FIELD_2_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<u16>() * 8 > D::max() as usize;
                        ASSERT
                    } as usize] = [];

                    const fn _flags_in_field_2_overlap_with_field_1() -> bool {
                        let flags = D::iter();

                        let mut i = 0;
                        while i < flags.len() {
                            let flag = flags[i] as u8;
                            if flag >= 7u8 && flag < 7u8 + 3u8 {
                                return true;
                            }

                            i += 1;
                        }

                        false
                    }

                    const _FLAGS_IN_FIELD_2_OVERLAP_WITH_FIELD_1: [();
                        0 - Self::_flags_in_field_2_overlap_with_field_1() as usize
                    ] = [];
                }

                // debug
                impl core::fmt::Debug for A {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        let mut s = f.debug_struct(core::stringify!(A));

                        {
                            struct BitFieldDebugImplementor<'a>(&'a A);

                            impl<'a> core::fmt::Debug for BitFieldDebugImplementor<'a> {
                                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                                    let mut s = f.debug_struct(core::stringify!(B));

                                    for flag in <B>::iter() {
                                        s.field(&alloc::format!("{:?}", flag), &self.0.b(*flag));
                                    }

                                    s.finish()
                                }
                            }

                            s.field(core::stringify!(b), &BitFieldDebugImplementor(&self));
                        }

                        let value = self.c();
                        if let core::result::Result::Ok(value) = value {
                            s.field(core::stringify!(c), &value);
                        } else {
                            s.field(core::stringify!(c), &value);
                        }

                        {
                            struct BitFieldDebugImplementor<'a>(&'a A);

                            impl<'a> core::fmt::Debug for BitFieldDebugImplementor<'a> {
                                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                                    let mut s = f.debug_struct(core::stringify!(D));

                                    for flag in <D>::iter() {
                                        s.field(&alloc::format!("{:?}", flag), &self.0.d(*flag));
                                    }

                                    s.finish()
                                }
                            }

                            s.field(core::stringify!(d), &BitFieldDebugImplementor(&self));
                        }

                        s.finish()
                    }
                }
            }.to_string()
        );
    }
}