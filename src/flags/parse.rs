//! Contains code to parse bit field flags.

impl super::Flags {
    pub(crate) fn parse(item: proc_macro2::TokenStream) -> syn::Result<Self> {
        let enumeration = crate::enumeration::Enumeration::parse(item)?;

        if enumeration.repr != "u8" {
            return Err(syn::Error::new(enumeration.repr.span(), "expected `u8`"))
        }

        Ok(Self(enumeration))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    macro_rules! parse_invalid {
        ($item:expr, $message:expr, ($sl:expr, $sc:expr), ($el:expr, $ec:expr)) => {{
            let error = Flags::parse($item.parse().unwrap()).map(|_| ()).unwrap_err();
            assert_eq!(error.to_string(), $message);
            compare_span!(error.span(), ($sl, $sc), ($el, $ec));
        }}
    }

    macro_rules! parse_valid {
        ($item:expr) => {
            Flags::parse($item.parse().unwrap()).unwrap()
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
    fn repr() {
        parse_invalid!(
            "#[repr(u16)] enum A { B }",
            "expected `u8`",
            (1, 7), (1, 10)
        );

        parse_valid!("#[repr(u8)] enum A { B }");
    }
}