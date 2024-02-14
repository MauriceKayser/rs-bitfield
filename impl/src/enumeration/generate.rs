//! Contains code to generate bit field fields.

impl super::Enumeration {
    /// Generates a `core::convert::TryFrom<REPR, Error = REPR>` implementation.
    pub fn generate_try_from(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let repr = &self.repr;
        let span = repr.span();
        let variants = &self.variants;

        // TODO: Add `const` when https://github.com/rust-lang/rfcs/pull/2632 is merged.
        quote::quote_spanned!(span =>
            impl core::convert::TryFrom<#repr> for #ident {
                type Error = #repr;

                #[allow(non_upper_case_globals)]
                #[inline(always)]
                fn try_from(value: #repr) -> core::result::Result<
                    Self, <Self as core::convert::TryFrom<#repr>>::Error
                > {
                    #(const #variants: #repr = #ident::#variants as #repr;)*
                    match value {
                        #(#variants)|* => core::result::Result::Ok(unsafe {
                            *(&value as *const #repr as *const Self)
                        }),
                        _ => core::result::Result::Err(value)
                    }
                }
            }
        )
    }
}

#[cfg(test)]
mod tests {
    macro_rules! assert_compare {
        ($generator:ident, $item:expr, $result:expr) => {{
            let field = parse_valid!($item).$generator().to_string();
            let expected = $result.to_string();

            assert_eq!(&field, &expected);
        }};
    }

    // Test macros.

    #[test]
    #[should_panic]
    fn test_assert_compare() {
        assert_compare!(generate_try_from, "#[repr(u8)] enum A { B }", quote::quote! {});
    }

    // Test generation.

    #[test]
    fn try_from() {
        assert_compare!(generate_try_from, "#[repr(u8)] enum A { B }", quote::quote! {
            impl core::convert::TryFrom<u8> for A {
                type Error = u8;

                #[allow(non_upper_case_globals)]
                #[inline(always)]
                fn try_from(value: u8) -> core::result::Result<
                    Self, <Self as core::convert::TryFrom<u8>>::Error
                > {
                    const B: u8 = A::B as u8;

                    match value {
                        B => core::result::Result::Ok(unsafe {
                            *(&value as *const u8 as *const Self)
                        }),
                        _ => core::result::Result::Err(value)
                    }
                }
            }
        });

        assert_compare!(generate_try_from, "#[repr(u16)] enum B { A }", quote::quote! {
            impl core::convert::TryFrom<u16> for B {
                type Error = u16;

                #[allow(non_upper_case_globals)]
                #[inline(always)]
                fn try_from(value: u16) -> core::result::Result<
                    Self, <Self as core::convert::TryFrom<u16>>::Error
                > {
                    const A: u16 = B::A as u16;

                    match value {
                        A => core::result::Result::Ok(unsafe {
                            *(&value as *const u16 as *const Self)
                        }),
                        _ => core::result::Result::Err(value)
                    }
                }
            }
        });

        assert_compare!(generate_try_from, "#[repr(u8)] enum A { B = 3, C }", quote::quote! {
            impl core::convert::TryFrom<u8> for A {
                type Error = u8;

                #[allow(non_upper_case_globals)]
                #[inline(always)]
                fn try_from(value: u8) -> core::result::Result<
                    Self, <Self as core::convert::TryFrom<u8>>::Error
                > {
                    const B: u8 = A::B as u8;
                    const C: u8 = A::C as u8;

                    match value {
                        B | C => core::result::Result::Ok(unsafe {
                            *(&value as *const u8 as *const Self)
                        }),
                        _ => core::result::Result::Err(value)
                    }
                }
            }
        });
    }
}