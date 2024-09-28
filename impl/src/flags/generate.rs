//! Contains code to generate bit field flags.

impl super::Flags {
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

    /// Generates a `const fn max() -> Self` implementation.
    fn generate_max(&self) -> proc_macro2::TokenStream {
        let first = &self.0.variants.first().unwrap();
        let vis = &self.0.vis;

        quote::quote! {
            /// Returns the flag with the highest bit value.
            #[inline(always)]
            #vis const fn max() -> Self {
                let mut i = 0;
                let mut max = Self::#first;

                while i < Self::iter().len() {
                    let current = Self::iter()[i];
                    if current as u8 > max as u8 {
                        max = current;
                    }

                    i += 1;
                }

                max
            }
        }
    }
}

/// Generates the user code for the parsed flags of a bit field.
impl core::convert::Into<proc_macro2::TokenStream> for super::Flags {
    fn into(self) -> proc_macro2::TokenStream {
        let ident = &self.0.ident;

        let iter = self.generate_iter();
        let max = self.generate_max();

        quote::quote! {
            impl #ident {
                #iter
                #max
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    macro_rules! assert_compare {
        ($generator:ident, $item:expr, $result:expr) => {{
            let flags = Flags(parse_valid!($item)).$generator().to_string();
            let expected = $result.to_string();

            assert_eq!(&flags, &expected);
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
    fn max() {
        assert_compare!(generate_max, "#[repr(u8)] pub enum A { B }", quote::quote! {
            /// Returns the flag with the highest bit value.
            #[inline(always)]
            pub const fn max() -> Self {
                let mut i = 0;
                let mut max = Self::B;

                while i < Self::iter().len() {
                    let current = Self::iter()[i];
                    if current as u8 > max as u8 {
                        max = current;
                    }

                    i += 1;
                }

                max
            }
        });

        assert_compare!(generate_max, "#[repr(u8)] enum B { C, D = 5, E, F = 4 }", quote::quote! {
            /// Returns the flag with the highest bit value.
            #[inline(always)]
            const fn max() -> Self {
                let mut i = 0;
                let mut max = Self::C;

                while i < Self::iter().len() {
                    let current = Self::iter()[i];
                    if current as u8 > max as u8 {
                        max = current;
                    }

                    i += 1;
                }

                max
            }
        });
    }

    #[test]
    fn everything() {
        assert_eq!(
            Into::<proc_macro2::TokenStream>::into(
                Flags(parse_valid!("#[repr(u8)] enum C { D, E = 3, F }"))
            ).to_string(),
            quote::quote! {
                impl C {
                    /// Returns an array containing all enumeration variants in the defined order.
                    #[inline(always)]
                    const fn iter() -> &'static [Self] {
                        &[
                            Self::D,
                            Self::E,
                            Self::F
                        ]
                    }

                    /// Returns the flag with the highest bit value.
                    #[inline(always)]
                    const fn max() -> Self {
                        let mut i = 0;
                        let mut max = Self::D;

                        while i < Self::iter().len() {
                            let current = Self::iter()[i];
                            if current as u8 > max as u8 {
                                max = current;
                            }

                            i += 1;
                        }

                        max
                    }
                }
            }.to_string()
        );
    }
}