//! Contains primitive type related helper functions.

pub(crate) fn is_primitive(ident: &syn::Ident) -> bool {
    is_bool(ident) || is_signed_primitive(ident) || is_unsigned_primitive(ident)
}

pub(crate) fn is_numeric_primitive(ident: &syn::Ident) -> bool {
    is_signed_primitive(ident) || is_unsigned_primitive(ident)
}

pub(crate) fn is_bool(ident: &syn::Ident) -> bool {
    ident == "bool"
}

pub(crate) fn is_signed_primitive(ident: &syn::Ident) -> bool {
    ident == "i8" || ident == "i16" || ident == "i32" || ident == "i64" || ident == "i128"
}

pub(crate) fn is_unsigned_primitive(ident: &syn::Ident) -> bool {
    ident == "u8" || ident == "u16" || ident == "u32" || ident == "u64" || ident == "u128"
}

pub(crate) fn primitive_bits(ident: &syn::Ident) -> Option<u8> {
    if ident == "bool" { Some(1) }
    else if ident == "i8" || ident == "u8" { Some(8) }
    else if ident == "i16" || ident == "u16" { Some(16) }
    else if ident == "i32" || ident == "u32" { Some(32) }
    else if ident == "i64" || ident == "u64" { Some(64) }
    else if ident == "i128" || ident == "u128" { Some(128) }
    else { None }
}

/// Returns the narrowest primitive type size to store `bits`.
pub(crate) fn field_primitive_size(bits: u8) -> u8 {
    const SIZES: &[u8] = &[8, 16, 32, 64, 128];

    for s in SIZES {
        if bits <= *s { return *s; }
    }

    unimplemented!("field size {} > {}", bits, SIZES.last().unwrap());
}

/// Returns the narrowest primitive type to store `bits`.
pub(crate) fn type_from_bits(bits: u8, is_signed: bool, span: proc_macro2::Span) -> syn::Ident {
    syn::Ident::new(&format!("{}{}", if is_signed { 'i' } else { 'u' }, field_primitive_size(bits)), span)
}

#[cfg(test)]
mod tests {
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
            assert_eq!(super::field_primitive_size(size.0), size.1);
        }
    }

    #[test]
    fn type_from_bits() {
        const BITS: &[u8] = &[1, 7, 8, 9, 15, 16, 17];
        const IS_SIGNED: &[bool] = &[false, true];

        const RESULTS: &[&str] = &[
            "u8", "i8",
            "u8", "i8",
            "u8", "i8",
            "u16", "i16",
            "u16", "i16",
            "u16", "i16",
            "u32", "i32"
        ];

        let span = proc_macro2::Span::call_site();
        let mut i = 0;
        for bits in BITS {
            for is_signed in IS_SIGNED {
                assert_eq!(super::type_from_bits(*bits, *is_signed, span), RESULTS[i]);
                i += 1;
            }
        }
    }
}