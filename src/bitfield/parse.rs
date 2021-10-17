//! Contains code to parse bit fields.

use syn::spanned::Spanned;

impl syn::parse::Parse for super::Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Read the amount of bits the field should store.
        let (base_type, primitive_type, bits, is_non_zero) = if input.peek(syn::LitInt) {
            // Parse integer literals.
            input.parse::<syn::LitInt>()
                .map_err(|e| e.span())
                .and_then(|lit| lit.base10_parse().map(|bits| (bits, lit.span())).map_err(|e| e.span()))
                .and_then(|(bits, span)| match bits {
                    8 | 16 | 32 | 64 | 128 => Ok((
                        std::borrow::Cow::Owned(format!("u{}", bits)),
                        std::borrow::Cow::Owned(format!("u{}", bits)),
                        span, Some(bits), false
                    )),
                    _ => Err(span)
                })
        } else {
            input.parse::<syn::Ident>()
                .map_err(|e| e.span())
                .and_then(|type_identifier| match type_identifier.to_string().as_ref() {
                    "size" => Ok(("usize", "usize", None, false)),
                    "NonZero8" => Ok(("core::num::NonZeroU8", "u8", Some(8), true)),
                    "NonZero16" => Ok(("core::num::NonZeroU16", "u16", Some(16), true)),
                    "NonZero32" => Ok(("core::num::NonZeroU32", "u32", Some(32), true)),
                    "NonZero64" => Ok(("core::num::NonZeroU64", "u64", Some(64), true)),
                    "NonZero128" => Ok(("core::num::NonZeroU128", "u128", Some(128), true)),
                    "NonZeroSize" => Ok(("core::num::NonZeroUsize", "usize", None, true)),
                    _ => Err(type_identifier.span())
                }.map(|d| (
                    std::borrow::Cow::Borrowed(d.0),
                    std::borrow::Cow::Borrowed(d.1),
                    type_identifier.span(), d.2, d.3
                )))
        }.map(|(base_type, primitive_type, span, bits, is_non_zero)| {
            let base_type: syn::Path = syn::parse_str(base_type.as_ref()).unwrap();
            (
                syn::parse2::<syn::Path>(quote::quote_spanned!(span => #base_type)).unwrap(),
                syn::Ident::new(primitive_type.as_ref(), span),
                bits, is_non_zero
            )
        }).map_err(|span| syn::Error::new(
            span, "expected one of: `8`, `16`, `32`, `64`, `128`, `size`, `NonZero8`, `NonZero16`, `NonZero32`, `NonZero64`, `NonZero128`, `NonZeroSize`"
        ))?;

        // Read the optional `allow_overlaps` identifier.
        let mut allow_overlaps = None;
        if !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            let ident: syn::Ident = input.parse()?;
            if ident == "allow_overlaps" {
                allow_overlaps = Some(ident);
            } else {
                return Err(syn::Error::new(
                    ident.span(), "expected either `allow_overlaps` or nothing"
                ));
            }
        }

        Ok(Self { base_type, primitive_type, bits, is_non_zero, allow_overlaps })
    }
}

impl super::BitField {
    /// Filters `Debug` and `Display` from `#[derive(...)]` and returns whether they occurred.
    fn filter_derive(attrs: Vec<syn::Attribute>) -> syn::Result<
        (Vec<syn::Attribute>, Option<proc_macro2::Span>, Option<proc_macro2::Span>)
    > {
        let mut debug = None;
        let mut display = None;
        let mut filtered_attrs = Vec::with_capacity(attrs.len());

        for mut attr in attrs {
            if !attr.path.is_ident("derive") {
                filtered_attrs.push(attr);
                continue;
            }

            let meta = match attr.parse_meta()? {
                syn::Meta::List(list) => list.nested,
                syn::Meta::Path(path) => return Err(syn::Error::new(
                    path.span(), "expected list"
                )),
                syn::Meta::NameValue(name) => return Err(syn::Error::new(
                    name.span(), "expected list"
                )),
            }.into_iter().filter(|meta| match meta {
                syn::NestedMeta::Meta(meta) => {
                    if meta.path().is_ident("Debug") {
                        debug = Some(meta.path().span());
                        false
                    } else if meta.path().is_ident("Display") {
                        display = Some(meta.path().span());
                        false
                    } else {
                        true
                    }
                },
                _ => true
            }).collect::<Vec<_>>();

            if !meta.is_empty() {
                // Store the filtered paths.
                attr.tokens = quote::quote! { (#(#meta),*) };

                filtered_attrs.push(attr);
            }
        }

        Ok((filtered_attrs, debug, display))
    }

    fn overlaps(left: &super::FieldDetails, right: &super::FieldDetails) -> syn::Result<bool> {
        let left_bit = left.bit.as_ref().unwrap().base10_parse::<u8>()?;
        let left_size = left.size.as_ref().unwrap().base10_parse::<u8>()?;

        let right_bit = right.bit.as_ref().unwrap().base10_parse::<u8>()?;
        let right_size = right.size.as_ref().unwrap().base10_parse::<u8>()?;

        return Ok(
            left_bit == right_bit ||
            left_bit  < right_bit && left_bit  + left_size  > right_bit ||
            right_bit < left_bit  && right_bit + right_size > left_bit
        );
    }

    /// Tries to parse the `attribute` and `item` into a `BitField` structure.
    pub(crate) fn parse(
        attribute: proc_macro2::TokenStream,
        item: proc_macro2::TokenStream
    ) -> syn::Result<Self> {
        /// Parses the item into a `super::BitField`.
        fn parse_bitfield(attr: super::Attribute, item: proc_macro2::TokenStream)
            -> syn::Result<super::BitField>
        {
            /// Helper structure to parse the basic struct.
            struct BitField {
                attrs: Vec<syn::Attribute>,
                vis: syn::Visibility,
                ident: syn::Ident,
                data: super::Data
            }

            impl syn::parse::Parse for BitField {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    let attrs = input.call(syn::Attribute::parse_outer)?;
                    let vis = input.parse()?;
                    input.parse::<syn::Token![struct]>()?;
                    let ident = input.parse()?;
                    let data = input.parse()?;

                    Ok(Self { attrs, vis, ident, data })
                }
            }

            let bit_field: BitField = syn::parse2(item)?;

            let (attrs, debug, display) =
                super::BitField::filter_derive(bit_field.attrs)?;

            Ok(super::BitField {
                attr, debug, display, attrs,
                vis: bit_field.vis,
                ident: bit_field.ident,
                data: bit_field.data
            })
        }

        /// Fills in optional `bit` information in `super::FieldDetails` and `super::FieldDetails`
        /// for implicit, primitive typed, fields.
        fn complete_fields(bitfield: &mut super::BitField) -> syn::Result<()> {
            let mut bit = 0u8;

            for entry in bitfield.data.entries_mut() {
                let primitive_size = entry.ty.get_ident().map(
                    |ident| crate::primitive::primitive_bits(ident)
                ).flatten();

                if let Some(field) = &mut entry.field {
                    // Handle optional `size`.
                    if field.size.is_none() {
                        if let Some(primitive_size) = primitive_size {
                            // Use the full primitive type size.
                            field.size = Some(syn::LitInt::new(
                                &format!("{}", primitive_size), entry.ty.span()
                            ))
                        } else {
                            return Err(syn::Error::new(entry.ty.span(),
                               "expected an explicit `size` value in the `field` attribute"
                            ));
                        }
                    }

                    if let Some(bit_value) = &field.bit {
                        // Store the next position (after the current field).
                        bit =
                            bit_value.base10_parse::<u8>()? +
                            field.size.as_ref().unwrap().base10_parse::<u8>()?;
                    } else {
                        // Handle fields with optional positions.
                        field.bit = Some(syn::parse_str(&bit.to_string())?);
                        bit += field.size.as_ref().unwrap().base10_parse::<u8>()?;
                    }
                } else if let Some(primitive_size) = primitive_size {
                    // Handle implicit primitive fields.
                    entry.field = Some(super::FieldDetails {
                        span: entry.ty.span(),
                        bit: Some(syn::LitInt::new(&bit.to_string(), entry.ty.span())),
                        size: Some(syn::LitInt::new(
                            &format!("{}", primitive_size), entry.ty.span()
                        )),
                        signed: None
                    });
                    bit += primitive_size;
                }
            }

            Ok(())
        }

        /// Validates the boundaries of all parsed fields.
        fn validate_bitfield(bitfield: &super::BitField) -> syn::Result<()> {
            /// Validates displayable content if `#[derive(Display)]` is defined.
            fn validate_display(bitfield: &super::BitField) -> syn::Result<()> {
                if bitfield.display.is_none() { return Ok(()); }

                if let super::Data::Named(entries) = &bitfield.data {
                    if entries.len() == 0 {
                        // Do not generate `Display` for bit fields with no fields or flags at all.
                        return Err(syn::Error::new(
                            bitfield.display.unwrap(),
                            "can not generate `Display` for empty bit fields"
                        ));
                    } else if entries.len() > 1 {
                        // Do not generate `Display` for bit fields with non-flags.
                        for entry in entries {
                            if entry.entry.field.is_some() {
                                return Err(syn::Error::new(
                                    bitfield.display.unwrap(),
                                    "can not generate `Display` for bit fields with non-flag fields"
                                ));
                            }
                        }
                    }
                }

                Ok(())
            }

            /// Validates the boundaries of one field.
            fn validate_field(bits: Option<u8>, entry: &super::Entry) -> syn::Result<()> {
                if let Some(field) = &entry.field {
                    let size = field.size.as_ref().unwrap().base10_parse::<u8>()?;

                    // Check the boundaries if the base type is not `usize`.
                    if let Some(bits) = bits {
                        let bit = field.bit.as_ref().unwrap().base10_parse::<u8>()?;

                        if let Some(span) =
                            if bit >= bits { Some(field.bit.span()) }
                            else if size > bits { Some(field.size.span()) }
                            else if bit + size > bits { Some(field.span) }
                            else { None }
                        {
                            return Err(syn::Error::new(span, format!(
                                "out of bounds, must not exceed {} bits, as stated in the `#[bitfield(bits)]` attribute",
                                bits
                            )));
                        }

                        if size == bits {
                            return Err(syn::Error::new(
                                field.size.span(), format!(
                                    "field has the size of the whole bit field, use a plain `{}` instead",
                                    quote::ToTokens::to_token_stream(&entry.ty)
                                )
                            ));
                        }
                    }

                    // Special handling for primitive types.
                    if let Some(ty) = entry.ty.get_ident() {
                        if crate::primitive::is_bool(ty) {
                            if size != 1 {
                                return Err(syn::Error::new(ty.span(), format!(
                                    "type is smaller than the specified size of {} bits", size
                                )));
                            }
                            if let Some(signed) = &field.signed {
                                return Err(syn::Error::new(
                                    signed.span(), "unnecessary attribute for `bool`"
                                ));
                            }
                        } else if
                            crate::primitive::is_signed_primitive(ty) ||
                            crate::primitive::is_unsigned_primitive(ty)
                        {
                            let field_size = crate::primitive::primitive_bits(ty).unwrap();

                            if field_size < size {
                                return Err(syn::Error::new(ty.span(), format!(
                                    "type is smaller than the specified size of {} bits", size
                                )));
                            }

                            if field_size != size && crate::primitive::is_signed_primitive(ty) {
                                return Err(syn::Error::new(
                                    ty.span(), format!(
                                        "a signed `{}` with a size of `{}` bits can not store negative numbers, use either `u{}` or `#[field(size = {})]`",
                                        ty, size, field_size, field_size
                                    )
                                ));
                            }

                            if let Some(bits) = bits {
                                if field_size > bits {
                                    return Err(syn::Error::new(ty.span(), format!(
                                        "bigger than the size of the bit field, use `u{}` instead",
                                        bits
                                    )));
                                }
                            }

                            for s in &[8u8, 16, 32, 64, 128] {
                                if field_size > *s && size <= *s {
                                    return Err(syn::Error::new(ty.span(), format!(
                                        "field only uses {} bits, use `u{}` instead", size, *s
                                    )));
                                }
                            }

                            if let Some(signed) = &field.signed {
                                return Err(syn::Error::new(signed.span(), format!(
                                    "unnecessary attribute for `{}`", ty
                                )));
                            }
                        }
                    }
                }

                Ok(())
            }

            /// Checks if any fields overlap.
            fn validate_overlaps(bitfield: &super::BitField) -> syn::Result<()> {
                let entries = match bitfield.data {
                    super::Data::Named(ref entries) => entries,
                    super::Data::Tuple(_) => return Ok(())
                };

                let mut overlap = None;
                let mut has_flags = false;

                for (i, entry) in entries.iter().enumerate() {
                    if let Some(field) = &entry.entry.field {
                        for inner in entries[i..].iter().filter(
                            |e| e.entry.field.is_some()
                        ).skip(1) {
                            if super::BitField::overlaps(
                                field, inner.entry.field.as_ref().unwrap()
                            )? {
                                overlap = Some((entry, inner));
                                break;
                            }
                        }
                    } else {
                        has_flags = true;
                    }

                    if overlap.is_some() && has_flags { break; }
                }

                if let Some(overlap) = overlap {
                    if bitfield.attr.allow_overlaps.is_none() {
                        return Err(syn::Error::new(overlap.1.entry.field.as_ref().unwrap().span,
                            format!(
                                "overlaps with field `{}`, please specify `allow_overlaps` if this is intended",
                                overlap.0.ident
                            )
                        ));
                    }
                } else if !has_flags {
                    if let Some(allow_overlaps) = &bitfield.attr.allow_overlaps {
                        return Err(syn::Error::new(allow_overlaps.span(), format!(
                            "unnecessary since no fields overlap"
                        )));
                    }
                }

                Ok(())
            }

            // Validate all fields separately.
            for entry in bitfield.data.entries() {
                validate_field(bitfield.attr.bits, entry)?;
            }

            validate_overlaps(bitfield)?;
            validate_display(bitfield)?;

            Ok(())
        }

        let attr = syn::parse2(attribute)?;
        let mut bitfield = parse_bitfield(attr, item)?;

        complete_fields(&mut bitfield)?;
        validate_bitfield(&bitfield)?;

        Ok(bitfield)
    }
}

impl syn::parse::Parse for super::Data {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Brace) {
            // Struct with named fields.

