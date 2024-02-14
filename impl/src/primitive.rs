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