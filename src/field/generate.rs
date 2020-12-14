//! Contains code to generate bit field fields.

impl super::Field {
    /// Generates a `core::convert::TryFrom<REPR, Error = REPR>` implementation.
    fn generate_try_from(&self) -> proc_macro2::TokenStream {
        let ident = &self.0.ident;
        let repr = &self.0.repr;
        let variants = &self.0.variants;

        quote::quote!(
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

/// Generates the user code for the parsed field of a bit field.
impl core::convert::Into<proc_macro2::TokenStream> for super::Field {
    fn into(self) -> proc_macro2::TokenStream {
        let try_from = self.generate_try_from();

        quote::quote! {
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

    #[test]
    fn everything() {
        assert_eq!(
            Into::<proc_macro2::TokenStream>::into(
                Field(parse_valid!("#[repr(u8)] enum C { D }"))
            ).to_string(),
            quote::quote! {
                impl core::convert::TryFrom<u8> for C {
                    type Error = u8;

                    #[allow(non_upper_case_globals)]
                    #[inline(always)]
                    fn try_from(value: u8) -> core::result::Result<
                        Self, <Self as core::convert::TryFrom<u8>>::Error
                    > {
                        const D: u8 = C::D as u8;

                        match value {
                            D => core::result::Result::Ok(unsafe {
                                *(&value as *const u8 as *const Self)
                            }),
                            _ => core::result::Result::Err(value)
                        }
                    }
                }
            }.to_string()
        );
    }
}