            let braces = syn::group::parse_braces(input)?;
            let entries: syn::punctuated::Punctuated<_, syn::Token![,]> =
                braces.content.parse_terminated(super::EntryNamed::parse)?;

            Ok(super::Data::Named(entries.into_pairs().map(
                |p| p.into_value()
            ).collect()))
        } else if lookahead.peek(syn::token::Paren) {
            // Tuple struct.

            let parens = syn::group::parse_parens(input)?;
            let entry = parens.content.parse()?;
            input.parse::<syn::Token![;]>()?;

            Ok(super::Data::Tuple(entry))
        } else {
            Err(input.error("unexpected token"))
        }
    }
}

impl syn::parse::Parse for super::Entry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ty = input.parse()?;
        let field = super::FieldDetails::parse(&mut attrs)?;

        Ok(Self { attrs, vis, ty, field })
    }
}

impl syn::parse::Parse for super::EntryNamed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let ty = input.parse()?;
        let field = super::FieldDetails::parse(&mut attrs)?;

        Ok(Self { ident, entry: super::Entry { attrs, vis, ty, field } })
    }
}

impl syn::parse::Parse for super::FieldDetails {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        fn parse_signed(buffer: &syn::parse::ParseBuffer) -> syn::Result<Option<syn::Ident>> {
            if buffer.peek(syn::Token![,]) && buffer.peek2(syn::Ident) {
                buffer.parse::<syn::Token![,]>()?;

                let ident: syn::Ident = buffer.parse()?;

                return if ident == "signed" {
                    Ok(Some(ident))
                } else {
                    Err(syn::Error::new(ident.span(), "did you mean `signed`?"))
                };
            }
            Ok(None)
        }

        fn validate_bit(bit: &syn::LitInt) -> syn::Result<()> {
            if !bit.base10_parse::<u8>().is_err() {
                Ok(())
            } else {
                Err(syn::Error::new(bit.span(), "expected a number between 0-255"))
            }
        }

        fn validate_size(size: &syn::LitInt) -> syn::Result<()> {
            if !size.base10_parse::<core::num::NonZeroU8>().is_err() {
                Ok(())
            } else {
                Err(syn::Error::new(size.span(), "expected a number between 1-255"))
            }
        }

        let buffer = syn::group::parse_parens(input)?.content;

        // Parse `bit = LitInt, signed?` or `size = LitInt, signed?`.
        if let Ok(ident) = buffer.parse::<syn::Ident>() {
            buffer.parse::<syn::Token![=]>()?;
            let value: syn::LitInt = buffer.parse()?;
            let signed = parse_signed(&buffer)?;

            if !buffer.is_empty() {
                return Err(buffer.error("unexpected token"));
            }

            return if ident == "bit" {
                validate_bit(&value)?;
                Ok(Self { span: value.span(), bit: Some(value), size: None, signed })
            } else if ident == "size" {
                validate_size(&value)?;
                Ok(Self { span: value.span(), bit: None, size: Some(value), signed })
            } else {
                Err(syn::Error::new(ident.span(), "expected `bit` or `size`"))
            };
        }

        // Parse `(bit: LitInt, size: LitInt)`.
        let bit: syn::LitInt = buffer.parse()?;
        validate_bit(&bit)?;

        buffer.parse::<syn::Token![,]>()?;

