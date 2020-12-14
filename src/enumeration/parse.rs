use syn::spanned::Spanned;

impl super::Enumeration {
    pub(crate) fn parse(item: proc_macro2::TokenStream) -> syn::Result<Self> {
        let input: syn::DeriveInput = syn::parse2(item)?;

        // Get the primitive type of the `repr` attribute.
        let repr = match input.attrs.iter()
            .find(|attr| attr.path.is_ident("repr"))
            .ok_or_else(|| syn::Error::new(input.span(), "expected `repr` attribute"))?
            .parse_meta()?
        {
            syn::Meta::List(list) => match list.nested.first().unwrap() {
                syn::NestedMeta::Meta(meta) => Ok(meta.path().get_ident().unwrap().clone()),
                syn::NestedMeta::Lit(lit) => Err(syn::Error::new(
                    lit.span(), "expected identifier"
                ))
            },
            syn::Meta::NameValue(value) => Err(syn::Error::new(
                value.span(), "expected list"
            )),
            syn::Meta::Path(path) => Err(syn::Error::new(
                path.span(), "expected list"
            ))
        }?;

        if repr != "u8" && repr != "u16" && repr != "u32" && repr != "u64" && repr != "u128" {
            return Err(syn::Error::new(
                repr.span(), "expected numerical representation"
            ));
        }

        Ok(Self {
            variants: match input.data {
                syn::Data::Enum(e) => {
                    if e.variants.is_empty() { return Err(syn::Error::new(
                        e.brace_token.span, "expected variants"
                    )) }

                    e.variants.into_iter().map(|v| v.ident).collect()
                },

                syn::Data::Struct(s) => return Err(syn::Error::new(
                    s.struct_token.span(), "expected enum"
                )),

                syn::Data::Union(u) => return Err(syn::Error::new(
                    u.union_token.span(), "expected enum"
                ))
            },
            repr,
            vis: input.vis,
            ident: input.ident
        })
    }
}

#[cfg(test)]
#[macro_use]
mod tests {
    macro_rules! parse_invalid {
        ($item:expr, $message:expr, ($sl:expr, $sc:expr), ($el:expr, $ec:expr)) => {{
            let error = crate::enumeration::Enumeration::parse($item.parse().unwrap()).map(|_| ()).unwrap_err();
            assert_eq!(error.to_string(), $message);
            compare_span!(error.span(), ($sl, $sc), ($el, $ec));
        }}
    }

    macro_rules! parse_valid {
        ($item:expr) => {
            crate::enumeration::Enumeration::parse($item.parse().unwrap()).unwrap()
        }
    }

    // Test macros.

    #[test]
    #[should_panic]
    fn test_parse_invalid() {
        parse_invalid!(
            "",
            "unexpected end of input, ...",
            (1, 0), (1, 0)
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_valid() {
        parse_valid!("fn a() {}");
    }

    // Test parsing.

    #[test]
    fn derive_input() {
        parse_invalid!(
            "fn a() {}",
            "expected one of: `struct`, `enum`, `union`",
            (1, 0), (1, 2)
        );

        parse_invalid!(
            "#[repr(u8)] enum A {}",
            "expected variants",
            (1, 19), (1, 21)
        );

        parse_invalid!(
            "#[repr(u8)] struct A {}",
            "expected enum",
            (1, 12), (1, 18)
        );

        parse_invalid!(
            "#[repr(u8)] union A {}",
            "expected enum",
            (1, 12), (1, 17)
        );
    }

    #[test]
    fn ident() {
        assert_eq!(parse_valid!("#[repr(u8)] enum A { B }").ident, "A");
        assert_eq!(parse_valid!("#[repr(u8)] enum B { A }").ident, "B");
    }

    #[test]
    fn repr() {
        parse_invalid!(
            "enum A { B }",
            "expected `repr` attribute",
            (1, 0), (1, 12)
        );

        parse_invalid!(
            "#[repr] enum A { B }",
            "expected list",
            (1, 2), (1, 6)
        );

        parse_invalid!(
            "#[repr[u8]] enum A { B }",
            "unexpected token",
            (1, 6), (1, 10)
        );

        parse_invalid!(
            "#[repr(\"u8\")] enum A { B }",
            "expected identifier",
            (1, 7), (1, 11)
        );

        parse_invalid!(
            "#[repr(u9)] enum A { B }",
            "expected numerical representation",
            (1, 7), (1, 9)
        );

        assert_eq!(parse_valid!("#[repr(u8)] enum A { B }").repr, "u8");
        assert_eq!(parse_valid!("#[repr(u16)] enum A { B }").repr, "u16");
        assert_eq!(parse_valid!("#[repr(u32)] enum A { B }").repr, "u32");
        assert_eq!(parse_valid!("#[repr(u64)] enum A { B }").repr, "u64");
        assert_eq!(parse_valid!("#[repr(u128)] enum A { B }").repr, "u128");
    }

    #[test]
    fn variants() {
        let variants = parse_valid!("#[repr(u8)] enum A { B }").variants;
        assert_eq!(variants.len(), 1);
        assert_eq!(variants.first().unwrap(), "B");

        let variants = parse_valid!("#[repr(u8)] enum A { B, C }").variants;
        assert_eq!(variants.len(), 2);
        assert_eq!(variants.first().unwrap(), "B");
        assert_eq!(variants.iter().skip(1).next().unwrap(), "C");
    }

    #[test]
    fn vis() {
        assert!(match parse_valid!("#[repr(u8)] enum A { B }").vis {
            syn::Visibility::Inherited => true,
            _ => false
        });

        assert!(match parse_valid!("#[repr(u8)] pub enum A { B }").vis {
            syn::Visibility::Public(_) => true,
            _ => false
        });
    }
}