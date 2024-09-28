//! Contains code to generate bit field fields.

impl super::Field {
    /// Generates a `const fn is_signed() -> bool` implementation.
    fn generate_is_signed(&self) -> proc_macro2::TokenStream {
        let is_signed = crate::primitive::is_signed_primitive(&self.0.repr);

        let vis = &self.0.vis;

        quote::quote!(
            /// Returns true if the enumeration is represented by a signed primitive type.
            #[inline(always)]
            #vis const fn is_signed() -> bool {
                #is_signed
            }
        )
    }

    /// Generates a `const fn iter() -> &'static [Self]` implementation.
    fn generate_iter(&self) -> proc_macro2::TokenStream {
        let variants = &self.0.variants;
        let vis = &self.0.vis;

        quote::quote!(
            /// Returns an array containing all enumeration variants in the defined order.
            #[inline(always)]
            #vis const fn iter() -> &'static [Self] {
                &[#(Self::#variants),*]
            }
        )
    }
}

/// Generates the user code for the parsed field of a bit field.
impl core::convert::Into<proc_macro2::TokenStream> for super::Field {
    fn into(self) -> proc_macro2::TokenStream {
        let ident = &self.0.ident;

        let is_signed = self.generate_is_signed();
        let iter = self.generate_iter();
        let try_from = self.0.generate_try_from();

        quote::quote! {
            impl #ident {
                #is_signed
                #iter
            }

            #try_from
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    macro_rules! assert_compare {
        ($generator:ident, $item:expr, $result:expr) => {{
            let field = Field(parse_valid!($item)).$generator().to_string();
            let expected = $result.to_string();

            assert_eq!(&field, &expected);
        }};
    }

    // Test macros.

    #[test]
    #[should_panic]
    fn test_assert_compare() {
        assert_compare!(generate_is_signed, "#[repr(u8)] enum A { B }", quote::quote! {});
    }

    // Test generation.

    #[test]
    fn is_signed() {
        assert_compare!(generate_is_signed, "#[repr(u8)] enum A { B }", quote::quote! {
            /// Returns true if the enumeration is represented by a signed primitive type.
            #[inline(always)]
            const fn is_signed() -> bool {
                false
            }
        });

        assert_compare!(generate_is_signed, "#[repr(u8)] pub enum A { B }", quote::quote! {
            /// Returns true if the enumeration is represented by a signed primitive type.
            #[inline(always)]
            pub const fn is_signed() -> bool {
                false
            }
        });

        assert_compare!(generate_is_signed, "#[repr(i8)] enum A { B }", quote::quote! {
            /// Returns true if the enumeration is represented by a signed primitive type.
            #[inline(always)]
            const fn is_signed() -> bool {
                true
            }
        });
    }

    #[test]
    fn iter() {
        assert_compare!(generate_iter, "#[repr(u8)] enum A { B }", quote::quote! {
            /// Returns an array containing all enumeration variants in the defined order.
            #[inline(always)]
            const fn iter() -> &'static [Self] {
                &[ Self::B ]
            }
        });

        assert_compare!(generate_iter, "#[repr(u8)] pub enum B { C, D = 6 }", quote::quote! {
            /// Returns an array containing all enumeration variants in the defined order.
            #[inline(always)]
            pub const fn iter() -> &'static [Self] {
                &[
                    Self::C,
                    Self::D
                ]
            }
        });
    }

    #[test]
    fn everything() {
        assert_eq!(
            Into::<proc_macro2::TokenStream>::into(
                Field(parse_valid!("#[repr(u8)] enum C { D }"))
            ).to_string(),
            quote::quote! {
                impl C {
                    /// Returns true if the enumeration is represented by a signed primitive type.
                    #[inline(always)]
                    const fn is_signed() -> bool {
                        false
                    }

                    /// Returns an array containing all enumeration variants in the defined order.
                    #[inline(always)]
                    const fn iter() -> &'static [Self] {
                        &[ Self::D ]
                    }
                }

                impl ::core::convert::TryFrom<u8> for C {
                    type Error = u8;

                    #[allow(non_upper_case_globals)]
                    #[inline(always)]
                    fn try_from(value: u8) -> ::core::result::Result<
                        Self, <Self as ::core::convert::TryFrom<u8>>::Error
                    > {
                        const D: u8 = C::D as u8;

                        match value {
                            D => ::core::result::Result::Ok(unsafe {
                                *(&value as *const u8 as *const Self)
                            }),
                            _ => ::core::result::Result::Err(value)
                        }
                    }
                }
            }.to_string()
        );
    }
}