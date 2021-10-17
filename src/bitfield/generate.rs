//! Contains code to generate bit fields.

use syn::spanned::Spanned;

impl super::BitField {
    /// Returns (as truthfully as possible):
    /// - `-1` if `left` is less visible than `right`.
    /// - `0` if `left` and `right` have the same visibility.
    /// - `1` if `left` is more visible than `right`.
    ///
    /// If `left` is `pub(in some::module)`, then `-1` is returned, except for when `right` is also
    /// `pub(in some::module)`. Otherwise, paths are not compared to each other.
    ///
    /// See https://doc.rust-lang.org/reference/visibility-and-privacy.html
    fn cmp_vis(left: &syn::Visibility, right: &syn::Visibility) -> i8 {
        match left {
            syn::Visibility::Public(_) => match right {
                syn::Visibility::Public(_) => 0,

                syn::Visibility::Crate(_) |
                syn::Visibility::Restricted(_) |
                syn::Visibility::Inherited => 1
            },

            syn::Visibility::Crate(_) => match right {
                syn::Visibility::Public(_) => -1,

                syn::Visibility::Crate(_) => 0,

                syn::Visibility::Restricted(right) => {
                    if let Some(right) = right.path.get_ident() {
                        if right == "crate" { return 0; }
                    }

                    1
                },

                syn::Visibility::Inherited => 1
            },

            syn::Visibility::Restricted(left) => match right {
                syn::Visibility::Public(_) => -1,

                syn::Visibility::Crate(_) => {
                    if let Some(left) = left.path.get_ident() {
                        if left == "crate" { return 0; }
                    }

                    -1
                },

                syn::Visibility::Restricted(right) => {
                    // Compare non-complex paths.
                    if let Some(left) = left.path.get_ident() {
                        if let Some(right) = right.path.get_ident() {
                            if left == "self" {
                                if right == "self" { return 0; }
                            } else if left == "super" {
                                if right == "self" { return 1; }
                                else if right == "super" { return 0; }
                            } else if left == "crate" {
                                if right == "crate" { return 0; }
                                return 1;
                            }
                        } else if left == "crate" {
                            return 1;
                        }
                    } else {
                        let left = &left.path;
                        let right = &right.path;
                        if quote::quote!(#left).to_string() == quote::quote!(#right).to_string() {
                            return 0;
                        }
                    }

                    // Complex paths, or `vis(left) < vis(right)`.
                    -1
                },

                syn::Visibility::Inherited => {
                    if let Some(left) = left.path.get_ident() {
                        if left == "self" {
                            return 0;
                        }
                    }

                    1
                }
            },

            syn::Visibility::Inherited => match right {
                syn::Visibility::Public(_) |
                syn::Visibility::Crate(_) => -1,

                syn::Visibility::Restricted(right) => {
                    if let Some(right) = right.path.get_ident() {
                        if right == "self" {
                            return 0;
                        }
                    }

                    -1
                }

                syn::Visibility::Inherited => 0
            }
        }
    }

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
        setter: &syn::Ident,
        inverter: &syn::Ident
    ) -> proc_macro2::TokenStream {
        let attrs = &entry.attrs;
        let vis = &entry.vis;
        let ty = &entry.ty;

        if let Some(field) = &entry.field {
            let bit = field.bit.as_ref().unwrap().base10_parse::<u8>().unwrap();
            let size = field.size.as_ref().unwrap().base10_parse::<u8>().unwrap();

            // Conversion for signed fields.
            let primitive_size = super::BitField::field_primitive_size(size);

            let conversion = (
                field.signed.is_some() ||
                ty.get_ident().map(|i| crate::primitive::is_signed_primitive(i)).unwrap_or_default()
            ).then(|| {
                let primitive_type = syn::Ident::new(
                    &format!("u{}", primitive_size), field.size.as_ref().unwrap().span()
                );
                quote::quote!(as #primitive_type)
            });

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

                        #(#attrs)*
                        /// Creates a copy of the bit field with the value of the field inverted.
                        #[inline(always)]
                        #[must_use = "leaves `self` unmodified and returns a modified variant"]
                        #vis const fn #inverter(&self) -> Self {
                            self._invert_bit(#bit)
                        }
                    };
                } else if crate::primitive::is_signed_primitive(ty) {
                    return quote::quote! {
                        #(#attrs)*
                        /// Gets the value of the field.
                        #[inline(always)]
                        #vis const fn #getter(&self) -> #ty {
                            self._field(#bit, #size) as _
                        }

                        #(#attrs)*
                        /// Creates a copy of the bit field with the new value.
                        #[inline(always)]
                        #[must_use = "leaves `self` unmodified and returns a modified variant"]
                        #vis const fn #setter(&self, value: #ty) -> Self {
                            self._set_field(#bit, #size, value #conversion as _)
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
                                self._field(#bit, #size) as _
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

                                Some(self._set_field(#bit, #size, value as _))
                            }
                        }
                    } else {
                        // Fields with a size == bits_of(FieldPrimitive).
                        quote::quote! {
                            #(#attrs)*
                            /// Gets the value of the field.
                            #[inline(always)]
                            #vis const fn #getter(&self) -> #ty {
                                self._field(#bit, #size) as _
                            }

                            #(#attrs)*
                            /// Creates a copy of the bit field with the new value.
                            #[inline(always)]
                            #[must_use = "leaves `self` unmodified and returns a modified variant"]
                            #vis const fn #setter(&self, value: #ty) -> Self {
                                self._set_field(#bit, #size, value as _)
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
                    self._set_field(#bit, #size, value #conversion as _)
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

            let base_type = &self.attr.base_type;
            let primitive_type = &self.attr.primitive_type;

            let (constructor, destructor) = if !self.attr.is_non_zero {
                (quote::quote!(Self(result)), quote::quote!(self.0))
            } else {(
                quote::quote!(Self(unsafe { #base_type::new_unchecked(result) })),
                quote::quote!(self.0.get())
            )};

            quote::quote! {
                #(#attrs)*
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                #vis const fn #getter(&self, flag: #ty) -> bool {
                    self._bit(flag as _)
                }

                #(#attrs)*
                /// Returns a bit mask of all possible flags.
                #[inline(always)]
                #vis const fn #getter_mask() -> #primitive_type {
                    let mut mask = 0;

                    let mut i = 0;
                    while i < #ty::iter().len() {
                        mask |= 1 << (#ty::iter()[i] as #primitive_type);

                        i += 1;
                    }

                    mask
                }

                #(#attrs)*
                /// Returns `true` if all flags are set.
                #[inline(always)]
                #vis const fn #getter_all(&self) -> bool {
                    (#destructor & Self::#getter_mask()) == Self::#getter_mask()
                }

                #(#attrs)*
                /// Returns `true` if any flag is set.
                #[inline(always)]
                #vis const fn #getter_any(&self) -> bool {
                    (#destructor & Self::#getter_mask()) != 0
                }

                #(#attrs)*
                /// Creates a copy of the bit field with the new value for the specified flag.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                #vis const fn #setter(&self, flag: #ty, value: bool) -> Self {
                    self._set_bit(flag as _, value)
                }

                #(#attrs)*
                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                #vis const fn #setter_all(&self) -> Self {
                    let result = #destructor | Self::#getter_mask();
                    #constructor
                }

                #(#attrs)*
                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                #vis const fn #setter_none(&self) -> Self {
                    let result = #destructor & !Self::#getter_mask();
                    #constructor
                }

                #(#attrs)*
                /// Creates a copy of the bit field with the value of the specified flag inverted.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                #vis const fn #inverter(&self, flag: #ty) -> Self {
                    self._invert_bit(flag as _)
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
                        ), &syn::Ident::new(
                            &format!("invert_{}", &entry.ident), entry.ident.span()
                        )
                    ));
                }

                fields
            },

            super::Data::Tuple(entry) => {
                vec!(Self::generate_accessor(
                    &self, entry,
                    &syn::Ident::new(
                        if entry.field.is_some() { "get" } else { "has" },
                        entry.ty.span()
                    ),
                    &syn::Ident::new("set", entry.ty.span()),
                    &syn::Ident::new("invert", entry.ty.span())
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

    /// Generates `core::ops::*` implementations for flags, but only if the flag visibility is equal
    /// or higher than the bit field visibility, and for fields, if the type of the field is only
    /// used once.
    fn generate_accessors_ops(&self) -> proc_macro2::TokenStream {
        // Collect the amount of occurrences for used field types.
        let mut field_type_occurrences = std::collections::HashMap::new();

        for ty in self.data
            .entries()
            .iter()
            .filter_map(|x| x.field.is_some().then(|| &x.ty))
        {
            let ty = quote::quote!(#ty).to_string();

            if let Some(occurrences) = field_type_occurrences.get_mut(&ty) {
                *occurrences += 1;
            } else {
                field_type_occurrences.insert(ty, 1usize);
            }
        }

        // Collect all operator implementations.
        let mut implementations = vec!();

        match &self.data {
            super::Data::Named(entries) => {
                for entry in entries {
                    if entry.entry.field.is_some() {
                        if let Some(ident) = entry.entry.ty.get_ident() {
                            if crate::primitive::is_primitive(ident) {
                                continue;
                            }
                        }

                        let ty = &entry.entry.ty;
                        let ty = quote::quote!(#ty).to_string();

                        if let Some(occurrences) = field_type_occurrences.get(&ty) {
                            if *occurrences == 1 {
                                implementations.push(Self::generate_accessor_ops_field(
                                    &self, &entry.entry.ty, &syn::Ident::new(
                                        &format!("set_{}", &entry.ident), entry.ident.span()
                                    )
                                ));
                            }
                        }
                    } else {
                        if Self::cmp_vis(&entry.entry.vis, &self.vis) >= 0 {
                            implementations.push(Self::generate_accessor_ops_flags(
                                &self, &entry.entry.ty, &syn::Ident::new(
                                    &format!("set_{}", &entry.ident), entry.ident.span()
                                ),&syn::Ident::new(
                                    &format!("invert_{}", &entry.ident), entry.ident.span()
                                )
                            ));
                        }
                    }
                }
            },

            super::Data::Tuple(entry) => {
                if entry.field.is_some() {
                    if !entry.ty.get_ident()
                        .map(|i| crate::primitive::is_primitive(i))
                        .unwrap_or_default()
                    {
                        let ty = &entry.ty;
                        let ty = quote::quote!(#ty).to_string();

                        if let Some(occurrences) = field_type_occurrences.get(&ty) {
                            if *occurrences == 1 {
                                implementations.push(Self::generate_accessor_ops_field(
                                    &self, &entry.ty,
                                    &syn::Ident::new("set", entry.ty.span())
                                ));
                            }
                        }
                    }
                } else {
                    if Self::cmp_vis(&entry.vis, &self.vis) >= 0 {
                        implementations.push(Self::generate_accessor_ops_flags(
                            &self, &entry.ty,
                            &syn::Ident::new("set", entry.ty.span()),
                            &syn::Ident::new("invert", entry.ty.span())
                        ));
                    }
                }
            }
        }

        if implementations.is_empty() {
            return proc_macro2::TokenStream::new();
        }

        quote::quote! {
            #(#implementations)*
        }
    }

    /// Generates `core::ops::*` implementations for a single field in a bit field.
    fn generate_accessor_ops_field(
        &self,
        field: &syn::Path,
        setter: &syn::Ident
    ) -> proc_macro2::TokenStream {
        let ty = &self.ident;

        quote::quote! {
            impl core::ops::Add<#field> for #ty {
                type Output = Self;

                #[inline(always)]
                fn add(self, value: #field) -> Self::Output {
                    self.#setter(value)
                }
            }

            impl core::ops::AddAssign<#field> for #ty {
                #[inline(always)]
                fn add_assign(&mut self, value: #field) {
                    self.0 = self.#setter(value).0;
                }
            }
        }
    }

    /// Generates `core::ops::*` implementations for a single flag type in a bit field.
    fn generate_accessor_ops_flags(
        &self,
        flags: &syn::Path,
        setter: &syn::Ident,
        inverter: &syn::Ident
    ) -> proc_macro2::TokenStream {
        let ty = &self.ident;

        quote::quote! {
            impl core::ops::Add<#flags> for #ty {
                type Output = Self;

                #[inline(always)]
                fn add(self, flag: #flags) -> Self::Output {
                    self.#setter(flag, true)
                }
            }

            impl core::ops::AddAssign<#flags> for #ty {
                #[inline(always)]
                fn add_assign(&mut self, flag: #flags) {
                    self.0 = self.#setter(flag, true).0;
                }
            }

            impl core::ops::BitXor<#flags> for #ty {
                type Output = Self;

                #[inline(always)]
                fn bitxor(self, flag: #flags) -> Self::Output {
                    self.#inverter(flag)
                }
            }

            impl core::ops::BitXorAssign<#flags> for #ty {
                #[inline(always)]
                fn bitxor_assign(&mut self, flag: #flags) {
                    self.0 = self.#inverter(flag).0;
                }
            }

            impl core::ops::Sub<#flags> for #ty {
                type Output = Self;

                #[inline(always)]
                fn sub(self, flag: #flags) -> Self::Output {
                    self.#setter(flag, false)
                }
            }

            impl core::ops::SubAssign<#flags> for #ty {
                #[inline(always)]
                fn sub_assign(&mut self, flag: #flags) {
                    self.0 = self.#setter(flag, false).0;
                }
            }
        }
    }

    /// Generates the accessors that directly work on the primitive bit field type.
    fn generate_accessors_low(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let base_type = &self.attr.base_type;
        let primitive_type = &self.attr.primitive_type;

        let (constructor, destructor) = if !self.attr.is_non_zero {
            (quote::quote!(Self(result)), quote::quote!(self.0))
        } else {(
            quote::quote!(Self(unsafe { #base_type::new_unchecked(result) })),
            quote::quote!(self.0.get())
        )};

        quote::quote! {
            impl #ident {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((#destructor >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = #destructor & !(1 << position);
                    let result = cleared | ((value as #primitive_type) << position);
                    #constructor
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = #destructor ^ ((1 as #primitive_type) << position);
                    #constructor
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> #primitive_type {
                    let shifted = #destructor >> position;

                    let rest = size as #primitive_type % (core::mem::size_of::<#primitive_type>() * 8) as #primitive_type;
                    let bit = (rest > 0) as #primitive_type;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: #primitive_type) -> Self {
                    let rest = size as #primitive_type % (core::mem::size_of::<#primitive_type>() * 8) as #primitive_type;
                    let bit = (rest > 0) as #primitive_type;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = #destructor & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    #constructor
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

        let ident = &self.ident;
        let base_type = &self.attr.base_type;

        let entries = self.data.entries();
        let mut assertions = entries.iter().enumerate().map(|(i, entry)| {
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
                        const ASSERT: bool = core::mem::size_of::<#base_type>() * 8 >= #bit + #size;
                        ASSERT
                    }
                });

                // `bits_of(BitField)` must not be == `size_of(Field)`.
                let size_not_equal_assertion = generate_assertion(&syn::Ident::new(&format!(
                    "_FIELD_{}_HAS_THE_SIZE_OF_THE_WHOLE_BIT_FIELD", i
                ), field.span), quote::quote! {
                    !{
                        const ASSERT: bool = core::mem::size_of::<#base_type>() * 8 != #size;
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
                        core::mem::size_of::<#base_type>() * 8 > #ty::max() as usize;
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
        }).collect::<Vec<_>>();

        // Ensure `sizeof<T> == sizeof<Option<T>>` for `NonZero` bitfield types.
        if self.attr.is_non_zero {
            assertions.push(generate_assertion(
                &syn::Ident::new(
                    "_OPTION_OF_NON_ZERO_BITFIELD_HAS_A_DIFFERENT_SIZE",
                    self.attr.base_type.span()
                ), quote::quote! {!{
                    const ASSERT: bool =
                        core::mem::size_of::<#ident>()
                        ==
                        core::mem::size_of::<core::option::Option<#ident>>()
                    ; ASSERT
                }}
            ));
        }

        if assertions.len() == 0 { return proc_macro2::TokenStream::new(); }

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
        let base_type = &self.attr.base_type;

        let constructor = if !self.attr.is_non_zero {
            quote::quote!(Self(0))
        } else {
            quote::quote!(Self(unsafe { #base_type::new_unchecked(0) }))
        };

        quote::quote! {
            impl #ident {
                /// Creates a new instance with all flags and fields cleared.
                #[inline(always)]
                #vis const fn new() -> Self {
                    #constructor
                }
            }
        }
    }

    /// Generates the main bit field structure.
    fn generate_struct(&self) -> proc_macro2::TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let ident = &self.ident;
        let base_type = &self.attr.base_type;

        quote::quote! {
            #[repr(transparent)]
            #(#attrs)*
            #vis struct #ident(#base_type);
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
        let accessors_ops = self.generate_accessors_ops();
        let assertions = self.generate_assertions();
        let debug = self.generate_debug();
        let display = self.generate_display();

        quote::quote! {
            #field
            #implementation
            #accessors_low
            #accessors
            #accessors_ops
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
                let inverter = syn::Ident::new("test_invert", entry.ty.span());

                assert_eq!(
                    bitfield.generate_accessor(&entry, &getter, &setter, &inverter).to_string(),
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
    fn cmp_vis() {
        let visibilities: &[syn::Visibility] = &[
            syn::parse_str::<syn::Visibility>("pub").unwrap(),
            syn::parse_str::<syn::Visibility>("crate").unwrap(),
            syn::parse_str::<syn::Visibility>("pub(crate)").unwrap(),
            syn::parse_str::<syn::Visibility>("pub(super)").unwrap(),
            syn::parse_str::<syn::Visibility>("pub(in super)").unwrap(),
            syn::parse_str::<syn::Visibility>("pub(in some::module)").unwrap(),
            syn::parse_str::<syn::Visibility>("pub(in more::module)").unwrap(),
            syn::parse_str::<syn::Visibility>("pub(self)").unwrap(),
            syn::parse_str::<syn::Visibility>("pub(in self)").unwrap(),
            syn::parse_str::<syn::Visibility>("").unwrap()
        ];

        const EXPECTED_RESULTS: &[i8] = &[
        //   p   c   p   p   p   p   p   p   p   p
        //   u   r   u   u   u   u   u   u   u   r
        //   b   a   b   b   b   b   b   b   b   i
        //       t   (   (   (   (   (   (   (   v
        //       e   c   s   i   s   m   s   i
        //           r   u   n   o   o   e   n
        //           a   p       m   r   l
        //           t   e   s   e   e   f   s
        //           e   r   u   :   :   )   e
        //           )   )   p   :   :       l
        //                   e   m   m       f
        //                   r   o   o       )
        //                   )   d   d
        //                       u   u
        //                       l   l
        //                       e   e
        //                       )   )
             0,  1,  1,  1,  1,  1,  1,  1,  1,  1, // pub (0-9)
            -1,  0,  0,  1,  1,  1,  1,  1,  1,  1, // crate (10-19)
            -1,  0,  0,  1,  1,  1,  1,  1,  1,  1, // pub(crate) (20-29)
            -1, -1, -1,  0,  0, -1, -1,  1,  1,  1, // pub(super) (30-39)
            -1, -1, -1,  0,  0, -1, -1,  1,  1,  1, // pub(in super) (40-49)
            -1, -1, -1, -1, -1,  0, -1, -1, -1,  1, // pub(in some::module) (50-59)
            -1, -1, -1, -1, -1, -1,  0, -1, -1,  1, // pub(in more::module) (60-69)
            -1, -1, -1, -1, -1, -1, -1,  0,  0,  0, // pub(self) (70-79)
            -1, -1, -1, -1, -1, -1, -1,  0,  0,  0, // pub(in self) (80-89)
            -1, -1, -1, -1, -1, -1, -1,  0,  0,  0  // priv (90-99)
        ];

        let mut index = 0;
        for left in 0..visibilities.len() {
            for right in 0..visibilities.len() {
                assert_eq!(
                    (EXPECTED_RESULTS[index], index),
                    (BitField::cmp_vis(&visibilities[left], &visibilities[right]), index)
                );
                index += 1;
            }
        }
    }

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
                self._bit(flag as _)
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
                self._set_bit(flag as _, value)
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                let result = self.0 | Self::test_get_mask();
                Self(result)
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                let result = self.0 & !Self::test_get_mask();
                Self(result)
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with the value of the specified flag inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_invert(&self, flag: A) -> Self {
                self._invert_bit(flag as _)
            }
        });
        assert_accessor!("NonZero8", "struct A(#[some_attribute1] #[some_attribute2] A);", quote::quote! {
            #[some_attribute1]
            #[some_attribute2]
            /// Returns `true` if the specified `flag` is set.
            #[inline(always)]
            const fn test_get(&self, flag: A) -> bool {
                self._bit(flag as _)
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
                (self.0.get() & Self::test_get_mask()) == Self::test_get_mask()
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Returns `true` if any flag is set.
            #[inline(always)]
            const fn test_get_any(&self) -> bool {
                (self.0.get() & Self::test_get_mask()) != 0
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with the new value for the specified flag.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, flag: A, value: bool) -> Self {
                self._set_bit(flag as _, value)
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                let result = self.0.get() | Self::test_get_mask();
                Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                let result = self.0.get() & !Self::test_get_mask();
                Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
            }

            #[some_attribute1]
            #[some_attribute2]
            /// Creates a copy of the bit field with the value of the specified flag inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_invert(&self, flag: A) -> Self {
                self._invert_bit(flag as _)
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        );
        assert_accessor!(
            "NonZero8", "struct A(#[some_attribute1] #[some_attribute2] #[field(0, 1)] A);", quote::quote! {
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
                    self._set_field(0u8, 1u8, value as _)
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
                self._bit(flag as _)
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
                self._set_bit(flag as _, value)
            }

            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            pub const fn test_set_all(&self) -> Self {
                let result = self.0 | Self::test_get_mask();
                Self(result)
            }

            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            pub const fn test_set_none(&self) -> Self {
                let result = self.0 & !Self::test_get_mask();
                Self(result)
            }

            /// Creates a copy of the bit field with the value of the specified flag inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            pub const fn test_invert(&self, flag: A) -> Self {
                self._invert_bit(flag as _)
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
                    self._set_field(0u8, 1u8, value as _)
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
                self._bit(flag as _)
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
                self._set_bit(flag as _, value)
            }

            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                let result = self.0 | Self::test_get_mask();
                Self(result)
            }

            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                let result = self.0 & !Self::test_get_mask();
                Self(result)
            }

            /// Creates a copy of the bit field with the value of the specified flag inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_invert(&self, flag: B) -> Self {
                self._invert_bit(flag as _)
            }
        });
        assert_accessor!("NonZero8", "struct A(B);", quote::quote! {
            /// Returns `true` if the specified `flag` is set.
            #[inline(always)]
            const fn test_get(&self, flag: B) -> bool {
                self._bit(flag as _)
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
                (self.0.get() & Self::test_get_mask()) == Self::test_get_mask()
            }

            /// Returns `true` if any flag is set.
            #[inline(always)]
            const fn test_get_any(&self) -> bool {
                (self.0.get() & Self::test_get_mask()) != 0
            }

            /// Creates a copy of the bit field with the new value for the specified flag.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, flag: B, value: bool) -> Self {
                self._set_bit(flag as _, value)
            }

            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                let result = self.0.get() | Self::test_get_mask();
                Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
            }

            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                let result = self.0.get() & !Self::test_get_mask();
                Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
            }

            /// Creates a copy of the bit field with the value of the specified flag inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_invert(&self, flag: B) -> Self {
                self._invert_bit(flag as _)
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        );
        assert_accessor!(
            "NonZero8", "struct A(#[field(0, 1)] B);", quote::quote! {
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
                    self._set_field(0u8, 1u8, value as _)
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
                self._bit(flag as _)
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
                self._set_bit(flag as _, value)
            }

            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                let result = self.0 | Self::test_get_mask();
                Self(result)
            }

            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                let result = self.0 & !Self::test_get_mask();
                Self(result)
            }

            /// Creates a copy of the bit field with the value of the specified flag inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_invert(&self, flag: A) -> Self {
                self._invert_bit(flag as _)
            }
        });
        assert_accessor!("NonZero32", "struct A(A);", quote::quote! {
            /// Returns `true` if the specified `flag` is set.
            #[inline(always)]
            const fn test_get(&self, flag: A) -> bool {
                self._bit(flag as _)
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
                (self.0.get() & Self::test_get_mask()) == Self::test_get_mask()
            }

            /// Returns `true` if any flag is set.
            #[inline(always)]
            const fn test_get_any(&self) -> bool {
                (self.0.get() & Self::test_get_mask()) != 0
            }

            /// Creates a copy of the bit field with the new value for the specified flag.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, flag: A, value: bool) -> Self {
                self._set_bit(flag as _, value)
            }

            /// Creates a copy of the bit field with all flags set.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_all(&self) -> Self {
                let result = self.0.get() | Self::test_get_mask();
                Self(unsafe { core::num::NonZeroU32::new_unchecked(result) })
            }

            /// Creates a copy of the bit field with all flags cleared.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set_none(&self) -> Self {
                let result = self.0.get() & !Self::test_get_mask();
                Self(unsafe { core::num::NonZeroU32::new_unchecked(result) })
            }

            /// Creates a copy of the bit field with the value of the specified flag inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_invert(&self, flag: A) -> Self {
                self._invert_bit(flag as _)
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        );
        assert_accessor!(
            "NonZero32", "struct A(#[field(0, 1)] A);", quote::quote! {
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
                    self._set_field(0u8, 1u8, value as _)
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
                    self._set_field(1u8, 9u8, value as _)
                }
            }
        );
        assert_accessor!(
            "NonZero32", "struct A(#[field(1, 9)] A);", quote::quote! {
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
                    self._set_field(1u8, 9u8, value as _)
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

            /// Creates a copy of the bit field with the value of the field inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_invert(&self) -> Self {
                self._invert_bit(2u8)
            }
        });
        assert_accessor!("NonZero8", "struct A(#[field(2, 1)] bool);", quote::quote! {
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

            /// Creates a copy of the bit field with the value of the field inverted.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_invert(&self) -> Self {
                self._invert_bit(2u8)
            }
        });

        assert_accessor!("8", "struct A(#[field(3, 2)] u8);", quote::quote! {
            /// Gets the value of the field.
            #[inline(always)]
            const fn test_get(&self) -> u8 {
                self._field(3u8, 2u8) as _
            }

            /// Creates a copy of the bit field with the new value.
            ///
            /// Returns `None` if `value` is bigger than the specified amount of
            /// bits for the field can store.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, value: u8) -> Option<Self> {
                if value >= (1 as u8).wrapping_shl(2u8 as u32) { return None; }

                Some(self._set_field(3u8, 2u8, value as _))
            }
        });
        assert_accessor!("NonZero8", "struct A(#[field(3, 2)] u8);", quote::quote! {
            /// Gets the value of the field.
            #[inline(always)]
            const fn test_get(&self) -> u8 {
                self._field(3u8, 2u8) as _
            }

            /// Creates a copy of the bit field with the new value.
            ///
            /// Returns `None` if `value` is bigger than the specified amount of
            /// bits for the field can store.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, value: u8) -> Option<Self> {
                if value >= (1 as u8).wrapping_shl(2u8 as u32) { return None; }

                Some(self._set_field(3u8, 2u8, value as _))
            }
        });

        assert_accessor!("16", "struct A(#[field(3, 8)] u8);", quote::quote! {
            /// Gets the value of the field.
            #[inline(always)]
            const fn test_get(&self) -> u8 {
                self._field(3u8, 8u8) as _
            }

            /// Creates a copy of the bit field with the new value.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, value: u8) -> Self {
                self._set_field(3u8, 8u8, value as _)
            }
        });
        assert_accessor!("NonZero16", "struct A(#[field(3, 8)] u8);", quote::quote! {
            /// Gets the value of the field.
            #[inline(always)]
            const fn test_get(&self) -> u8 {
                self._field(3u8, 8u8) as _
            }

            /// Creates a copy of the bit field with the new value.
            #[inline(always)]
            #[must_use = "leaves `self` unmodified and returns a modified variant"]
            const fn test_set(&self, value: u8) -> Self {
                self._set_field(3u8, 8u8, value as _)
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
                    self._set_field(0u8, 8u8, value as _)
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
                    self._set_field(0u8, 8u8, value as u8 as _)
                }
            }
        );

        assert_accessor!(
            "32", "struct A(u8);", quote::quote! {
                /// Gets the value of the field.
                #[inline(always)]
                const fn test_get(&self) -> u8 {
                    self._field(0u8, 8u8) as _
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: u8) -> Self {
                    self._set_field(0u8, 8u8, value as _)
                }
            }
        );

        assert_accessor!(
            "32", "struct A(i8);", quote::quote! {
                /// Gets the value of the field.
                #[inline(always)]
                const fn test_get(&self) -> i8 {
                    self._field(0u8, 8u8) as _
                }

                /// Creates a copy of the bit field with the new value.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn test_set(&self, value: i8) -> Self {
                    self._set_field(0u8, 8u8, value as u8 as _)
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
                    self._bit(flag as _)
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
                    self._set_bit(flag as _, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_all(&self) -> Self {
                    let result = self.0 | Self::has_mask();
                    Self(result)
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_none(&self) -> Self {
                    let result = self.0 & !Self::has_mask();
                    Self(result)
                }

                /// Creates a copy of the bit field with the value of the specified flag inverted.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn invert(&self, flag: B) -> Self {
                    self._invert_bit(flag as _)
                }
            }
        });
        assert_compare!(generate_accessors, "NonZero8", "struct A(B);", quote::quote! {
            impl A {
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                const fn has(&self, flag: B) -> bool {
                    self._bit(flag as _)
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
                    (self.0.get() & Self::has_mask()) == Self::has_mask()
                }

                /// Returns `true` if any flag is set.
                #[inline(always)]
                const fn has_any(&self) -> bool {
                    (self.0.get() & Self::has_mask()) != 0
                }

                /// Creates a copy of the bit field with the new value for the specified flag.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set(&self, flag: B, value: bool) -> Self {
                    self._set_bit(flag as _, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_all(&self) -> Self {
                    let result = self.0.get() | Self::has_mask();
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_none(&self) -> Self {
                    let result = self.0.get() & !Self::has_mask();
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
                }

                /// Creates a copy of the bit field with the value of the specified flag inverted.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn invert(&self, flag: B) -> Self {
                    self._invert_bit(flag as _)
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        });
        assert_compare!(generate_accessors, "NonZero8", "struct A(#[field(0, 1)] B);", quote::quote! {
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A {}", quote::quote! {});
        assert_compare!(generate_accessors, "NonZero8", "struct A {}", quote::quote! {});

        assert_compare!(generate_accessors, "8", "struct A { b: B }", quote::quote! {
            impl A {
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                const fn b(&self, flag: B) -> bool {
                    self._bit(flag as _)
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
                    self._set_bit(flag as _, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_all(&self) -> Self {
                    let result = self.0 | Self::b_mask();
                    Self(result)
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_none(&self) -> Self {
                    let result = self.0 & !Self::b_mask();
                    Self(result)
                }

                /// Creates a copy of the bit field with the value of the specified flag inverted.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn invert_b(&self, flag: B) -> Self {
                    self._invert_bit(flag as _)
                }
            }
        });
        assert_compare!(generate_accessors, "NonZero8", "struct A { b: B }", quote::quote! {
            impl A {
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                const fn b(&self, flag: B) -> bool {
                    self._bit(flag as _)
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
                    (self.0.get() & Self::b_mask()) == Self::b_mask()
                }

                /// Returns `true` if any flag is set.
                #[inline(always)]
                const fn b_any(&self) -> bool {
                    (self.0.get() & Self::b_mask()) != 0
                }

                /// Creates a copy of the bit field with the new value for the specified flag.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b(&self, flag: B, value: bool) -> Self {
                    self._set_bit(flag as _, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_all(&self) -> Self {
                    let result = self.0.get() | Self::b_mask();
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_none(&self) -> Self {
                    let result = self.0.get() & !Self::b_mask();
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
                }

                /// Creates a copy of the bit field with the value of the specified flag inverted.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn invert_b(&self, flag: B) -> Self {
                    self._invert_bit(flag as _)
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        });
        assert_compare!(generate_accessors, "NonZero8", "struct A {#[field(0, 1)] b: B}", quote::quote! {
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A {b: B, #[field(0, 1)] c: C}", quote::quote! {
            impl A {
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                const fn b(&self, flag: B) -> bool {
                    self._bit(flag as _)
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
                    self._set_bit(flag as _, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_all(&self) -> Self {
                    let result = self.0 | Self::b_mask();
                    Self(result)
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_none(&self) -> Self {
                    let result = self.0 & !Self::b_mask();
                    Self(result)
                }

                /// Creates a copy of the bit field with the value of the specified flag inverted.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn invert_b(&self, flag: B) -> Self {
                    self._invert_bit(flag as _)
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        });
        assert_compare!(generate_accessors, "NonZero8", "struct A {b: B, #[field(0, 1)] c: C}", quote::quote! {
            impl A {
                /// Returns `true` if the specified `flag` is set.
                #[inline(always)]
                const fn b(&self, flag: B) -> bool {
                    self._bit(flag as _)
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
                    (self.0.get() & Self::b_mask()) == Self::b_mask()
                }

                /// Returns `true` if any flag is set.
                #[inline(always)]
                const fn b_any(&self) -> bool {
                    (self.0.get() & Self::b_mask()) != 0
                }

                /// Creates a copy of the bit field with the new value for the specified flag.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b(&self, flag: B, value: bool) -> Self {
                    self._set_bit(flag as _, value)
                }

                /// Creates a copy of the bit field with all flags set.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_all(&self) -> Self {
                    let result = self.0.get() | Self::b_mask();
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
                }

                /// Creates a copy of the bit field with all flags cleared.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b_none(&self) -> Self {
                    let result = self.0.get() & !Self::b_mask();
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
                }

                /// Creates a copy of the bit field with the value of the specified flag inverted.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn invert_b(&self, flag: B) -> Self {
                    self._invert_bit(flag as _)
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
                    self._set_field(0u8, 1u8, value as _)
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A(#[field(0, 1)] u8);", quote::quote! {
            impl A {
                /// Gets the value of the field.
                #[inline(always)]
                const fn get(&self) -> u8 {
                    self._field(0u8, 1u8) as _
                }

                /// Creates a copy of the bit field with the new value.
                ///
                /// Returns `None` if `value` is bigger than the specified amount of
                /// bits for the field can store.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set(&self, value: u8) -> Option<Self> {
                    if value >= (1 as u8).wrapping_shl(1u8 as u32) { return None; }

                    Some(self._set_field(0u8, 1u8, value as _))
                }
            }
        });
        assert_compare!(generate_accessors, "NonZero8", "struct A(#[field(0, 1)] u8);", quote::quote! {
            impl A {
                /// Gets the value of the field.
                #[inline(always)]
                const fn get(&self) -> u8 {
                    self._field(0u8, 1u8) as _
                }

                /// Creates a copy of the bit field with the new value.
                ///
                /// Returns `None` if `value` is bigger than the specified amount of
                /// bits for the field can store.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set(&self, value: u8) -> Option<Self> {
                    if value >= (1 as u8).wrapping_shl(1u8 as u32) { return None; }

                    Some(self._set_field(0u8, 1u8, value as _))
                }
            }
        });

        assert_compare!(generate_accessors, "8", "struct A { #[field(0, 1)] b: u8 }", quote::quote! {
            impl A {
                /// Gets the value of the field.
                #[inline(always)]
                const fn b(&self) -> u8 {
                    self._field(0u8, 1u8) as _
                }

                /// Creates a copy of the bit field with the new value.
                ///
                /// Returns `None` if `value` is bigger than the specified amount of
                /// bits for the field can store.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b(&self, value: u8) -> Option<Self> {
                    if value >= (1 as u8).wrapping_shl(1u8 as u32) { return None; }

                    Some(self._set_field(0u8, 1u8, value as _))
                }
            }
        });
        assert_compare!(generate_accessors, "NonZero8", "struct A { #[field(0, 1)] b: u8 }", quote::quote! {
            impl A {
                /// Gets the value of the field.
                #[inline(always)]
                const fn b(&self) -> u8 {
                    self._field(0u8, 1u8) as _
                }

                /// Creates a copy of the bit field with the new value.
                ///
                /// Returns `None` if `value` is bigger than the specified amount of
                /// bits for the field can store.
                #[inline(always)]
                #[must_use = "leaves `self` unmodified and returns a modified variant"]
                const fn set_b(&self, value: u8) -> Option<Self> {
                    if value >= (1 as u8).wrapping_shl(1u8 as u32) { return None; }

                    Some(self._set_field(0u8, 1u8, value as _))
                }
            }
        });
    }

    #[test]
    fn accessors_ops() {
        assert_compare!(generate_accessors_ops, "8", "struct A(B);", quote::quote! {
            impl core::ops::Add<B> for A {
                type Output = Self;

                #[inline(always)]
                fn add(self, flag: B) -> Self::Output {
                    self.set(flag, true)
                }
            }

            impl core::ops::AddAssign<B> for A {
                #[inline(always)]
                fn add_assign(&mut self, flag: B) {
                    self.0 = self.set(flag, true).0;
                }
            }

            impl core::ops::BitXor<B> for A {
                type Output = Self;

                #[inline(always)]
                fn bitxor(self, flag: B) -> Self::Output {
                    self.invert(flag)
                }
            }

            impl core::ops::BitXorAssign<B> for A {
                #[inline(always)]
                fn bitxor_assign(&mut self, flag: B) {
                    self.0 = self.invert(flag).0;
                }
            }

            impl core::ops::Sub<B> for A {
                type Output = Self;

                #[inline(always)]
                fn sub(self, flag: B) -> Self::Output {
                    self.set(flag, false)
                }
            }

            impl core::ops::SubAssign<B> for A {
                #[inline(always)]
                fn sub_assign(&mut self, flag: B) {
                    self.0 = self.set(flag, false).0;
                }
            }
        });

        assert_compare!(generate_accessors_ops, "8", "pub struct A(B);", quote::quote! {});

        assert_compare!(generate_accessors_ops, "8", "struct A(#[field(0, 1)] B);", quote::quote! {
            impl core::ops::Add<B> for A {
                type Output = Self;

                #[inline(always)] fn add(self, value: B) -> Self::Output {
                    self.set(value)
                }
            }

            impl core::ops::AddAssign<B> for A {
                #[inline(always)]
                fn add_assign(&mut self, value: B) {
                    self.0 = self.set(value).0;
                }
            }
        });

        assert_compare!(generate_accessors_ops, "16", "struct A(u8);", quote::quote! {});

        assert_compare!(generate_accessors_ops, "8", "struct A { #[field(0, 1)] b: u8 }", quote::quote! {});

        assert_compare!(generate_accessors_ops, "8", "struct A {#[field(0, 1)] b1: B, #[field(1, 1)] b2: B}", quote::quote! {});

        assert_compare!(generate_accessors_ops, "8", "struct A {#[field(0, 1)] b: B, #[field(1, 1)] c: C}", quote::quote! {
            impl core::ops::Add<B> for A {
                type Output = Self;

                #[inline(always)] fn add(self, value: B) -> Self::Output {
                    self.set_b(value)
                }
            }

            impl core::ops::AddAssign<B> for A {
                #[inline(always)]
                fn add_assign(&mut self, value: B) {
                    self.0 = self.set_b(value).0;
                }
            }

            impl core::ops::Add<C> for A {
                type Output = Self;

                #[inline(always)] fn add(self, value: C) -> Self::Output {
                    self.set_c(value)
                }
            }

            impl core::ops::AddAssign<C> for A {
                #[inline(always)]
                fn add_assign(&mut self, value: C) {
                    self.0 = self.set_c(value).0;
                }
            }
        });

        assert_compare!(generate_accessors_ops, "8", "struct A {#[field(0, 1)] b: B, #[field(1, 1)] b: B, #[field(2, 1)] c: C}", quote::quote! {
            impl core::ops::Add<C> for A {
                type Output = Self;

                #[inline(always)] fn add(self, value: C) -> Self::Output {
                    self.set_c(value)
                }
            }

            impl core::ops::AddAssign<C> for A {
                #[inline(always)]
                fn add_assign(&mut self, value: C) {
                    self.0 = self.set_c(value).0;
                }
            }
        });

        assert_compare!(generate_accessors_ops, "8", "struct A {}", quote::quote! {});

        assert_compare!(generate_accessors_ops, "8", "struct A { b: B }", quote::quote! {
            impl core::ops::Add<B> for A {
                type Output = Self;

                #[inline(always)]
                fn add(self, flag: B) -> Self::Output {
                    self.set_b(flag, true)
                }
            }

            impl core::ops::AddAssign<B> for A {
                #[inline(always)]
                fn add_assign(&mut self, flag: B) {
                    self.0 = self.set_b(flag, true).0;
                }
            }

            impl core::ops::BitXor<B> for A {
                type Output = Self;

                #[inline(always)]
                fn bitxor(self, flag: B) -> Self::Output {
                    self.invert_b(flag)
                }
            }

            impl core::ops::BitXorAssign<B> for A {
                #[inline(always)]
                fn bitxor_assign(&mut self, flag: B) {
                    self.0 = self.invert_b(flag).0;
                }
            }

            impl core::ops::Sub<B> for A {
                type Output = Self;

                #[inline(always)]
                fn sub(self, flag: B) -> Self::Output {
                    self.set_b(flag, false)
                }
            }

            impl core::ops::SubAssign<B> for A {
                #[inline(always)]
                fn sub_assign(&mut self, flag: B) {
                    self.0 = self.set_b(flag, false).0;
                }
            }
        });

        assert_compare!(generate_accessors_ops, "8", "pub struct A { b: B }", quote::quote! {});

        assert_compare!(generate_accessors_ops, "8", "struct A { b: B, c: C }", quote::quote! {
            impl core::ops::Add<B> for A {
                type Output = Self;

                #[inline(always)]
                fn add(self, flag: B) -> Self::Output {
                    self.set_b(flag, true)
                }
            }

            impl core::ops::AddAssign<B> for A {
                #[inline(always)]
                fn add_assign(&mut self, flag: B) {
                    self.0 = self.set_b(flag, true).0;
                }
            }

            impl core::ops::BitXor<B> for A {
                type Output = Self;

                #[inline(always)]
                fn bitxor(self, flag: B) -> Self::Output {
                    self.invert_b(flag)
                }
            }

            impl core::ops::BitXorAssign<B> for A {
                #[inline(always)]
                fn bitxor_assign(&mut self, flag: B) {
                    self.0 = self.invert_b(flag).0;
                }
            }

            impl core::ops::Sub<B> for A {
                type Output = Self;

                #[inline(always)]
                fn sub(self, flag: B) -> Self::Output {
                    self.set_b(flag, false)
                }
            }

            impl core::ops::SubAssign<B> for A {
                #[inline(always)]
                fn sub_assign(&mut self, flag: B) {
                    self.0 = self.set_b(flag, false).0;
                }
            }

            impl core::ops::Add<C> for A {
                type Output = Self;

                #[inline(always)]
                fn add(self, flag: C) -> Self::Output {
                    self.set_c(flag, true)
                }
            }

            impl core::ops::AddAssign<C> for A {
                #[inline(always)]
                fn add_assign(&mut self, flag: C) {
                    self.0 = self.set_c(flag, true).0;
                }
            }

            impl core::ops::BitXor<C> for A {
                type Output = Self;

                #[inline(always)]
                fn bitxor(self, flag: C) -> Self::Output {
                    self.invert_c(flag)
                }
            }

            impl core::ops::BitXorAssign<C> for A {
                #[inline(always)]
                fn bitxor_assign(&mut self, flag: C) {
                    self.0 = self.invert_c(flag).0;
                }
            }

            impl core::ops::Sub<C> for A {
                type Output = Self;

                #[inline(always)]
                fn sub(self, flag: C) -> Self::Output {
                    self.set_c(flag, false)
                }
            }

            impl core::ops::SubAssign<C> for A {
                #[inline(always)]
                fn sub_assign(&mut self, flag: C) {
                    self.0 = self.set_c(flag, false).0;
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
                    let result = cleared | ((value as u8) << position);
                    Self(result)
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0 ^ ((1 as u8) << position);
                    Self(result)
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u8 {
                    let shifted = self.0 >> position;

                    let rest = size as u8 % (core::mem::size_of::<u8>() * 8) as u8;
                    let bit = (rest > 0) as u8;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u8) -> Self {
                    let rest = size as u8 % (core::mem::size_of::<u8>() * 8) as u8;
                    let bit = (rest > 0) as u8;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });
        assert_compare!(generate_accessors_low, "NonZero8", "struct A(B);", quote::quote! {
            impl A {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0.get() >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0.get() & !(1 << position);
                    let result = cleared | ((value as u8) << position);
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0.get() ^ ((1 as u8) << position);
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u8 {
                    let shifted = self.0.get() >> position;

                    let rest = size as u8 % (core::mem::size_of::<u8>() * 8) as u8;
                    let bit = (rest > 0) as u8;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u8) -> Self {
                    let rest = size as u8 % (core::mem::size_of::<u8>() * 8) as u8;
                    let bit = (rest > 0) as u8;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0.get() & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(unsafe { core::num::NonZeroU8::new_unchecked(result) })
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
                    let result = cleared | ((value as u16) << position);
                    Self(result)
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0 ^ ((1 as u16) << position);
                    Self(result)
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u16 {
                    let shifted = self.0 >> position;

                    let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                    let bit = (rest > 0) as u16;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u16) -> Self {
                    let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                    let bit = (rest > 0) as u16;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });
        assert_compare!(generate_accessors_low, "NonZero16", "struct B(C);", quote::quote! {
            impl B {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0.get() >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0.get() & !(1 << position);
                    let result = cleared | ((value as u16) << position);
                    Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0.get() ^ ((1 as u16) << position);
                    Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u16 {
                    let shifted = self.0.get() >> position;

                    let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                    let bit = (rest > 0) as u16;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u16) -> Self {
                    let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                    let bit = (rest > 0) as u16;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0.get() & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
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
                    let result = cleared | ((value as u32) << position);
                    Self(result)
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0 ^ ((1 as u32) << position);
                    Self(result)
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u32 {
                    let shifted = self.0 >> position;

                    let rest = size as u32 % (core::mem::size_of::<u32>() * 8) as u32;
                    let bit = (rest > 0) as u32;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u32) -> Self {
                    let rest = size as u32 % (core::mem::size_of::<u32>() * 8) as u32;
                    let bit = (rest > 0) as u32;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });
        assert_compare!(generate_accessors_low, "NonZero32", "struct C(D);", quote::quote! {
            impl C {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0.get() >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0.get() & !(1 << position);
                    let result = cleared | ((value as u32) << position);
                    Self(unsafe { core::num::NonZeroU32::new_unchecked(result) })
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0.get() ^ ((1 as u32) << position);
                    Self(unsafe { core::num::NonZeroU32::new_unchecked(result) })
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u32 {
                    let shifted = self.0.get() >> position;

                    let rest = size as u32 % (core::mem::size_of::<u32>() * 8) as u32;
                    let bit = (rest > 0) as u32;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u32) -> Self {
                    let rest = size as u32 % (core::mem::size_of::<u32>() * 8) as u32;
                    let bit = (rest > 0) as u32;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0.get() & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(unsafe { core::num::NonZeroU32::new_unchecked(result) })
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
                    let result = cleared | ((value as u64) << position);
                    Self(result)
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0 ^ ((1 as u64) << position);
                    Self(result)
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u64 {
                    let shifted = self.0 >> position;

                    let rest = size as u64 % (core::mem::size_of::<u64>() * 8) as u64;
                    let bit = (rest > 0) as u64;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u64) -> Self {
                    let rest = size as u64 % (core::mem::size_of::<u64>() * 8) as u64;
                    let bit = (rest > 0) as u64;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });
        assert_compare!(generate_accessors_low, "NonZero64", "struct D(E);", quote::quote! {
            impl D {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0.get() >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0.get() & !(1 << position);
                    let result = cleared | ((value as u64) << position);
                    Self(unsafe { core::num::NonZeroU64::new_unchecked(result) })
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0.get() ^ ((1 as u64) << position);
                    Self(unsafe { core::num::NonZeroU64::new_unchecked(result) })
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u64 {
                    let shifted = self.0.get() >> position;

                    let rest = size as u64 % (core::mem::size_of::<u64>() * 8) as u64;
                    let bit = (rest > 0) as u64;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u64) -> Self {
                    let rest = size as u64 % (core::mem::size_of::<u64>() * 8) as u64;
                    let bit = (rest > 0) as u64;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0.get() & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(unsafe { core::num::NonZeroU64::new_unchecked(result) })
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
                    let result = cleared | ((value as u128) << position);
                    Self(result)
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0 ^ ((1 as u128) << position);
                    Self(result)
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u128 {
                    let shifted = self.0 >> position;

                    let rest = size as u128 % (core::mem::size_of::<u128>() * 8) as u128;
                    let bit = (rest > 0) as u128;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u128) -> Self {
                    let rest = size as u128 % (core::mem::size_of::<u128>() * 8) as u128;
                    let bit = (rest > 0) as u128;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0 & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(result)
                }
            }
        });
        assert_compare!(generate_accessors_low, "NonZero128", "struct E(F);", quote::quote! {
            impl E {
                /// Returns a boolean value whether the specified flag is set.
                #[inline(always)]
                const fn _bit(&self, position: u8) -> bool {
                    ((self.0.get() >> position) & 1) != 0
                }

                /// Returns a modified instance with the flag set to the specified value.
                #[inline(always)]
                const fn _set_bit(&self, position: u8, value: bool) -> Self {
                    let cleared = self.0.get() & !(1 << position);
                    let result = cleared | ((value as u128) << position);
                    Self(unsafe { core::num::NonZeroU128::new_unchecked(result) })
                }

                /// Returns a modified instance with the bit value inverted.
                #[inline(always)]
                const fn _invert_bit(&self, position: u8) -> Self {
                    let result = self.0.get() ^ ((1 as u128) << position);
                    Self(unsafe { core::num::NonZeroU128::new_unchecked(result) })
                }

                /// Returns a field (subset of bits) from the internal value.
                #[inline(always)]
                const fn _field(&self, position: u8, size: u8) -> u128 {
                    let shifted = self.0.get() >> position;

                    let rest = size as u128 % (core::mem::size_of::<u128>() * 8) as u128;
                    let bit = (rest > 0) as u128;

                    let limit = bit.wrapping_shl(rest as u32);
                    let mask = limit.wrapping_sub((size > 0) as _);
                    let result = shifted & mask;

                    result
                }

                /// Returns a modified variant with the field set to the specified value.
                #[inline(always)]
                const fn _set_field(&self, position: u8, size: u8, value: u128) -> Self {
                    let rest = size as u128 % (core::mem::size_of::<u128>() * 8) as u128;
                    let bit = (rest > 0) as u128;

                    let limit = bit.wrapping_shl(rest as u32);
                    let negative_mask = limit.wrapping_sub((size > 0) as _);
                    let positioned_used_bits = negative_mask << position;
                    let positioned_mask = !positioned_used_bits;
                    let cleared = self.0.get() & positioned_mask;

                    let shifted_value = value << position;

                    let result = cleared | shifted_value;

                    Self(unsafe { core::num::NonZeroU128::new_unchecked(result) })
                }
            }
        });
    }

    #[test]
    fn assertions() {
        let non_zero_check = quote::quote! {
            const _OPTION_OF_NON_ZERO_BITFIELD_HAS_A_DIFFERENT_SIZE: [(); 0 - !{
                const ASSERT: bool =
                    core::mem::size_of::<A>()
                        ==
                    core::mem::size_of::<core::option::Option<A>>()
                ; ASSERT
            } as usize] = [];
        };

        assert_compare!(generate_assertions,
            "8", "struct A {}",
            quote::quote! {}
        );
        assert_compare!(generate_assertions,
            "NonZero8", "struct A {}",
            quote::quote!(impl A { #non_zero_check })
        );

        let check_1 = quote::quote! {
            const _TYPE_IN_FIELD_0_IS_SMALLER_THAN_THE_SPECIFIED_SIZE_OF_9_BITS: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<B>() * 8 >= 9;
                ASSERT
            } as usize] = [];

            const _SIGNED_TYPE_IN_FIELD_0_CAN_NEVER_BE_NEGATIVE: [(); 0 - !{
                const ASSERT: bool = !B::is_signed() || core::mem::size_of::<B>() * 8 == 9;
                ASSERT
            } as usize] = [];
        };
        assert_compare!(generate_assertions,
            "16", "struct A(#[field(0, 9)] B);",
            quote::quote!(impl A { #check_1 })
        );
        assert_compare!(generate_assertions,
            "NonZero16", "struct A(#[field(0, 9)] B);",
            quote::quote!(impl A { #check_1 #non_zero_check })
        );

        assert_compare!(
            generate_assertions,
            "16, allow_overlaps", "struct A(#[field(0, 9)] B);",
            quote::quote!(impl A { #check_1 })
        );
        assert_compare!(
            generate_assertions,
            "NonZero16, allow_overlaps", "struct A(#[field(0, 9)] B);",
            quote::quote!(impl A { #check_1 #non_zero_check })
        );

        let check_2 = quote::quote! {
            const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<B>() == 1;
                ASSERT
            } as usize] = [];

            const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<u16>() * 8 > B::max() as usize;
                ASSERT
            } as usize] = [];
        };
        assert_compare!(generate_assertions,
            "16", "struct A(B);",
            quote::quote!(impl A { #check_2 })
        );
        assert_compare!(generate_assertions,
            "16, allow_overlaps", "struct A(B);",
            quote::quote!(impl A { #check_2 })
        );
        let check_2_non_zero = quote::quote! {
            const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<B>() == 1;
                ASSERT
            } as usize] = [];

            const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<core::num::NonZeroU16>() * 8 > B::max() as usize;
                ASSERT
            } as usize] = [];
        };
        assert_compare!(generate_assertions,
            "NonZero16", "struct A(B);",
            quote::quote!(impl A { #check_2_non_zero #non_zero_check })
        );
        assert_compare!(generate_assertions,
            "NonZero16, allow_overlaps", "struct A(B);",
            quote::quote!(impl A { #check_2_non_zero #non_zero_check })
        );

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
        let check_3_non_zero = quote::quote! {
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
                const ASSERT: bool = core::mem::size_of::<core::num::NonZeroU8>() * 8 > C::max() as usize;
                ASSERT
            } as usize] = [];
        };
        assert_compare!(
            generate_assertions, "NonZero8", "struct A { #[field(0, 2)] b: B, c: C }", quote::quote! {
                impl A {
                    #check_3_non_zero

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

                    #non_zero_check
                }
            }
        );
        assert_compare!(
            generate_assertions, "NonZero8, allow_overlaps", "struct A { #[field(0, 2)] b: B, c: C }",
            quote::quote! { impl A { #check_3_non_zero #non_zero_check } }
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
        let check_4_non_zero = quote::quote! {
            const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<B>() == 1;
                ASSERT
            } as usize] = [];

            const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<core::num::NonZeroU8>() * 8 > B::max() as usize;
                ASSERT
            } as usize] = [];
        };
        let check_5_non_zero = quote::quote! {
            const _FLAGS_IN_FIELD_1_MUST_BE_REPR_U8: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<C>() == 1;
                ASSERT
            } as usize] = [];

            const _FLAGS_IN_FIELD_1_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                const ASSERT: bool = core::mem::size_of::<core::num::NonZeroU8>() * 8 > C::max() as usize;
                ASSERT
            } as usize] = [];
        };
        assert_compare!(
            generate_assertions, "NonZero8", "struct A { b: B, c: C }", quote::quote! {
                impl A {
                    #check_4_non_zero

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

                    #check_5_non_zero

                    #non_zero_check
                }
            }
        );
        assert_compare!(
            generate_assertions, "NonZero8, allow_overlaps", "struct A { b: B, c: C }", quote::quote! {
                impl A {
                    #check_4_non_zero
                    #check_5_non_zero
                    #non_zero_check
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
            generate_assertions, "NonZeroSize", "struct A(B);", quote::quote! {
                impl A {
                    const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<B>() == 1;
                        ASSERT
                    } as usize] = [];

                    const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<core::num::NonZeroUsize>() * 8 > B::max() as usize;
                        ASSERT
                    } as usize] = [];

                    #non_zero_check
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
        assert_compare!(
            generate_assertions, "NonZeroSize", "struct A(#[field(4, 3)] B);", quote::quote! {
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
                        const ASSERT: bool = core::mem::size_of::<core::num::NonZeroUsize>() * 8 >= 4 + 3;
                        ASSERT
                    } as usize] = [];

                    const _FIELD_0_HAS_THE_SIZE_OF_THE_WHOLE_BIT_FIELD: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<core::num::NonZeroUsize>() * 8 != 3;
                        ASSERT
                    } as usize] = [];

                    #non_zero_check
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
        assert_compare!(generate_impl, "NonZero8", "struct A(A);", quote::quote! {
            impl A {
                /// Creates a new instance with all flags and fields cleared.
                #[inline(always)]
                const fn new() -> Self {
                    Self(unsafe { core::num::NonZeroU8::new_unchecked(0) })
                }
            }
        });
    }

    #[test]
    fn struct_bit() {
        assert_compare!(generate_struct, "8", "struct A(A);", quote::quote! {
            #[repr(transparent)]
            struct A(u8);
        });
        assert_compare!(generate_struct, "NonZero8", "struct A(A);", quote::quote! {
            #[repr(transparent)]
            struct A(core::num::NonZeroU8);
        });

        assert_compare!(generate_struct, "16", "struct A(A);", quote::quote! {
            #[repr(transparent)]
            struct A(u16);
        });
        assert_compare!(generate_struct, "NonZero16", "struct A(A);", quote::quote! {
            #[repr(transparent)]
            struct A(core::num::NonZeroU16);
        });
    }

    #[test]
    fn struct_attrs() {
        assert_compare!(
            generate_struct, "8", "#[some_attribute1] #[some_attribute2] struct A(A);",
            quote::quote! {
                #[repr(transparent)]
                #[some_attribute1]
                #[some_attribute2]
                struct A(u8);
            }
        );
    }

    #[test]
    fn struct_vis() {
        assert_compare!(generate_struct, "8", "struct A(A);", quote::quote! {
            #[repr(transparent)]
            struct A(u8);
        });

        assert_compare!(generate_struct, "8", "pub struct A(A);", quote::quote! {
            #[repr(transparent)]
            pub struct A(u8);
        });
    }

    #[test]
    fn struct_ident() {
        assert_compare!(generate_struct, "8", "struct A(A);", quote::quote! {
            #[repr(transparent)]
            struct A(u8);
        });

        assert_compare!(generate_struct, "8", "struct B(A);", quote::quote! {
            #[repr(transparent)]
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
                #[repr(transparent)]
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
                        let result = cleared | ((value as u16) << position);
                        Self(result)
                    }

                    /// Returns a modified instance with the bit value inverted.
                    #[inline(always)]
                    const fn _invert_bit(&self, position: u8) -> Self {
                        let result = self.0 ^ ((1 as u16) << position);
                        Self(result)
                    }

                    /// Returns a field (subset of bits) from the internal value.
                    #[inline(always)]
                    const fn _field(&self, position: u8, size: u8) -> u16 {
                        let shifted = self.0 >> position;

                        let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                        let bit = (rest > 0) as u16;

                        let limit = bit.wrapping_shl(rest as u32);
                        let mask = limit.wrapping_sub((size > 0) as _);
                        let result = shifted & mask;

                        result
                    }

                    /// Returns a modified variant with the field set to the specified value.
                    #[inline(always)]
                    const fn _set_field(&self, position: u8, size: u8, value: u16) -> Self {
                        let rest = size as u16 % (core::mem::size_of::<u16> () * 8) as u16;
                        let bit = (rest > 0) as u16;

                        let limit = bit.wrapping_shl(rest as u32);
                        let negative_mask = limit.wrapping_sub((size > 0) as _);
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
                        self._bit(flag as _)
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
                        self._set_bit(flag as _, value)
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with all flags set.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn set_b_all(&self) -> Self {
                        let result = self.0 | Self::b_mask();
                        Self(result)
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with all flags cleared.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn set_b_none(&self) -> Self {
                        let result = self.0 & !Self::b_mask();
                        Self(result)
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with the value of the specified flag inverted.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn invert_b(&self, flag: B) -> Self {
                        self._invert_bit(flag as _)
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
                        self._set_field(7u8, 3u8, value as _)
                    }

                    #[doc = " D4 "]
                    /// Returns `true` if the specified `flag` is set.
                    #[inline(always)]
                    const fn d(&self, flag: D) -> bool {
                        self._bit(flag as _)
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
                        self._set_bit(flag as _, value)
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with all flags set.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn set_d_all(&self) -> Self {
                        let result = self.0 | Self::d_mask();
                        Self(result)
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with all flags cleared.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn set_d_none(&self) -> Self {
                        let result = self.0 & !Self::d_mask();
                        Self(result)
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with the value of the specified flag inverted.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn invert_d(&self, flag: D) -> Self {
                        self._invert_bit(flag as _)
                    }
                }

                // accessors ops flags
                impl core::ops::Add<B> for A {
                    type Output = Self;

                    #[inline(always)]
                    fn add(self, flag: B) -> Self::Output {
                        self.set_b(flag, true)
                    }
                }

                impl core::ops::AddAssign<B> for A {
                    #[inline(always)]
                    fn add_assign(&mut self, flag: B) {
                        self.0 = self.set_b(flag, true).0;
                    }
                }

                impl core::ops::BitXor<B> for A {
                    type Output = Self;

                    #[inline(always)]
                    fn bitxor(self, flag: B) -> Self::Output {
                        self.invert_b(flag)
                    }
                }

                impl core::ops::BitXorAssign<B> for A {
                    #[inline(always)]
                    fn bitxor_assign(&mut self, flag: B) {
                        self.0 = self.invert_b(flag).0;
                    }
                }

                impl core::ops::Sub<B> for A {
                    type Output = Self;

                    #[inline(always)]
                    fn sub(self, flag: B) -> Self::Output {
                        self.set_b(flag, false)
                    }
                }

                impl core::ops::SubAssign<B> for A {
                    #[inline(always)]
                    fn sub_assign(&mut self, flag: B) {
                        self.0 = self.set_b(flag, false).0;
                    }
                }

                impl core::ops::Add<C> for A {
                    type Output = Self;

                    #[inline(always)] fn add(self, value: C) -> Self::Output {
                        self.set_c(value)
                    }
                }

                impl core::ops::AddAssign<C> for A {
                    #[inline(always)]
                    fn add_assign(&mut self, value: C) {
                        self.0 = self.set_c(value).0;
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
        assert_eq!(
            Into::<proc_macro2::TokenStream>::into(parse_valid!(
                "NonZero16", "/** D1 */ #[derive(Debug)] pub(crate) struct A { /** D2 */ pub(crate) b: B, /** D3 */ #[field(7, 3)] pub c: C, /** D4 */ d: D }"
            )).to_string(),
            quote::quote! {
                // field
                #[repr(transparent)]
                #[doc = " D1 "]
                pub(crate) struct A(core::num::NonZeroU16);

                // implementation
                impl A {
                    /// Creates a new instance with all flags and fields cleared.
                    #[inline(always)]
                    pub(crate) const fn new() -> Self {
                        Self(unsafe { core::num::NonZeroU16::new_unchecked(0) })
                    }
                }

                // accessors_low
                impl A {
                    /// Returns a boolean value whether the specified flag is set.
                    #[inline(always)]
                    const fn _bit(&self, position: u8) -> bool {
                        ((self.0.get() >> position) & 1) != 0
                    }

                    /// Returns a modified instance with the flag set to the specified value.
                    #[inline(always)]
                    const fn _set_bit(&self, position: u8, value: bool) -> Self {
                        let cleared = self.0.get() & !(1 << position);
                        let result = cleared | ((value as u16) << position);
                        Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                    }

                    /// Returns a modified instance with the bit value inverted.
                    #[inline(always)]
                    const fn _invert_bit(&self, position: u8) -> Self {
                        let result = self.0.get() ^ ((1 as u16) << position);
                        Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                    }

                    /// Returns a field (subset of bits) from the internal value.
                    #[inline(always)]
                    const fn _field(&self, position: u8, size: u8) -> u16 {
                        let shifted = self.0.get() >> position;

                        let rest = size as u16 % (core::mem::size_of::<u16>() * 8) as u16;
                        let bit = (rest > 0) as u16;

                        let limit = bit.wrapping_shl(rest as u32);
                        let mask = limit.wrapping_sub((size > 0) as _);
                        let result = shifted & mask;

                        result
                    }

                    /// Returns a modified variant with the field set to the specified value.
                    #[inline(always)]
                    const fn _set_field(&self, position: u8, size: u8, value: u16) -> Self {
                        let rest = size as u16 % (core::mem::size_of::<u16> () * 8) as u16;
                        let bit = (rest > 0) as u16;

                        let limit = bit.wrapping_shl(rest as u32);
                        let negative_mask = limit.wrapping_sub((size > 0) as _);
                        let positioned_used_bits = negative_mask << position;
                        let positioned_mask = !positioned_used_bits;
                        let cleared = self.0.get() & positioned_mask;

                        let shifted_value = value << position;

                        let result = cleared | shifted_value;

                        Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                    }
                }

                // accessors
                impl A {
                    #[doc = " D2 "]
                    /// Returns `true` if the specified `flag` is set.
                    #[inline(always)]
                    pub(crate) const fn b(&self, flag: B) -> bool {
                        self._bit(flag as _)
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
                        (self.0.get() & Self::b_mask()) == Self::b_mask()
                    }

                    #[doc = " D2 "]
                    /// Returns `true` if any flag is set.
                    #[inline(always)]
                    pub(crate) const fn b_any(&self) -> bool {
                        (self.0.get() & Self::b_mask()) != 0
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with the new value for the specified flag.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn set_b(&self, flag: B, value: bool) -> Self {
                        self._set_bit(flag as _, value)
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with all flags set.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn set_b_all(&self) -> Self {
                        let result = self.0.get() | Self::b_mask();
                        Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with all flags cleared.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn set_b_none(&self) -> Self {
                        let result = self.0.get() & !Self::b_mask();
                        Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                    }

                    #[doc = " D2 "]
                    /// Creates a copy of the bit field with the value of the specified flag inverted.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    pub(crate) const fn invert_b(&self, flag: B) -> Self {
                        self._invert_bit(flag as _)
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
                        self._set_field(7u8, 3u8, value as _)
                    }

                    #[doc = " D4 "]
                    /// Returns `true` if the specified `flag` is set.
                    #[inline(always)]
                    const fn d(&self, flag: D) -> bool {
                        self._bit(flag as _)
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
                        (self.0.get() & Self::d_mask()) == Self::d_mask()
                    }

                    #[doc = " D4 "]
                    /// Returns `true` if any flag is set.
                    #[inline(always)]
                    const fn d_any(&self) -> bool {
                        (self.0.get() & Self::d_mask()) != 0
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with the new value for the specified flag.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn set_d(&self, flag: D, value: bool) -> Self {
                        self._set_bit(flag as _, value)
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with all flags set.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn set_d_all(&self) -> Self {
                        let result = self.0.get() | Self::d_mask();
                        Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with all flags cleared.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn set_d_none(&self) -> Self {
                        let result = self.0.get() & !Self::d_mask();
                        Self(unsafe { core::num::NonZeroU16::new_unchecked(result) })
                    }

                    #[doc = " D4 "]
                    /// Creates a copy of the bit field with the value of the specified flag inverted.
                    #[inline(always)]
                    #[must_use = "leaves `self` unmodified and returns a modified variant"]
                    const fn invert_d(&self, flag: D) -> Self {
                        self._invert_bit(flag as _)
                    }
                }

                // accessors ops flags
                impl core::ops::Add<B> for A {
                    type Output = Self;

                    #[inline(always)]
                    fn add(self, flag: B) -> Self::Output {
                        self.set_b(flag, true)
                    }
                }

                impl core::ops::AddAssign<B> for A {
                    #[inline(always)]
                    fn add_assign(&mut self, flag: B) {
                        self.0 = self.set_b(flag, true).0;
                    }
                }

                impl core::ops::BitXor<B> for A {
                    type Output = Self;

                    #[inline(always)]
                    fn bitxor(self, flag: B) -> Self::Output {
                        self.invert_b(flag)
                    }
                }

                impl core::ops::BitXorAssign<B> for A {
                    #[inline(always)]
                    fn bitxor_assign(&mut self, flag: B) {
                        self.0 = self.invert_b(flag).0;
                    }
                }

                impl core::ops::Sub<B> for A {
                    type Output = Self;

                    #[inline(always)]
                    fn sub(self, flag: B) -> Self::Output {
                        self.set_b(flag, false)
                    }
                }

                impl core::ops::SubAssign<B> for A {
                    #[inline(always)]
                    fn sub_assign(&mut self, flag: B) {
                        self.0 = self.set_b(flag, false).0;
                    }
                }

                impl core::ops::Add<C> for A {
                    type Output = Self;

                    #[inline(always)] fn add(self, value: C) -> Self::Output {
                        self.set_c(value)
                    }
                }

                impl core::ops::AddAssign<C> for A {
                    #[inline(always)]
                    fn add_assign(&mut self, value: C) {
                        self.0 = self.set_c(value).0;
                    }
                }

                // assertions
                impl A {
                    const _FLAGS_IN_FIELD_0_MUST_BE_REPR_U8: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<B>() == 1; ASSERT
                    } as usize] = [];

                    const _FLAGS_IN_FIELD_0_EXCEED_THE_BIT_FIELD_SIZE: [(); 0 - !{
                        const ASSERT: bool = core::mem::size_of::<core::num::NonZeroU16>() * 8 > B::max() as usize;
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
                        const ASSERT: bool = core::mem::size_of::<core::num::NonZeroU16>() * 8 > D::max() as usize;
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

                    const _OPTION_OF_NON_ZERO_BITFIELD_HAS_A_DIFFERENT_SIZE: [(); 0 - !{
                        const ASSERT: bool =
                            core::mem::size_of::<A>()
                                ==
                            core::mem::size_of::<core::option::Option<A>>()
                        ; ASSERT
                    } as usize] = [];
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