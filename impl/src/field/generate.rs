//! Contains code to generate bit field fields.

impl super::Field {
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

    /// Generates a `const fn size() -> u8` implementation.
    fn generate_size(&self) -> proc_macro2::TokenStream {
        let vis = &self.0.vis;
        let repr = &self.0.repr;

        quote::quote!(
            /// Returns the amount of bits this type uses as a field.
            #[inline(always)]
            #vis const fn size() -> u8 {
                // Return the full type size if a variant has a negative discriminant.
                let mut i = 0;
                while i < Self::iter().len() {
                    if (Self::iter()[i] as #repr) < 0 {
                        return (core::mem::size_of::<Self>() * 8) as u8;
                    }
                    i += 1;
                }

                // Otherwise return the amount of bits used for the variant with the biggest discriminant.
                let mut max = Self::iter()[0] as #repr;

                i = 1;
                while i < Self::iter().len() {
                    let current = Self::iter()[i];

                    if current as #repr > max {
                        max = current as #repr;
                    }

                    i += 1;
                }

                match max {
                    0 => 1,
                    _ => max.ilog2() as u8 + 1
                }
            }
        )
    }
}

/// Generates the user code for the parsed field of a bit field.
impl core::convert::Into<proc_macro2::TokenStream> for super::Field {
    fn into(self) -> proc_macro2::TokenStream {
        let ident = &self.0.ident;

        let iter = self.generate_iter();
        let size = self.generate_size();
        let try_from = self.0.generate_try_from();

        quote::quote! {
            impl #ident {
                #iter
                #size
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
        assert_compare!(generate_iter, "#[repr(u8)] enum A { B }", quote::quote! {});
    }

    // Test generation.

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
    fn size() {
        assert_compare!(generate_size, "#[repr(u16)] pub enum A { B }", quote::quote! {
            /// Returns the amount of bits this type uses as a field.
            #[inline(always)]
            pub const fn size() -> u8 {
                let mut i = 0;
                while i < Self::iter().len() {
                    if (Self::iter()[i] as u16) < 0 {
                        return (core::mem::size_of::<Self>() * 8) as u8;
                    }
                    i += 1;
                }

                let mut max = Self::iter()[0] as u16;

                i = 1;
                while i < Self::iter().len() {
                    let current = Self::iter()[i];

                    if current as u16 > max {
                        max = current as u16;
                    }

                    i += 1;
                }

                match max {
                    0 => 1,
                    _ => max.ilog2() as u8 + 1
                }
            }
        });

        assert_compare!(generate_size, "#[repr(i16)] enum A { B }", quote::quote! {
            /// Returns the amount of bits this type uses as a field.
            #[inline(always)]
            const fn size() -> u8 {
                let mut i = 0;
                while i < Self::iter().len() {
                    if (Self::iter()[i] as i16) < 0 {
                        return (core::mem::size_of::<Self>() * 8) as u8;
                    }
                    i += 1;
                }

                let mut max = Self::iter()[0] as i16;

                i = 1;
                while i < Self::iter().len() {
                    let current = Self::iter()[i];

                    if current as i16 > max {
                        max = current as i16;
                    }

                    i += 1;
                }

                match max {
                    0 => 1,
                    _ => max.ilog2() as u8 + 1
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
                impl C {
                    /// Returns an array containing all enumeration variants in the defined order.
                    #[inline(always)]
                    const fn iter() -> &'static [Self] {
                        &[ Self::D ]
                    }

                    /// Returns the amount of bits this type uses as a field.
                    #[inline(always)]
                    const fn size() -> u8 {
                        let mut i = 0;
                        while i < Self::iter().len() {
                            if (Self::iter()[i] as u8) < 0 {
                                return (core::mem::size_of::<Self>() * 8) as u8;
                            }
                            i += 1;
                        }

                        let mut max = Self::iter()[0] as u8;

                        i = 1;
                        while i < Self::iter().len() {
                            let current = Self::iter()[i];

                            if current as u8 > max {
                                max = current as u8;
                            }

                            i += 1;
                        }

                        match max {
                            0 => 1,
                            _ => max.ilog2() as u8 + 1
                        }
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