        let size: syn::LitInt = buffer.parse()?;
        validate_size(&size)?;

        let signed = parse_signed(&buffer)?;

        if !buffer.is_empty() {
            return Err(buffer.error("unexpected token"));
        }

        let span = bit.span().join(size.span()).unwrap();

        Ok(Self { span, bit: Some(bit), size: Some(size), signed })
    }
}

impl super::FieldDetails {
    // Parse and remove the optional `field` attribute from `attrs`.
    fn parse(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Option<Self>> {
        if let Some(index) = attrs.iter().enumerate().find(
            |(_, attr)| attr.path.is_ident("field")
        ).map(|result| result.0) {
            Ok(Some(syn::parse2(attrs.remove(index).tokens)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
#[macro_use]
pub(super) mod tests {
    use quote::ToTokens;
    use super::super::*;

    macro_rules! filter_derive {
        ($attributes:expr, $len:expr, $debug:expr, $display:expr) => {{
            let attrs = syn::parse_str::<syn::DeriveInput>(
                concat!($attributes, "enum A { B }")
            ).unwrap().attrs;
            let result = BitField::filter_derive(attrs).unwrap();

            assert_eq!(result.0.len(), $len);
            assert_eq!(
                result.1.map(|s| (s.start().line, s.start().column, s.end().line, s.end().column)),
                $debug
            );
            assert_eq!(
                result.2.map(|s| (s.start().line, s.start().column, s.end().line, s.end().column)),
                $display
            );

            result
        }};

        ($attributes:expr, $len:expr, $debug:expr, $display:expr, $result:expr) => {
            let result = filter_derive!($attributes, $len, $debug, $display);
            assert_eq!(
                quote::ToTokens::to_token_stream(&result.0.first().unwrap()).to_string(),
                $result.to_string()
            );
        };
    }

    macro_rules! parse_invalid {
        ($attribute:expr, $item:expr, $message:expr, ($sl:expr, $sc:expr), ($el:expr, $ec:expr)) => {{
            let error = BitField::parse(
                $attribute.parse().unwrap(), $item.parse().unwrap()
            ).map(|_| ()).unwrap_err();
            assert_eq!(error.to_string(), $message);
            compare_span!(error.span(), ($sl, $sc), ($el, $ec));
        }}
    }

    macro_rules! parse_valid {
        ($attribute:expr, $item:expr) => {
            BitField::parse($attribute.parse().unwrap(), $item.parse().unwrap()).unwrap()
        }
    }

    // Test macros.

    #[test]
    #[should_panic]
    fn test_compare_span() {
        if let Data::Tuple(entry) = parse_valid!(
            "8", "struct A(#[field(0, 1)] A);"
        ).data {
            compare_span!(entry.field.as_ref().unwrap().span, (1, 17), (1, 16));
        }
    }

    #[test]
    #[should_panic]
    fn test_filter_derive() {
        filter_derive!("", 0, None, Some((0, 0, 0, 0)));
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid() {
        parse_invalid!(
            "Ident", "",
            "expected literal",
            (1, 0), (1, 6)
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_valid() {
        parse_valid!("8", "struct A(A)");
    }

    // Test parsing.

    #[test]
    fn attribute_allow_overlaps() {
        assert!(parse_valid!("8", "struct A(A);").attr.allow_overlaps.is_none());

        parse_invalid!(
            "8,", "",
            "unexpected end of input, expected identifier",
            (1, 0), (1, 0)
        );

        parse_invalid!(
            "8, Ident", "",
            "expected either `allow_overlaps` or nothing",
            (1, 3), (1, 8)
        );

        parse_invalid!(
            "8, allow_overlaps", "struct A {}",
            "unnecessary since no fields overlap",
            (1, 3), (1, 17)
        );

        parse_invalid!(
            "8, allow_overlaps", "struct A { #[field(0, 2)] b: B, #[field(2, 2)] c: C }",
            "unnecessary since no fields overlap",
            (1, 3), (1, 17)
        );

        compare_span!(parse_valid!(
            "8, allow_overlaps", "struct A { #[field(0, 2)] b: B, #[field(1, 2)] c: C }"
        ).attr.allow_overlaps.unwrap().span(), (1, 3), (1, 17));

        compare_span!(parse_valid!(
            "8, allow_overlaps", "struct A { #[field(0, 2)] b: B, c: C }"
        ).attr.allow_overlaps.unwrap().span(), (1, 3), (1, 17));
    }

    #[test]
    fn attribute_bits_size() {
        parse_invalid!(
            "Ident", "",
            "expected one of: `8`, `16`, `32`, `64`, `128`, `size`, `NonZero8`, `NonZero16`, `NonZero32`, `NonZero64`, `NonZero128`, `NonZeroSize`",
            (1, 0), (1, 5)
        );

        parse_invalid!(
            "-1", "",
            "expected one of: `8`, `16`, `32`, `64`, `128`, `size`, `NonZero8`, `NonZero16`, `NonZero32`, `NonZero64`, `NonZero128`, `NonZeroSize`",
            (1, 0), (1, 2)
        );

        parse_invalid!(
            "0", "",
            "expected one of: `8`, `16`, `32`, `64`, `128`, `size`, `NonZero8`, `NonZero16`, `NonZero32`, `NonZero64`, `NonZero128`, `NonZeroSize`",
            (1, 0), (1, 1)
        );

        parse_invalid!(
            "1", "",
            "expected one of: `8`, `16`, `32`, `64`, `128`, `size`, `NonZero8`, `NonZero16`, `NonZero32`, `NonZero64`, `NonZero128`, `NonZeroSize`",
            (1, 0), (1, 1)
        );

        let attr = parse_valid!("8", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(8));
        assert_eq!(attr.base_type.to_token_stream().to_string(), "u8");
        assert_eq!(attr.primitive_type, "u8");

        let attr = parse_valid!("NonZero8", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(8));
        assert_eq!(
            attr.base_type.to_token_stream().to_string(),
            quote::quote!(core::num::NonZeroU8).to_string()
        );
        assert_eq!(attr.primitive_type, "u8");

        let attr = parse_valid!("16", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(16));
        assert_eq!(attr.base_type.to_token_stream().to_string(), "u16");
        assert_eq!(attr.primitive_type, "u16");

        let attr = parse_valid!("NonZero16", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(16));
        assert_eq!(
            attr.base_type.to_token_stream().to_string(),
            quote::quote!(core::num::NonZeroU16).to_string()
        );
        assert_eq!(attr.primitive_type, "u16");

        let attr = parse_valid!("32", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(32));
        assert_eq!(attr.base_type.to_token_stream().to_string(), "u32");
        assert_eq!(attr.primitive_type, "u32");

        let attr = parse_valid!("NonZero32", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(32));
        assert_eq!(
            attr.base_type.to_token_stream().to_string(),
            quote::quote!(core::num::NonZeroU32).to_string()
        );
        assert_eq!(attr.primitive_type, "u32");

        let attr = parse_valid!("64", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(64));
        assert_eq!(attr.base_type.to_token_stream().to_string(), "u64");
        assert_eq!(attr.primitive_type, "u64");

        let attr = parse_valid!("NonZero64", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(64));
        assert_eq!(
            attr.base_type.to_token_stream().to_string(),
            quote::quote!(core::num::NonZeroU64).to_string()
        );
        assert_eq!(attr.primitive_type, "u64");

        let attr = parse_valid!("128", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(128));
        assert_eq!(attr.base_type.to_token_stream().to_string(), "u128");
        assert_eq!(attr.primitive_type, "u128");

        let attr = parse_valid!("NonZero128", "struct A(A);").attr;
        assert_eq!(attr.bits, Some(128));
        assert_eq!(
            attr.base_type.to_token_stream().to_string(),
            quote::quote!(core::num::NonZeroU128).to_string()
        );
        assert_eq!(attr.primitive_type, "u128");

        let attr = parse_valid!("size", "struct A(A);").attr;
        assert_eq!(attr.bits, None);
        assert_eq!(attr.base_type.to_token_stream().to_string(), "usize");
        assert_eq!(attr.primitive_type, "usize");

        let attr = parse_valid!("NonZeroSize", "struct A(A);").attr;
        assert_eq!(attr.bits, None);
        assert_eq!(
            attr.base_type.to_token_stream().to_string(),
            quote::quote!(core::num::NonZeroUsize).to_string()
        );
        assert_eq!(attr.primitive_type, "usize");
    }

    #[test]
    fn bitfield_attrs() {
        let attrs = parse_valid!(
            "8", "#[some_attribute1] #[some_attribute2] struct A(A);"
        ).attrs;

        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs.first().unwrap().path.get_ident().unwrap().to_string(), "some_attribute1");
    }

    #[test]
    fn bitfield_debug() {
        assert!(parse_valid!("8", "struct A(A);").debug.is_none());
        assert!(parse_valid!("8", "#[derive(Display)] struct A(A);").debug.is_none());
        assert!(parse_valid!("8", "#[derive(Debug)] struct A(A);").debug.is_some());
    }

    #[test]
    fn bitfield_display() {
        assert!(parse_valid!("8", "struct A(A);").display.is_none());
        assert!(parse_valid!("8", "#[derive(Debug)] struct A(A);").display.is_none());
        assert!(parse_valid!("8", "#[derive(Display)] struct A(A);").display.is_some());
        parse_invalid!(
            "8", "#[derive(Display)] struct A {}",
            "can not generate `Display` for empty bit fields",
            (1, 9), (1, 16)
        );
        parse_invalid!(
            "8", "#[derive(Display)] struct A { b: B, #[field(0, 2)] c: C }",
            "can not generate `Display` for bit fields with non-flag fields",
            (1, 9), (1, 16)
        );
    }

    #[test]
    fn bitfield_vis() {
        assert!(match &parse_valid!("8", "struct A(A);").vis {
            syn::Visibility::Inherited => true,
            _ => false
        });

        assert!(match &parse_valid!("8", "pub struct A(A);").vis {
            syn::Visibility::Public(_) => true,
            _ => false
        });
    }

    #[test]
    fn bitfield_ident() {
        assert_eq!(parse_valid!("8", "struct A(A);").ident.to_string(), "A");
        assert_eq!(parse_valid!("8", "struct B(A);").ident.to_string(), "B");
    }

    #[test]
    fn complete_fields() {
        macro_rules! assert_field {
            ($data:expr, $skip:expr, $bit:expr, $size:expr) => {
                assert_eq!(
                    $data.entries().iter().skip($skip).next().unwrap().field.as_ref().map(|field| (
                        field.bit.as_ref().unwrap().base10_parse::<u8>().unwrap(),
                        field.size.as_ref().unwrap().base10_parse::<u8>().unwrap(),
                    )).unwrap(), ($bit, $size)
                );
            };
        }

        assert_field!(parse_valid!("8", "struct A(#[field(bit = 2)] bool);").data, 0, 2, 1);
        assert_field!(parse_valid!("NonZero8", "struct A(#[field(bit = 2)] bool);").data, 0, 2, 1);
        assert_field!(parse_valid!("16", "struct A(#[field(bit = 2)] i8);").data, 0, 2, 8);
        assert_field!(parse_valid!("NonZero16", "struct A(#[field(bit = 2)] i8);").data, 0, 2, 8);
        assert_field!(parse_valid!("16", "struct A(#[field(bit = 2)] u8);").data, 0, 2, 8);
        assert_field!(parse_valid!("NonZero16", "struct A(#[field(bit = 2)] u8);").data, 0, 2, 8);
        assert_field!(parse_valid!("32", "struct A(#[field(bit = 2)] i16);").data, 0, 2, 16);
        assert_field!(parse_valid!("NonZero32", "struct A(#[field(bit = 2)] i16);").data, 0, 2, 16);
        assert_field!(parse_valid!("32", "struct A(#[field(bit = 2)] u16);").data, 0, 2, 16);
        assert_field!(parse_valid!("NonZero32", "struct A(#[field(bit = 2)] u16);").data, 0, 2, 16);
        assert_field!(parse_valid!("64", "struct A(#[field(bit = 2)] i32);").data, 0, 2, 32);
        assert_field!(parse_valid!("NonZero64", "struct A(#[field(bit = 2)] i32);").data, 0, 2, 32);
        assert_field!(parse_valid!("64", "struct A(#[field(bit = 2)] u32);").data, 0, 2, 32);
        assert_field!(parse_valid!("NonZero64", "struct A(#[field(bit = 2)] u32);").data, 0, 2, 32);
        assert_field!(parse_valid!("128", "struct A(#[field(bit = 2)] i64);").data, 0, 2, 64);
        assert_field!(parse_valid!("NonZero128", "struct A(#[field(bit = 2)] i64);").data, 0, 2, 64);
        assert_field!(parse_valid!("128", "struct A(#[field(bit = 2)] u64);").data, 0, 2, 64);
        assert_field!(parse_valid!("NonZero128", "struct A(#[field(bit = 2)] u64);").data, 0, 2, 64);
        parse_invalid!(
            "128", "struct A(#[field(bit = 0)] u128);",
            "field has the size of the whole bit field, use a plain `u128` instead",
            (1, 27), (1, 31)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(bit = 0)] u128);",
            "field has the size of the whole bit field, use a plain `u128` instead",
            (1, 27), (1, 31)
        );
        parse_invalid!(
            "128", "struct A(#[field(bit = 0)] i128);",
            "field has the size of the whole bit field, use a plain `i128` instead",
            (1, 27), (1, 31)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(bit = 0)] i128);",
            "field has the size of the whole bit field, use a plain `i128` instead",
            (1, 27), (1, 31)
        );
        parse_invalid!(
            "8", "struct A(#[field(bit = 2)] B);",
            "expected an explicit `size` value in the `field` attribute",
            (1, 27), (1, 28)
        );
        parse_invalid!(
            "NonZero8", "struct A(#[field(bit = 2)] B);",
            "expected an explicit `size` value in the `field` attribute",
            (1, 27), (1, 28)
        );

        let data = parse_valid!(
            "8", "struct A { #[field(1, 2)] b: B, #[field(size = 1)] c: bool, #[field(size = 1)] d: bool }"
        ).data;
        assert_field!(data, 0, 1, 2);
        assert_field!(data, 1, 3, 1);
        assert_field!(data, 2, 4, 1);

        let data = parse_valid!(
            "NonZero8", "struct A { #[field(1, 2)] b: B, #[field(size = 1)] c: bool, #[field(size = 1)] d: bool }"
        ).data;
        assert_field!(data, 0, 1, 2);
        assert_field!(data, 1, 3, 1);
        assert_field!(data, 2, 4, 1);

        let data = parse_valid!(
            "16", "struct A { b: bool, c: u8, d: bool }"
        ).data;
        assert_field!(data, 0, 0, 1);
        assert_field!(data, 1, 1, 8);
        assert_field!(data, 2, 9, 1);

        let data = parse_valid!(
            "NonZero16", "struct A { b: bool, c: u8, d: bool }"
        ).data;
        assert_field!(data, 0, 0, 1);
        assert_field!(data, 1, 1, 8);
        assert_field!(data, 2, 9, 1);
    }

    #[test]
    fn data_named() {
        assert_eq!(parse_valid!("8", "struct A {}").data.entries().len(), 0);
        assert_eq!(parse_valid!("8", "struct A { a: A }").data.entries().len(), 1);
        assert_eq!(parse_valid!("8", "struct A { #[field(0, 1)] a: A }").data.entries().len(), 1);
        assert_eq!(parse_valid!("8", "struct A { a: A, #[field(0, 1)] b: B }").data.entries().len(), 2);
        assert_eq!(parse_valid!("8", "struct A { #[field(0, 1)] a: A, b: B }").data.entries().len(), 2);
    }

    #[test]
    fn data_tuple() {
        parse_invalid!(
            "8", "struct A();",
            "unexpected end of input, expected identifier",
            (1, 9), (1, 10)
        );

        parse_invalid!(
            "8", "struct A(A)",
            "expected `;`",
            (1, 0), (1, 0)
        );

        assert!(match parse_valid!("8", "struct A(A);").data {
            Data::Tuple(_) => true,
            _ => false
        });
    }

    #[test]
    fn entry_attrs() {
        let attrs = match parse_valid!(
            "8", "struct A(#[some_attribute1] #[some_attribute2] A);"
        ).data {
            Data::Tuple(entry) => Some(entry.attrs),
            _ => None
        }.unwrap();

        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs.first().unwrap().path.get_ident().unwrap().to_string(), "some_attribute1");
    }

    #[test]
    fn entry_vis() {
        assert!(match parse_valid!("8", "struct A(A);").data {
            Data::Tuple(entry) => match entry.vis {
                syn::Visibility::Inherited => true,
                _ => false
            },
            _ => false
        });

        assert!(match parse_valid!("8", "struct A(pub A);").data {
            Data::Tuple(entry) => match entry.vis {
                syn::Visibility::Public(_) => true,
                _ => false
            },
            _ => false
        });
    }

    #[test]
    fn entry_ty() {
        match parse_valid!("8", "struct A(A);").data {
            Data::Tuple(entry) => {
                let mut ts = proc_macro2::TokenStream::new();
                quote::ToTokens::to_tokens(&entry.ty, &mut ts);
                assert_eq!(ts.to_string(), "A");
            },
            _ => panic!("expected valid parsing")
        }

        match parse_valid!("8", "struct A(crate::module::Type);").data {
            Data::Tuple(entry) => {
                let mut ts = proc_macro2::TokenStream::new();
                quote::ToTokens::to_tokens(&entry.ty, &mut ts);
                assert_eq!(ts.to_string(), quote::quote! { crate::module::Type }.to_string());
            },
            _ => panic!("expected valid parsing")
        }
    }

    #[test]
    fn entry_field() {
        assert!(match parse_valid!("8", "struct A(A);").data {
            Data::Tuple(entry) => entry.field.is_none(),
            _ => false
        });

        parse_invalid!(
            "8", "struct A(#[field(0, 1, 3)] A);",
            "unexpected token",
            (1, 21), (1, 22)
        );

        assert!(match parse_valid!("8", "struct A(#[field(0, 1)] A);").data {
            Data::Tuple(entry) => entry.field.is_some(),
            _ => false
        });
    }

    #[test]
    fn entry_named_ident() {
        let entries = match parse_valid!(
            "8", "struct A { a: A }"
        ).data {
            Data::Named(entries) => Some(entries),
            _ => None
        }.unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries.first().unwrap().ident, "a");

        let entries = match parse_valid!(
            "8", "struct A { b: B }"
        ).data {
            Data::Named(entries) => Some(entries),
            _ => None
        }.unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries.first().unwrap().ident, "b");
    }

    #[test]
    fn field_details_span() {
        match parse_valid!("8", "struct A(#[field(0, 1)] A);").data {
            Data::Tuple(entry) => {
                compare_span!(entry.field.as_ref().unwrap().span, (1, 17), (1, 21));
            },
            _ => panic!("expected `Data::Tuple`")
        };

        match parse_valid!("8", "struct A(#[field( 0 , 1 )] A);").data {
            Data::Tuple(entry) => {
                compare_span!(entry.field.as_ref().unwrap().span, (1, 18), (1, 23));
            },
            _ => panic!("expected `Data::Tuple`")
        };
    }

    #[test]
    fn field_details_bit() {
        parse_invalid!(
            "8", "struct A(#[field(Ident, 1)] A);",
            "expected `=`",
            (1, 22), (1, 23)
        );

        parse_invalid!(
            "8", "struct A(#[field(-1, 1)] A);",
            "expected a number between 0-255",
            (1, 17), (1, 19)
        );

        parse_invalid!(
            "8", "struct A(#[field(0x100, 1)] A);",
            "expected a number between 0-255",
            (1, 17), (1, 22)
        );

        parse_invalid!(
            "8", "struct A(#[field(bit = 0x100)] bool);",
            "expected a number between 0-255",
            (1, 23), (1, 28)
        );

        assert!(match parse_valid!("8", "struct A(#[field(bit = 1)] bool);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().bit.unwrap().base10_parse::<u8>().unwrap() == 1
            },
            _ => false
        });

        assert!(match parse_valid!("8", "struct A(#[field(0, 1)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().bit.unwrap().base10_parse::<u8>().unwrap() == 0
            },
            _ => false
        });

        assert!(match parse_valid!("8", "struct A(#[field(1, 1)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().bit.unwrap().base10_parse::<u8>().unwrap() == 1
            },
            _ => false
        });
    }

    #[test]
    fn field_details_size() {
        parse_invalid!(
            "8", "struct A(#[field(0, Ident)] A);",
            "expected integer literal",
            (1, 20), (1, 25)
        );

        parse_invalid!(
            "8", "struct A(#[field(0, -1)] A);",
            "expected a number between 1-255",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "8", "struct A(#[field(0, 0)] A);",
            "expected a number between 1-255",
            (1, 20), (1, 21)
        );

        parse_invalid!(
            "8", "struct A(#[field(0, 0x100)] A);",
            "expected a number between 1-255",
            (1, 20), (1, 25)
        );

        parse_invalid!(
            "8", "struct A(#[field(size = 0x100)] A);",
            "expected a number between 1-255",
            (1, 24), (1, 29)
        );

        assert!(match parse_valid!("8", "struct A(#[field(size = 2)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().size.as_ref().unwrap().base10_parse::<u8>().unwrap() == 2
            },
            _ => false
        });

        assert!(match parse_valid!("8", "struct A(#[field(0, 1)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().size.as_ref().unwrap().base10_parse::<u8>().unwrap() == 1
            },
            _ => false
        });

        assert!(match parse_valid!("8", "struct A(#[field(1, 2)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().size.as_ref().unwrap().base10_parse::<u8>().unwrap() == 2
            },
            _ => false
        });
    }

    #[test]
    fn field_details_signed() {
        assert!(match parse_valid!("8", "struct A(#[field(size = 2)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().signed.is_none()
            },
            _ => false
        });

        assert!(match parse_valid!("8", "struct A(#[field(size = 2, signed)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().signed.is_some()
            },
            _ => false
        });

        assert!(match parse_valid!("8", "struct A(#[field(1, 2)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().signed.is_none()
            },
            _ => false
        });

        assert!(match parse_valid!("8", "struct A(#[field(1, 2, signed)] A);").data {
            Data::Tuple(entry) => {
                entry.field.unwrap().signed.is_some()
            },
            _ => false
        });
    }

    #[test]
    fn field_details_extra_tokens() {
        parse_invalid!(
            "8", "struct A(#[field(1, 2, 3)] B);",
            "unexpected token",
            (1, 21), (1, 22)
        );

        parse_invalid!(
            "8", "struct A(#[field(bit = 1, 2)] B);",
            "unexpected token",
            (1, 24), (1, 25)
        );

        parse_invalid!(
            "8", "struct A(#[field(bit = 1, signed, X)] B);",
            "unexpected token",
            (1, 32), (1, 33)
        );

        parse_invalid!(
            "8", "struct A(#[field(size = 1, 2)] B);",
            "unexpected token",
            (1, 25), (1, 26)
        );

        parse_invalid!(
            "8", "struct A(#[field(bit = 1, signed, X)] B);",
            "unexpected token",
            (1, 32), (1, 33)
        );
    }

    #[test]
    fn field_details_short() {
        parse_invalid!(
            "8", "struct A(#[field(x = 1)] B);",
            "expected `bit` or `size`",
            (1, 17), (1, 18)
        );
    }

    #[test]
    fn filter_derive() {
        filter_derive!("", 0, None, None);
        filter_derive!(
            "#[derive(Other)]", 1,
            None, None,
            quote::quote! { #[derive(Other)] }
        );
        filter_derive!(
            "#[derive(Debug)]", 0,
            Some((1, 9, 1, 14)), None
        );
        filter_derive!(
            "#[derive(Display)]", 0,
            None, Some((1, 9, 1, 16))
        );
        filter_derive!(
            "#[derive(Debug, Display)]", 0,
            Some((1, 9, 1, 14)), Some((1, 16, 1, 23))
        );
        filter_derive!(
            "#[derive(Debug, Other, Display)]", 1,
            Some((1, 9, 1, 14)), Some((1, 23, 1, 30)),
            quote::quote! { #[derive(Other)] }
        );
        filter_derive!(
            "#[derive(Other)] /** */", 2,
            None, None,
            quote::quote! { #[derive(Other)] }
        );
        filter_derive!(
            "#[derive(Debug)] /** */", 1,
            Some((1, 9, 1, 14)), None,
            quote::quote! { #[doc = " "] }
        );
        filter_derive!(
            "#[derive(Display)] /** */", 1,
            None, Some((1, 9, 1, 16)),
            quote::quote! { #[doc = " "] }
        );
        filter_derive!(
            "#[derive(Debug, Display)] /** */", 1,
            Some((1, 9, 1, 14)), Some((1, 16, 1, 23)),
            quote::quote! { #[doc = " "] }
        );
        filter_derive!(
            "#[derive(Debug, Other, Display)] /** */", 2,
            Some((1, 9, 1, 14)), Some((1, 23, 1, 30)),
            quote::quote! { #[derive(Other)] }
        );
    }

    #[test]
    fn overlaps() {
        let zero_one = syn::parse_str("(0, 1)").unwrap();
        let zero_two = syn::parse_str("(0, 2)").unwrap();
        let zero_three = syn::parse_str("(0, 3)").unwrap();
        let one_one = syn::parse_str("(1, 1)").unwrap();
        let one_two = syn::parse_str("(1, 2)").unwrap();
        let two_one = syn::parse_str("(2, 1)").unwrap();

        // l-l r-r
        assert!(!BitField::overlaps(&zero_one, &two_one).unwrap());
        // l-lr-r
        assert!(!BitField::overlaps(&zero_one, &one_one).unwrap());
        // l-rl-r
        assert!(BitField::overlaps(&zero_two, &one_two).unwrap());
        // l-r-lr
        assert!(BitField::overlaps(&zero_two, &one_one).unwrap());
        // r-l-rl
        assert!(BitField::overlaps(&one_one, &zero_two).unwrap());
        // l-r-r-l
        assert!(BitField::overlaps(&zero_three, &one_one).unwrap());
        // lr-lr
        assert!(BitField::overlaps(&zero_one, &zero_one).unwrap());
        // r-l-l-r
        assert!(BitField::overlaps(&one_one, &zero_three).unwrap());
        // lr-r-l
        assert!(BitField::overlaps(&zero_two, &zero_one).unwrap());
        // rl-l-r
        assert!(BitField::overlaps(&zero_one, &zero_two).unwrap());
        // r-lr-l
        assert!(BitField::overlaps(&one_two, &zero_two).unwrap());
        // r-rl-l
        assert!(!BitField::overlaps(&zero_one, &one_one).unwrap());
        // r-r l-l
        assert!(!BitField::overlaps(&zero_one, &two_one).unwrap());
    }

    #[test]
    fn validate_bounds() {
        parse_invalid!(
            "8", "struct A(#[field(8, 1)] A);",
            "out of bounds, must not exceed 8 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 18)
        );

        parse_invalid!(
            "8", "struct A(#[field(0, 9)] A);",
            "out of bounds, must not exceed 8 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 21)
        );

        parse_invalid!(
            "8", "struct A(#[field(1, 8)] A);",
            "out of bounds, must not exceed 8 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 21)
        );

        parse_valid!("8", "struct A(#[field(1, 7)] A);");

        parse_invalid!(
            "NonZero8", "struct A(#[field(8, 1)] A);",
            "out of bounds, must not exceed 8 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 18)
        );

        parse_invalid!(
            "NonZero8", "struct A(#[field(0, 9)] A);",
            "out of bounds, must not exceed 8 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 21)
        );

        parse_invalid!(
            "NonZero8", "struct A(#[field(1, 8)] A);",
            "out of bounds, must not exceed 8 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 21)
        );

        parse_valid!("NonZero8", "struct A(#[field(1, 7)] A);");

        parse_invalid!(
            "16", "struct A(#[field(16, 1)] A);",
            "out of bounds, must not exceed 16 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 19)
        );

        parse_invalid!(
            "16", "struct A(#[field(0, 17)] A);",
            "out of bounds, must not exceed 16 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "16", "struct A(#[field(1, 16)] A);",
            "out of bounds, must not exceed 16 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 22)
        );

        parse_valid!("16", "struct A(#[field(1, 15)] A);");

        parse_invalid!(
            "NonZero16", "struct A(#[field(16, 1)] A);",
            "out of bounds, must not exceed 16 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 19)
        );

        parse_invalid!(
            "NonZero16", "struct A(#[field(0, 17)] A);",
            "out of bounds, must not exceed 16 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "NonZero16", "struct A(#[field(1, 16)] A);",
            "out of bounds, must not exceed 16 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 22)
        );

        parse_valid!("NonZero16", "struct A(#[field(1, 15)] A);");

        parse_invalid!(
            "32", "struct A(#[field(32, 1)] A);",
            "out of bounds, must not exceed 32 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 19)
        );

        parse_invalid!(
            "32", "struct A(#[field(0, 33)] A);",
            "out of bounds, must not exceed 32 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "32", "struct A(#[field(1, 32)] A);",
            "out of bounds, must not exceed 32 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 22)
        );

        parse_valid!("32", "struct A(#[field(1, 31)] A);");

        parse_invalid!(
            "NonZero32", "struct A(#[field(32, 1)] A);",
            "out of bounds, must not exceed 32 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 19)
        );

        parse_invalid!(
            "NonZero32", "struct A(#[field(0, 33)] A);",
            "out of bounds, must not exceed 32 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "NonZero32", "struct A(#[field(1, 32)] A);",
            "out of bounds, must not exceed 32 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 22)
        );

        parse_valid!("NonZero32", "struct A(#[field(1, 31)] A);");

        parse_invalid!(
            "64", "struct A(#[field(64, 1)] A);",
            "out of bounds, must not exceed 64 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 19)
        );

        parse_invalid!(
            "64", "struct A(#[field(0, 65)] A);",
            "out of bounds, must not exceed 64 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "64", "struct A(#[field(1, 64)] A);",
            "out of bounds, must not exceed 64 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 22)
        );

        parse_valid!("64", "struct A(#[field(1, 63)] A);");

        parse_invalid!(
            "NonZero64", "struct A(#[field(64, 1)] A);",
            "out of bounds, must not exceed 64 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 19)
        );

        parse_invalid!(
            "NonZero64", "struct A(#[field(0, 65)] A);",
            "out of bounds, must not exceed 64 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 64)] A);",
            "out of bounds, must not exceed 64 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 22)
        );

        parse_valid!("NonZero64", "struct A(#[field(1, 63)] A);");

        parse_invalid!(
            "128", "struct A(#[field(128, 1)] A);",
            "out of bounds, must not exceed 128 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 20)
        );

        parse_invalid!(
            "128", "struct A(#[field(0, 129)] A);",
            "out of bounds, must not exceed 128 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 23)
        );

        parse_invalid!(
            "128", "struct A(#[field(1, 128)] A);",
            "out of bounds, must not exceed 128 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 23)
        );

        parse_valid!("128", "struct A(#[field(1, 127)] A);");

        parse_invalid!(
            "NonZero128", "struct A(#[field(128, 1)] A);",
            "out of bounds, must not exceed 128 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 20)
        );

        parse_invalid!(
            "NonZero128", "struct A(#[field(0, 129)] A);",
            "out of bounds, must not exceed 128 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 20), (1, 23)
        );

        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 128)] A);",
            "out of bounds, must not exceed 128 bits, as stated in the `#[bitfield(bits)]` attribute",
            (1, 17), (1, 23)
        );

        parse_valid!("NonZero128", "struct A(#[field(1, 127)] A);");
    }

    #[test]
    fn validate_capacity() {
        parse_valid!("8", "struct A(#[field(1, 7)] u8);");
        parse_valid!("NonZero8", "struct A(#[field(1, 7)] u8);");

        parse_valid!("16", "struct A(#[field(1, 8)] u8);");
        parse_valid!("NonZero16", "struct A(#[field(1, 8)] u8);");
        parse_invalid!(
            "16", "struct A(#[field(1, 8)] u16);",
            "field only uses 8 bits, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "NonZero16", "struct A(#[field(1, 8)] u16);",
            "field only uses 8 bits, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_valid!("16", "struct A(#[field(1, 15)] u16);");
        parse_valid!("NonZero16", "struct A(#[field(1, 15)] u16);");

        parse_valid!("32", "struct A(#[field(1, 8)] u8);");
        parse_valid!("NonZero32", "struct A(#[field(1, 8)] u8);");
        parse_valid!("32", "struct A(#[field(1, 16)] u16);");
        parse_valid!("NonZero32", "struct A(#[field(1, 16)] u16);");
        parse_invalid!(
            "32", "struct A(#[field(1, 8)] u16);",
            "field only uses 8 bits, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(1, 8)] u16);",
            "field only uses 8 bits, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "32", "struct A(#[field(1, 16)] u32);",
            "field only uses 16 bits, use `u16` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(1, 16)] u32);",
            "field only uses 16 bits, use `u16` instead",
            (1, 25), (1, 28)
        );
        parse_valid!("32", "struct A(#[field(1, 31)] u32);");
        parse_valid!("NonZero32", "struct A(#[field(1, 31)] u32);");

        parse_valid!("64", "struct A(#[field(1, 8)] u8);");
        parse_valid!("NonZero64", "struct A(#[field(1, 8)] u8);");
        parse_valid!("64", "struct A(#[field(1, 16)] u16);");
        parse_valid!("NonZero64", "struct A(#[field(1, 16)] u16);");
        parse_valid!("64", "struct A(#[field(1, 32)] u32);");
        parse_valid!("NonZero64", "struct A(#[field(1, 32)] u32);");
        parse_invalid!(
            "64", "struct A(#[field(1, 8)] u16);",
            "field only uses 8 bits, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 8)] u16);",
            "field only uses 8 bits, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "64", "struct A(#[field(1, 16)] u32);",
            "field only uses 16 bits, use `u16` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 16)] u32);",
            "field only uses 16 bits, use `u16` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "64", "struct A(#[field(1, 32)] u64);",
            "field only uses 32 bits, use `u32` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 32)] u64);",
            "field only uses 32 bits, use `u32` instead",
            (1, 25), (1, 28)
        );
        parse_valid!("64", "struct A(#[field(1, 63)] u64);");
        parse_valid!("NonZero64", "struct A(#[field(1, 63)] u64);");

        parse_valid!("128", "struct A(#[field(1, 8)] u8);");
        parse_valid!("NonZero128", "struct A(#[field(1, 8)] u8);");
        parse_valid!("128", "struct A(#[field(1, 16)] u16);");
        parse_valid!("NonZero128", "struct A(#[field(1, 16)] u16);");
        parse_valid!("128", "struct A(#[field(1, 32)] u32);");
        parse_valid!("NonZero128", "struct A(#[field(1, 32)] u32);");
        parse_valid!("128", "struct A(#[field(1, 64)] u64);");
        parse_valid!("NonZero128", "struct A(#[field(1, 64)] u64);");
        parse_invalid!(
            "128", "struct A(#[field(1, 8)] u16);",
            "field only uses 8 bits, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 8)] u16);",
            "field only uses 8 bits, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "128", "struct A(#[field(1, 16)] u32);",
            "field only uses 16 bits, use `u16` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 16)] u32);",
            "field only uses 16 bits, use `u16` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "128", "struct A(#[field(1, 32)] u64);",
            "field only uses 32 bits, use `u32` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 32)] u64);",
            "field only uses 32 bits, use `u32` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "128", "struct A(#[field(1, 64)] u128);",
            "field only uses 64 bits, use `u64` instead",
            (1, 25), (1, 29)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 64)] u128);",
            "field only uses 64 bits, use `u64` instead",
            (1, 25), (1, 29)
        );
        parse_valid!("128", "struct A(#[field(1, 127)] u128);");
        parse_valid!("NonZero128", "struct A(#[field(1, 127)] u128);");
    }

    #[test]
    fn validate_field_size() {
        parse_invalid!(
            "8", "struct A(#[field(1, 2)] bool);",
            "type is smaller than the specified size of 2 bits",
            (1, 24), (1, 28)
        );
        parse_invalid!(
            "NonZero8", "struct A(#[field(1, 2)] bool);",
            "type is smaller than the specified size of 2 bits",
            (1, 24), (1, 28)
        );

        parse_valid!("8", "struct A(#[field(1, 7)] u8);");
        parse_valid!("NonZero8", "struct A(#[field(1, 7)] u8);");

        parse_invalid!(
            "16", "struct A(#[field(1, 15)] u8);",
            "type is smaller than the specified size of 15 bits",
            (1, 25), (1, 27)
        );
        parse_invalid!(
            "NonZero16", "struct A(#[field(1, 15)] u8);",
            "type is smaller than the specified size of 15 bits",
            (1, 25), (1, 27)
        );

        parse_valid!("16", "struct A(#[field(1, 15)] u16);");
        parse_valid!("NonZero16", "struct A(#[field(1, 15)] u16);");

        parse_invalid!(
            "32", "struct A(#[field(1, 31)] u8);",
            "type is smaller than the specified size of 31 bits",
            (1, 25), (1, 27)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(1, 31)] u8);",
            "type is smaller than the specified size of 31 bits",
            (1, 25), (1, 27)
        );

        parse_invalid!(
            "32", "struct A(#[field(1, 31)] u16);",
            "type is smaller than the specified size of 31 bits",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(1, 31)] u16);",
            "type is smaller than the specified size of 31 bits",
            (1, 25), (1, 28)
        );

        parse_valid!("32", "struct A(#[field(1, 31)] u32);");
        parse_valid!("NonZero32", "struct A(#[field(1, 31)] u32);");

        parse_invalid!(
            "64", "struct A(#[field(1, 63)] u8);",
            "type is smaller than the specified size of 63 bits",
            (1, 25), (1, 27)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 63)] u8);",
            "type is smaller than the specified size of 63 bits",
            (1, 25), (1, 27)
        );

        parse_invalid!(
            "64", "struct A(#[field(1, 63)] u16);",
            "type is smaller than the specified size of 63 bits",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 63)] u16);",
            "type is smaller than the specified size of 63 bits",
            (1, 25), (1, 28)
        );

        parse_invalid!(
            "64", "struct A(#[field(1, 63)] u32);",
            "type is smaller than the specified size of 63 bits",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 63)] u32);",
            "type is smaller than the specified size of 63 bits",
            (1, 25), (1, 28)
        );

        parse_valid!("64", "struct A(#[field(1, 63)] u64);");
        parse_valid!("NonZero64", "struct A(#[field(1, 63)] u64);");

        parse_invalid!(
            "128", "struct A(#[field(1, 127)] u8);",
            "type is smaller than the specified size of 127 bits",
            (1, 26), (1, 28)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 127)] u8);",
            "type is smaller than the specified size of 127 bits",
            (1, 26), (1, 28)
        );

        parse_invalid!(
            "128", "struct A(#[field(1, 127)] u16);",
            "type is smaller than the specified size of 127 bits",
            (1, 26), (1, 29)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 127)] u16);",
            "type is smaller than the specified size of 127 bits",
            (1, 26), (1, 29)
        );

        parse_invalid!(
            "128", "struct A(#[field(1, 127)] u32);",
            "type is smaller than the specified size of 127 bits",
            (1, 26), (1, 29)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 127)] u32);",
            "type is smaller than the specified size of 127 bits",
            (1, 26), (1, 29)
        );

        parse_invalid!(
            "128", "struct A(#[field(1, 127)] u64);",
            "type is smaller than the specified size of 127 bits",
            (1, 26), (1, 29)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 127)] u64);",
            "type is smaller than the specified size of 127 bits",
            (1, 26), (1, 29)
        );

        parse_valid!("128", "struct A(#[field(1, 127)] u128);");
        parse_valid!("NonZero128", "struct A(#[field(1, 127)] u128);");
    }

    #[test]
    fn validate_overlaps() {
        parse_invalid!(
            "8", "struct A { #[field(0, 2)] b: B, #[field(1, 2)] c: C }",
            "overlaps with field `b`, please specify `allow_overlaps` if this is intended",
            (1, 40), (1, 44)
        );

        parse_invalid!(
            "8, allow_overlaps", "struct A { #[field(0, 1)] b: B, #[field(1, 2)] c: C }",
            "unnecessary since no fields overlap",
            (1, 3), (1, 17)
        );

        parse_valid!("8, allow_overlaps", "struct A { #[field(0, 2)] b: B, #[field(1, 2)] c: C }");
    }

    #[test]
    fn validate_primitive_signed() {
        parse_invalid!(
            "16", "struct A(#[field(0, 9)] i8);",
            "type is smaller than the specified size of 9 bits",
            (1, 24), (1, 26)
        );
        parse_invalid!(
            "NonZero16", "struct A(#[field(0, 9)] i8);",
            "type is smaller than the specified size of 9 bits",
            (1, 24), (1, 26)
        );

        parse_invalid!(
            "8", "struct A(#[field(1, 7)] i8);",
            "a signed `i8` with a size of `7` bits can not store negative numbers, use either `u8` or `#[field(size = 8)]`",
            (1, 24), (1, 26)
        );
        parse_invalid!(
            "NonZero8", "struct A(#[field(1, 7)] i8);",
            "a signed `i8` with a size of `7` bits can not store negative numbers, use either `u8` or `#[field(size = 8)]`",
            (1, 24), (1, 26)
        );

        parse_valid!("16", "struct A(#[field(0, 8)] i8);");
        parse_valid!("NonZero16", "struct A(#[field(0, 8)] i8);");

        parse_valid!("8", "struct A(#[field(1, 7)] u8);");
        parse_valid!("NonZero8", "struct A(#[field(1, 7)] u8);");

        parse_invalid!(
            "32", "struct A(#[field(0, 17)] i16);",
            "type is smaller than the specified size of 17 bits",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(0, 17)] i16);",
            "type is smaller than the specified size of 17 bits",
            (1, 25), (1, 28)
        );

        parse_invalid!(
            "16", "struct A(#[field(1, 15)] i16);",
            "a signed `i16` with a size of `15` bits can not store negative numbers, use either `u16` or `#[field(size = 16)]`",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero16", "struct A(#[field(1, 15)] i16);",
            "a signed `i16` with a size of `15` bits can not store negative numbers, use either `u16` or `#[field(size = 16)]`",
            (1, 25), (1, 28)
        );

        parse_valid!("32", "struct A(#[field(0, 16)] i16);");
        parse_valid!("NonZero32", "struct A(#[field(0, 16)] i16);");

        parse_valid!("16", "struct A(#[field(1, 15)] u16);");
        parse_valid!("NonZero16", "struct A(#[field(1, 15)] u16);");

        parse_invalid!(
            "64", "struct A(#[field(0, 33)] i32);",
            "type is smaller than the specified size of 33 bits",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(0, 33)] i32);",
            "type is smaller than the specified size of 33 bits",
            (1, 25), (1, 28)
        );

        parse_invalid!(
            "32", "struct A(#[field(1, 31)] i32);",
            "a signed `i32` with a size of `31` bits can not store negative numbers, use either `u32` or `#[field(size = 32)]`",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(1, 31)] i32);",
            "a signed `i32` with a size of `31` bits can not store negative numbers, use either `u32` or `#[field(size = 32)]`",
            (1, 25), (1, 28)
        );

        parse_valid!("64", "struct A(#[field(0, 32)] i32);");
        parse_valid!("NonZero64", "struct A(#[field(0, 32)] i32);");

        parse_valid!("32", "struct A(#[field(1, 31)] u32);");
        parse_valid!("NonZero32", "struct A(#[field(1, 31)] u32);");

        parse_invalid!(
            "128", "struct A(#[field(0, 65)] i64);",
            "type is smaller than the specified size of 65 bits",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(0, 65)] i64);",
            "type is smaller than the specified size of 65 bits",
            (1, 25), (1, 28)
        );

        parse_invalid!(
            "64", "struct A(#[field(1, 63)] i64);",
            "a signed `i64` with a size of `63` bits can not store negative numbers, use either `u64` or `#[field(size = 64)]`",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 63)] i64);",
            "a signed `i64` with a size of `63` bits can not store negative numbers, use either `u64` or `#[field(size = 64)]`",
            (1, 25), (1, 28)
        );

        parse_valid!("128", "struct A(#[field(0, 64)] i64);");
        parse_valid!("NonZero128", "struct A(#[field(0, 64)] i64);");

        parse_valid!("64", "struct A(#[field(1, 63)] u64);");
        parse_valid!("NonZero64", "struct A(#[field(1, 63)] u64);");

        /* Not possible:
        parse_invalid!(
            "256", "struct A(#[field(0, 129)] i128);",
            "type is smaller than the specified size of 129 bits",
            (1, 26), (1, 30)
        );
        parse_invalid!(
            "NonZero256", "struct A(#[field(0, 129)] i128);",
            "type is smaller than the specified size of 129 bits",
            (1, 26), (1, 30)
        );
        */

        parse_invalid!(
            "128", "struct A(#[field(1, 127)] i128);",
            "a signed `i128` with a size of `127` bits can not store negative numbers, use either `u128` or `#[field(size = 128)]`",
            (1, 26), (1, 30)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(1, 127)] i128);",
            "a signed `i128` with a size of `127` bits can not store negative numbers, use either `u128` or `#[field(size = 128)]`",
            (1, 26), (1, 30)
        );

        // Not possible: parse_valid!("256", "struct A(#[field(0, 128)] i128);");
        // Not possible: parse_valid!("NonZero256", "struct A(#[field(0, 128)] i128);");

        parse_valid!("128", "struct A(#[field(1, 127)] u128);");
        parse_valid!("NonZero128", "struct A(#[field(1, 127)] u128);");
    }

    #[test]
    fn validate_primitive_size() {
        parse_valid!("8", "struct A(#[field(1, 7)] u8);");
        parse_valid!("NonZero8", "struct A(#[field(1, 7)] u8);");

        parse_invalid!(
            "8", "struct A(#[field(1, 7)] u16);",
            "bigger than the size of the bit field, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "NonZero8", "struct A(#[field(1, 7)] u16);",
            "bigger than the size of the bit field, use `u8` instead",
            (1, 24), (1, 27)
        );

        parse_invalid!(
            "8", "struct A(#[field(1, 7)] u32);",
            "bigger than the size of the bit field, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "NonZero8", "struct A(#[field(1, 7)] u32);",
            "bigger than the size of the bit field, use `u8` instead",
            (1, 24), (1, 27)
        );

        parse_invalid!(
            "8", "struct A(#[field(1, 7)] u64);",
            "bigger than the size of the bit field, use `u8` instead",
            (1, 24), (1, 27)
        );
        parse_invalid!(
            "NonZero8", "struct A(#[field(1, 7)] u64);",
            "bigger than the size of the bit field, use `u8` instead",
            (1, 24), (1, 27)
        );

        parse_invalid!(
            "8", "struct A(#[field(1, 7)] u128);",
            "bigger than the size of the bit field, use `u8` instead",
            (1, 24), (1, 28)
        );
        parse_invalid!(
            "NonZero8", "struct A(#[field(1, 7)] u128);",
            "bigger than the size of the bit field, use `u8` instead",
            (1, 24), (1, 28)
        );

        parse_valid!("16", "struct A(#[field(1, 7)] u8);");
        parse_valid!("NonZero16", "struct A(#[field(1, 7)] u8);");

        parse_valid!("16", "struct A(#[field(1, 15)] u16);");
        parse_valid!("NonZero16", "struct A(#[field(1, 15)] u16);");

        parse_invalid!(
            "16", "struct A(#[field(1, 15)] u32);",
            "bigger than the size of the bit field, use `u16` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero16", "struct A(#[field(1, 15)] u32);",
            "bigger than the size of the bit field, use `u16` instead",
            (1, 25), (1, 28)
        );

        parse_invalid!(
            "16", "struct A(#[field(1, 15)] u64);",
            "bigger than the size of the bit field, use `u16` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero16", "struct A(#[field(1, 15)] u64);",
            "bigger than the size of the bit field, use `u16` instead",
            (1, 25), (1, 28)
        );

        parse_invalid!(
            "16", "struct A(#[field(1, 15)] u128);",
            "bigger than the size of the bit field, use `u16` instead",
            (1, 25), (1, 29)
        );
        parse_invalid!(
            "NonZero16", "struct A(#[field(1, 15)] u128);",
            "bigger than the size of the bit field, use `u16` instead",
            (1, 25), (1, 29)
        );

        parse_valid!("32", "struct A(#[field(1, 7)] u8);");
        parse_valid!("NonZero32", "struct A(#[field(1, 7)] u8);");

        parse_valid!("32", "struct A(#[field(1, 15)] u16);");
        parse_valid!("NonZero32", "struct A(#[field(1, 15)] u16);");

        parse_valid!("32", "struct A(#[field(1, 31)] u32);");
        parse_valid!("NonZero32", "struct A(#[field(1, 31)] u32);");

        parse_invalid!(
            "32", "struct A(#[field(1, 31)] u64);",
            "bigger than the size of the bit field, use `u32` instead",
            (1, 25), (1, 28)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(1, 31)] u64);",
            "bigger than the size of the bit field, use `u32` instead",
            (1, 25), (1, 28)
        );

        parse_invalid!(
            "32", "struct A(#[field(1, 31)] u128);",
            "bigger than the size of the bit field, use `u32` instead",
            (1, 25), (1, 29)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(1, 31)] u128);",
            "bigger than the size of the bit field, use `u32` instead",
            (1, 25), (1, 29)
        );

        parse_valid!("64", "struct A(#[field(1, 7)] u8);");
        parse_valid!("NonZero64", "struct A(#[field(1, 7)] u8);");

        parse_valid!("64", "struct A(#[field(1, 15)] u16);");
        parse_valid!("NonZero64", "struct A(#[field(1, 15)] u16);");

        parse_valid!("64", "struct A(#[field(1, 31)] u32);");
        parse_valid!("NonZero64", "struct A(#[field(1, 31)] u32);");

        parse_valid!("64", "struct A(#[field(1, 63)] u64);");
        parse_valid!("NonZero64", "struct A(#[field(1, 63)] u64);");

        parse_invalid!(
            "64", "struct A(#[field(1, 63)] u128);",
            "bigger than the size of the bit field, use `u64` instead",
            (1, 25), (1, 29)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(1, 63)] u128);",
            "bigger than the size of the bit field, use `u64` instead",
            (1, 25), (1, 29)
        );

        parse_valid!("128", "struct A(#[field(1, 7)] u8);");
        parse_valid!("NonZero128", "struct A(#[field(1, 7)] u8);");

        parse_valid!("128", "struct A(#[field(1, 15)] u16);");
        parse_valid!("NonZero128", "struct A(#[field(1, 15)] u16);");

        parse_valid!("128", "struct A(#[field(1, 31)] u32);");
        parse_valid!("NonZero128", "struct A(#[field(1, 31)] u32);");

        parse_valid!("128", "struct A(#[field(1, 63)] u64);");
        parse_valid!("NonZero128", "struct A(#[field(1, 63)] u64);");

        parse_valid!("128", "struct A(#[field(1, 127)] u128);");
        parse_valid!("NonZero128", "struct A(#[field(1, 127)] u128);");
    }

    #[test]
    fn validate_size() {
        parse_invalid!(
            "8", "struct A(#[field(0, 8)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 21)
        );
        parse_invalid!(
            "NonZero8", "struct A(#[field(0, 8)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 21)
        );

        parse_invalid!(
            "16", "struct A(#[field(0, 16)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 22)
        );
        parse_invalid!(
            "NonZero16", "struct A(#[field(0, 16)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "32", "struct A(#[field(0, 32)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 22)
        );
        parse_invalid!(
            "NonZero32", "struct A(#[field(0, 32)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "64", "struct A(#[field(0, 64)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 22)
        );
        parse_invalid!(
            "NonZero64", "struct A(#[field(0, 64)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 22)
        );

        parse_invalid!(
            "128", "struct A(#[field(0, 128)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 23)
        );
        parse_invalid!(
            "NonZero128", "struct A(#[field(0, 128)] A);",
            "field has the size of the whole bit field, use a plain `A` instead",
            (1, 20), (1, 23)
        );
    }
}