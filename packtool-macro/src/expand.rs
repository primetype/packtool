use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

use crate::ast::{
    AccessorType, Container, Data, PackedAttributes, PackedEnum, PackedField, PackedStruct,
    PackedTuple, PackedUnitOrigin, PackedVariant, ValueType,
};

pub fn packed_definitions(container: Container) -> TokenStream {
    let ident = container.ident();

    if let Err(error) = check(&container) {
        return error.to_compile_error();
    }

    let size = expand_size(&container);
    let check = expand_check(&container);
    let unchecked_read_from_slice = expand_read_from_slice(&container);
    let unchecked_write_to_slice = expand_write_to_slice(&container);
    let accessors = expand_accessors(&container);

    // Capture the type's generics and, for every type parameter, inject a
    // `T: Packed` bound into the where-clause — the standard `derive` pattern.
    // That makes `<T as Packed>::SIZE` and the read/write/check calls valid in
    // the generated body. For concrete types `generics` is empty, so
    // `split_for_impl()` yields nothing and the output is unchanged.
    //
    // LIFETIMES: we only bind type parameters here. A `Packed` type OWNS its
    // bytes — `unchecked_read_from_slice` reconstructs an owned `Self` from a
    // slice, and there are no borrowing fields — so a lifetime parameter cannot
    // meaningfully appear on a `Packed` struct: any `&'a _` field would itself
    // have to be `Packed`, which borrows do not implement. We therefore neither
    // bind nor special-case lifetimes; a lifetime-bearing field simply fails the
    // `T: Packed` requirement on that field, as it should.
    let mut generics = container.generics();
    let type_param_idents: Vec<syn::Ident> =
        generics.type_params().map(|tp| tp.ident.clone()).collect();
    if !type_param_idents.is_empty() {
        let where_clause = generics.make_where_clause();
        for ident in &type_param_idents {
            where_clause
                .predicates
                .push(syn::parse_quote!(#ident: ::packtool::Packed));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #accessors
        }

        impl #impl_generics Packed for #ident #ty_generics #where_clause {
            const SIZE: usize = #size;

            #unchecked_read_from_slice
            #unchecked_write_to_slice

            #check
        }
    }
}

fn check(container: &Container) -> Result<()> {
    match &container.data {
        Data::Unit(unit) => {
            check_no_attribute_accessor("Unit", &container.attributes)?;
            // all unit types need to have a value associated
            if container.attributes.value.is_none() {
                return Err(syn::Error::new_spanned(
                    unit._struct_token,
                    "Expecting a value associated to this type (#[packed(valued = ...)])",
                ));
            }
        }
        Data::Tuple(t) => {
            check_no_attribute_value(
                "an unnamed struct (parenthesis struct)",
                &container.attributes,
            )?;
            check_no_attribute_accessor(
                "an unnamed struct (parenthesis struct)",
                &container.attributes,
            )?;
            check_no_value_in_field(&t.fields)?;
        }
        Data::Struct(s) => {
            check_no_attribute_value("a named struct (braced struct)", &container.attributes)?;
            check_no_attribute_accessor("a named struct (braced struct)", &container.attributes)?;
            check_no_value_in_field(&s.fields)?;
        }
        Data::Enum(enumeration) => {
            check_no_attribute_value("an enum", &container.attributes)?;
            check_no_attribute_accessor("an enum", &container.attributes)?;
            check_enum(container, enumeration)?;
        }
    }

    Ok(())
}

/// Validate a packed enum.
///
/// The supported shapes are:
///
/// * a pure unit enum — every variant is a unit with an explicit discriminant
///   and the enum carries a `#[repr(...)]` (the historical behaviour); or
/// * the above PLUS at most one catch-all `#[packed(fallback)]` variant, which
///   must be a single-field tuple variant with NO discriminant, whose field
///   type is exactly the `#[repr(...)]` integer type.
///
/// Any other shape — a data-carrying non-fallback variant, more than one
/// fallback, a fallback of the wrong arity, or a fallback whose field type does
/// not match the repr — is rejected here at compile time.
fn check_enum(container: &Container, enumeration: &PackedEnum) -> Result<()> {
    // at most one fallback variant.
    let fallbacks: Vec<&PackedVariant> = enumeration
        .variants
        .iter()
        .filter(|v| v.attributes.fallback)
        .collect();
    if let Some(extra) = fallbacks.get(1) {
        return Err(syn::Error::new_spanned(
            &extra.ident,
            "packed enums may declare at most one #[packed(fallback)] variant",
        ));
    }
    let fallback = fallbacks.first().copied();

    // every non-fallback variant must be a unit variant with an explicit
    // discriminant — unchanged from the historical behaviour.
    for variant in enumeration
        .variants
        .iter()
        .filter(|v| !v.attributes.fallback)
    {
        if !variant.fields.is_empty() {
            return Err(syn::Error::new_spanned(
                enumeration._struct_token,
                "packed enums with data-carrying variants are not supported yet",
            ));
        }
        if variant.discriminant.is_none() {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "Missing explicit discriminant for packed enum",
            ));
        }
    }

    // every packed enum needs a repr to fix its wire size.
    let repr = match container.attributes.repr.as_ref() {
        Some(repr) => repr,
        None => {
            return Err(syn::Error::new_spanned(
                enumeration._struct_token,
                "Pure enumeration variants should have a repr(...) attributes to set the size",
            ));
        }
    };

    // validate the fallback variant's shape.
    if let Some(fallback) = fallback {
        if fallback.discriminant.is_some() {
            return Err(syn::Error::new_spanned(
                &fallback.ident,
                "a #[packed(fallback)] variant must not declare a discriminant",
            ));
        }
        if fallback.fields.len() != 1 {
            return Err(syn::Error::new_spanned(
                &fallback.ident,
                "a #[packed(fallback)] variant must be a single-field tuple variant carrying the #[repr(...)] integer",
            ));
        }
        let field = fallback
            .fields
            .first()
            .expect("the fallback variant has exactly one field");
        if !fallback_field_matches_repr(&field.ty, repr) {
            return Err(syn::Error::new_spanned(
                &field.ty,
                "a #[packed(fallback)] variant's field type must be the same integer type as the enum's #[repr(...)]",
            ));
        }
    }

    Ok(())
}

/// `true` when the fallback field's type is exactly the repr integer type
/// (e.g. `Other(u16)` for `#[repr(u16)]`).
fn fallback_field_matches_repr(ty: &syn::Type, repr: &syn::Path) -> bool {
    let repr_ident = match repr.get_ident() {
        Some(ident) => ident,
        None => return false,
    };
    match ty {
        syn::Type::Path(type_path) if type_path.qself.is_none() => {
            type_path.path.is_ident(repr_ident)
        }
        _ => false,
    }
}

fn check_no_attribute_accessor(scope: &str, attributes: &PackedAttributes) -> Result<()> {
    if !matches!(attributes.accessor, AccessorType::Default) {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "Cannot have an accessor associated to {scope}",
                scope = scope
            ),
        ));
    }
    Ok(())
}

fn check_no_attribute_value(scope: &str, attributes: &PackedAttributes) -> Result<()> {
    if let Some(value) = attributes.value.as_ref() {
        return Err(syn::Error::new(
            value.span(),
            format!("Cannot have a value associated to {scope}", scope = scope),
        ));
    }
    Ok(())
}

fn check_no_value_in_field<'a, I>(fields: I) -> Result<()>
where
    I: IntoIterator<Item = &'a PackedField>,
{
    for field in fields {
        check_no_attribute_value("a field of a structure", &field.attributes)?;
    }

    Ok(())
}

fn expand_size_from_types<'a, I>(fields: I) -> TokenStream
where
    I: IntoIterator<Item = &'a PackedField>,
{
    let fields = fields.into_iter().map(|f| &f.ty);
    quote! { #( < #fields as Packed >::SIZE )+* }
}

fn expand_size_from_enumeration(repr: &syn::Path, enumeration: &PackedEnum) -> TokenStream {
    assert!(
        !enumeration.variants.is_empty(),
        "unit enums should have been converted to a packed_unit"
    );

    if enumeration.fallback_variant().is_some() {
        // a data-carrying fallback variant inflates `size_of::<Enum>()`, so the
        // wire size is taken from the repr integer width instead — the fallback
        // carries exactly the repr int and no extra bytes.
        quote! { ::core::mem::size_of::<#repr>() }
    } else if enumeration.only_unit_variants() {
        let ident = enumeration.ident();
        quote! { ::core::mem::size_of::<#ident>() }
    } else {
        todo!("variadic size enumeration not working yet")
    }
}

fn expand_size_from_value_type(ident: &syn::Ident, value: &ValueType) -> TokenStream {
    match value {
        ValueType::Lit(lit) => expand_size_from_lit(ident, lit),
        ValueType::Const(con) => {
            //
            quote! { ::std::mem::size_of_val(& #con) }
        }
    }
}

fn expand_size_from_lit(ident: &syn::Ident, value: &syn::Lit) -> TokenStream {
    match value {
        syn::Lit::Str(string) => {
            let len = string.value().len();
            quote! { #len }
        }
        syn::Lit::ByteStr(bytes) => {
            let len = bytes.value().len();
            quote! { #len }
        }
        syn::Lit::Byte(_) => {
            quote! { 1 }
        }
        syn::Lit::Char(char) => {
            let len = char.value().len_utf8();
            quote! { #len }
        }
        syn::Lit::Int(int) => {
            if int.suffix().is_empty() {
                quote! { ::core::mem::size_of::<#ident>() }
            } else {
                let ident = syn::Ident::new(int.suffix(), int.span());
                quote! { ::core::mem::size_of::<#ident>() }
            }
        }
        syn::Lit::Float(_) => {
            syn::Error::new_spanned(value, "floating point values are not supported")
                .to_compile_error()
        }
        syn::Lit::Bool(_) => {
            syn::Error::new_spanned(value, "boolean values are not supported").to_compile_error()
        }
        syn::Lit::Verbatim(_) => {
            syn::Error::new_spanned(value, "verbatim values are not supported").to_compile_error()
        }
        _ => syn::Error::new_spanned(value, "unsupported literal value").to_compile_error(),
    }
}

fn expand_size(container: &Container) -> TokenStream {
    match &container.data {
        Data::Unit(_) => expand_size_from_value_type(
            container.ident(),
            container
                .attributes
                .value
                .as_ref()
                .expect("all units must have a packed(value = %)"),
        ),
        Data::Tuple(tuple) => expand_size_from_types(&tuple.fields),
        Data::Struct(structure) => expand_size_from_types(&structure.fields),
        Data::Enum(enumeration) => expand_size_from_enumeration(
            container
                .attributes
                .repr
                .as_ref()
                .expect("Should have a repr on every enums"),
            enumeration,
        ),
    }
}

fn expand_check_data_unit_value(ident: &syn::Ident, value: &ValueType) -> TokenStream {
    match value {
        ValueType::Lit(lit) => expand_check_data_unit(ident, lit),
        ValueType::Const(con) => {
            quote! {
                fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
                    fn check_<C: Packed + ::std::fmt::Debug + PartialEq>(con: C, slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
                        let value = <C as Packed>::unchecked_read_from_slice(slice);

                        ::packtool::ensure!(
                            #ident,
                            value == con,
                            "Invalid value, expected {expected:?} but received {received:?}",
                            expected = con,
                            received = slice,
                        );

                        Ok(())
                    }

                    check_(#con, slice)

                }
            }
        }
    }
}

fn expand_check_data_unit(ident: &syn::Ident, value: &syn::Lit) -> TokenStream {
    match value {
        syn::Lit::Str(string) => {
            quote! {
                fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
                    ::packtool::ensure!(
                        #ident,
                        slice == #string.as_bytes(),
                        "Invalid string, expected {expected} but received {received}",
                        expected = #string,
                        received = ::std::string::String::from_utf8_lossy(slice),
                    );

                    Ok(())
                }
            }
        }
        syn::Lit::ByteStr(bytes) => {
            quote! {
                fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
                    ::packtool::ensure!(
                        #ident,
                        slice == #bytes,
                        "Invalid string, expected {expected:?} but received {received:?}",
                        expected = #bytes,
                        received = slice,
                    );

                    Ok(())
                }
            }
        }
        syn::Lit::Byte(byte) => {
            quote! {
                fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
                    ::packtool::ensure!(
                        #ident,
                        slice[0] == #byte,
                        "Invalid byte string, expected {expected:X} but received {received:X}",
                        expected = #byte,
                        received = slice[0],
                    );

                    Ok(())
                }
            }
        }
        syn::Lit::Char(char) => {
            quote! {
                fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
                    use ::packtool::Context as _;
                    let c = ::std::str::from_utf8(slice)
                        .context("Failed to parse valid utf8 char from the slice")?;

                    ::packtool::ensure!(
                        #ident,
                        c.chars().next() == Some(#char),
                        "Invalid UTF8 encoded char, expected {expected} but received {received}",
                        expected = #char,
                        received = c,
                    );

                    Ok(())
                }
            }
        }
        syn::Lit::Int(int) => {
            if int.suffix().is_empty() {
                syn::Error::new_spanned(
                    int,
                    "expect to know the exact type of the value, add suffix (like in `0i64`)",
                )
                .to_compile_error()
            } else {
                let ident = syn::Ident::new(int.suffix(), int.span());
                quote! {
                    fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
                        use ::packtool::Context as _;
                        use ::core::convert::TryInto as _;
                        let int = <#ident>::from_le_bytes(
                            slice.try_into()
                                .context("expecting to parse integer value")?
                        );

                        ::packtool::ensure!(
                            #ident,
                            int == #int,
                            "Invalid packed integer, expected {expected} but received {received}",
                            expected = #int,
                            received = int,
                        );

                        Ok(())
                    }
                }
            }
        }
        syn::Lit::Float(_) => {
            syn::Error::new_spanned(value, "floating point values are not supported")
                .to_compile_error()
        }
        syn::Lit::Bool(_) => {
            syn::Error::new_spanned(value, "boolean values are not supported").to_compile_error()
        }
        syn::Lit::Verbatim(_) => {
            syn::Error::new_spanned(value, "verbatim values are not supported").to_compile_error()
        }
        _ => syn::Error::new_spanned(value, "unsupported literal value").to_compile_error(),
    }
}

fn expand_check_data_field(
    field: &PackedField,
    index: usize,
    start: TokenStream,
) -> (TokenStream, TokenStream) {
    let ty = &field.ty;
    let on_error = if let Some(ident) = field.ident.as_ref() {
        quote! {
            context(
                ::packtool::Error::invalid_field::<#ty>(
                    stringify!(#ident)
                )
            )
        }
    } else {
        quote! {
            context(
                ::packtool::Error::invalid_tuple::<#ty>(
                    #index
                )
            )
        }
    };

    let end = quote! {
        #start + <#ty as Packed>::SIZE
    };
    let quote = quote! {
        <#ty as Packed>::check(&slice[(#start)..(#end)]).#on_error?;
    };

    (quote, end)
}

fn expand_check_data_fields<'a, I>(fields: I) -> TokenStream
where
    I: IntoIterator<Item = &'a PackedField>,
{
    let mut checks = Vec::new();

    let mut start = quote! { 0 };
    for (index, field) in fields.into_iter().enumerate() {
        let (check, end) = expand_check_data_field(field, index, start.clone());
        checks.push(check);
        start = end;
    }

    quote! { #(#checks)* }
}

fn expand_check_data_variants<'a, I>(repr: &syn::Path, variants: I) -> TokenStream
where
    I: IntoIterator<Item = &'a PackedVariant>,
{
    let mut discriminants = Vec::new();

    for variant in variants.into_iter() {
        let discriminant = if let Some(discriminant) = variant.discriminant.as_ref() {
            discriminant
        } else {
            panic!("should always be a discriminant")
        };
        discriminants.push(&discriminant.1);
    }

    let value = if repr.is_ident("u8") {
        quote! { slice[0] }
    } else if repr.is_ident("i8") {
        quote! { slice[0] as i8 }
    } else {
        quote! {
            <#repr>::from_le_bytes(
                slice.try_into()
                    .context("invalid length")?
            )
        }
    };

    quote! {
        match #value {
            #[allow(clippy::unused_unit)]
            # ( #discriminants )|* => {
                ()
            }
            found => return Err(
                ::packtool::Error::invalid_discriminant::<Self, _>(
                    found,
                    ::core::concat!(#(#discriminants , ", "),*),
                )
            ),
        }
    }
}

fn expand_check_data_tuple(tuple: &PackedTuple) -> TokenStream {
    let fields = expand_check_data_fields(&tuple.fields);
    quote! {
        fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
            use ::core::convert::TryInto as _;
            use ::packtool::Context as _;

            #fields

            Ok(())
        }
    }
}

fn expand_check_data_structure(structure: &PackedStruct) -> TokenStream {
    let fields = expand_check_data_fields(&structure.fields);

    quote! {
        fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
            use ::core::convert::TryInto as _;
            use ::packtool::Context as _;

            #fields

            Ok(())
        }
    }
}

fn expand_check_data_enumeration(repr: &syn::Path, enumeration: &PackedEnum) -> TokenStream {
    if enumeration.fallback_variant().is_some() {
        // with a fallback every repr value is valid: an unknown discriminant
        // simply decodes into the catch-all variant carrying the raw integer.
        return quote! {
            fn check(_slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
                Ok(())
            }
        };
    }

    let variants = expand_check_data_variants(repr, &enumeration.variants);

    quote! {
        fn check(slice: &[u8]) -> ::std::result::Result<(), ::packtool::Error> {
            use ::core::convert::TryInto as _;
            use ::packtool::Context as _;

            #variants

            Ok(())
        }
    }
}

fn expand_check(container: &Container) -> TokenStream {
    match &container.data {
        Data::Unit(_) => expand_check_data_unit_value(
            container.ident(),
            container
                .attributes
                .value
                .as_ref()
                .expect("all units must have a packed(value = %)"),
        ),
        Data::Tuple(tuple) => expand_check_data_tuple(tuple),
        Data::Struct(structure) => expand_check_data_structure(structure),
        Data::Enum(enumeration) => expand_check_data_enumeration(
            container
                .attributes
                .repr
                .as_ref()
                .expect("Should have a repr on every enums"),
            enumeration,
        ),
    }
}

fn expand_read_from_slice_data_unit(ident: &syn::Ident, from: &PackedUnitOrigin) -> TokenStream {
    let constructor = match from {
        PackedUnitOrigin::Unit => quote! { #ident },
        PackedUnitOrigin::Tuple => quote! { #ident () },
        PackedUnitOrigin::Brace => quote! { #ident {} },
    };

    quote! {
        fn unchecked_read_from_slice(_view: &[u8]) -> Self {
            #constructor
        }
    }
}

fn expand_read_from_slice_data_variants(
    repr: &syn::Path,
    ident: &syn::Ident,
    enumeration: &PackedEnum,
) -> TokenStream {
    let fallback = enumeration.fallback_variant();

    let mut discriminants = Vec::new();

    for variant in enumeration
        .variants
        .iter()
        .filter(|v| !v.attributes.fallback)
    {
        let (_, discriminant) = if let Some(discriminant) = variant.discriminant.as_ref() {
            discriminant
        } else {
            panic!("should always be a discriminant")
        };
        let variant = &variant.ident;

        discriminants.push({
            quote! {
                #discriminant => { #ident :: #variant }
            }
        });
    }

    let value = if repr.is_ident("u8") {
        quote! { slice[0] }
    } else if repr.is_ident("i8") {
        quote! { slice[0] as i8 }
    } else {
        quote! {
            <#repr>::from_le_bytes(
                slice.try_into().unwrap()
            )
        }
    };

    // an unknown discriminant either decodes into the fallback variant carrying
    // the raw repr integer, or — with no fallback — is unreachable because
    // `check` already rejected it.
    let default = if let Some(fallback) = fallback {
        let fallback = &fallback.ident;
        quote! { unknown => { #ident :: #fallback ( unknown ) } }
    } else {
        quote! { _ => ::core::panic!("Invalid discriminant") }
    };

    quote! {
        match #value {
            #( #discriminants )*
            #default
        }
    }
}

fn expand_read_from_slice_data_enumeration(
    repr: &syn::Path,
    ident: &syn::Ident,
    enumeration: &PackedEnum,
) -> TokenStream {
    let variants = expand_read_from_slice_data_variants(repr, ident, enumeration);

    quote! {
        fn unchecked_read_from_slice(slice: &[u8]) -> Self {
            use ::core::convert::TryInto as _;

            #variants
        }
    }
}

fn expand_read_from_slice_data_field(
    field: &PackedField,
    start: TokenStream,
) -> (TokenStream, TokenStream) {
    let ty = &field.ty;

    let end = quote! {
        #start + <#ty as Packed>::SIZE
    };
    let quote = if let Some(ident) = field.ident.as_ref() {
        quote! {
            #ident : <#ty as Packed>::unchecked_read_from_slice(&slice[(#start)..(#end)])
        }
    } else {
        quote! {
            <#ty as Packed>::unchecked_read_from_slice(&slice[(#start)..(#end)])
        }
    };

    (quote, end)
}

fn expand_read_from_slice_data_fields<'a, I>(fields: I) -> TokenStream
where
    I: IntoIterator<Item = &'a PackedField>,
{
    let mut checks = Vec::new();

    let mut start = quote! { 0 };
    for field in fields.into_iter() {
        let (check, end) = expand_read_from_slice_data_field(field, start.clone());
        checks.push(check);
        start = end;
    }

    quote! { #(#checks),* }
}

fn expand_read_from_slice_data_tuple(tuple: &PackedTuple) -> TokenStream {
    let ident = tuple.ident();
    let fields = expand_read_from_slice_data_fields(&tuple.fields);
    quote! {
        fn unchecked_read_from_slice(slice: &[u8]) -> Self {
            use ::core::convert::TryInto as _;

            #ident (
                #fields
            )
        }
    }
}

fn expand_read_from_slice_data_structure(structure: &PackedStruct) -> TokenStream {
    let fields = expand_read_from_slice_data_fields(&structure.fields);
    let ident = structure.ident();

    quote! {
        fn unchecked_read_from_slice(slice: &[u8]) -> Self {
            use ::core::convert::TryInto as _;

            #ident {
                #fields
            }
        }
    }
}

fn expand_read_from_slice(container: &Container) -> TokenStream {
    match &container.data {
        Data::Unit(unit) => expand_read_from_slice_data_unit(container.ident(), &unit.from),
        Data::Tuple(tuple) => expand_read_from_slice_data_tuple(tuple),
        Data::Struct(structure) => expand_read_from_slice_data_structure(structure),
        Data::Enum(enumeration) => expand_read_from_slice_data_enumeration(
            container
                .attributes
                .repr
                .as_ref()
                .expect("Should have a repr on every enums"),
            container.ident(),
            enumeration,
        ),
    }
}

fn expand_write_to_slice_data_unit_value(value: &ValueType) -> TokenStream {
    match value {
        ValueType::Lit(lit) => expand_write_to_slice_data_unit(lit),
        ValueType::Const(con) => {
            quote! {
                fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
                    #con.unchecked_write_to_slice(slice);
                }
            }
        }
    }
}

fn expand_write_to_slice_data_unit(value: &syn::Lit) -> TokenStream {
    match value {
        syn::Lit::Str(string) => {
            quote! {
                fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
                    slice.copy_from_slice(#string.as_bytes());
                }
            }
        }
        syn::Lit::ByteStr(bytes) => {
            quote! {
                fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
                    slice.copy_from_slice(#bytes);
                }
            }
        }
        syn::Lit::Byte(byte) => {
            quote! {
                fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
                    slice[0] = #byte;
                }
            }
        }
        syn::Lit::Char(char) => {
            quote! {
                fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
                    slice.copy_from_slice(#char.encode_utf8(&mut [0u8; 4]).as_bytes());
                }
            }
        }
        syn::Lit::Int(int) => {
            if int.suffix().is_empty() {
                syn::Error::new_spanned(
                    int,
                    "expect to know the exact type of the value, add suffix (like in `0i64`)",
                )
                .to_compile_error()
            } else {
                quote! {
                    fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
                        slice.copy_from_slice(
                            &(#int).to_le_bytes()
                        );
                    }
                }
            }
        }
        syn::Lit::Float(_) => {
            syn::Error::new_spanned(value, "floating point values are not supported")
                .to_compile_error()
        }
        syn::Lit::Bool(_) => {
            syn::Error::new_spanned(value, "boolean values are not supported").to_compile_error()
        }
        syn::Lit::Verbatim(_) => {
            syn::Error::new_spanned(value, "verbatim values are not supported").to_compile_error()
        }
        _ => syn::Error::new_spanned(value, "unsupported literal value").to_compile_error(),
    }
}

fn expand_write_to_slice_data_variants(
    repr: &syn::Path,
    ident: &syn::Ident,
    enumeration: &PackedEnum,
) -> TokenStream {
    let mut discriminants = Vec::new();

    for variant in enumeration.variants.iter() {
        let variant_ident = &variant.ident;

        if variant.attributes.fallback {
            // the fallback variant binds its raw repr integer and writes it
            // verbatim — exactly the repr width, no discriminant.
            let value = if repr.is_ident("u8") || repr.is_ident("i8") {
                quote! { slice[0] = *value; }
            } else {
                quote! {
                    slice.copy_from_slice(&<#repr>::to_le_bytes(*value));
                }
            };

            discriminants.push(quote! {
                #ident :: #variant_ident ( value ) => { #value }
            });
            continue;
        }

        let (_, discriminant) = if let Some(discriminant) = variant.discriminant.as_ref() {
            discriminant
        } else {
            panic!("should always be a discriminant")
        };

        let value = if repr.is_ident("u8") {
            quote! { slice[0] = #discriminant; }
        } else if repr.is_ident("i8") {
            quote! { slice[0] = #discriminant as i8; }
        } else {
            quote! {
                slice.copy_from_slice(&<#repr>::to_le_bytes(#discriminant));
            }
        };

        discriminants.push({
            quote! {
                #ident :: #variant_ident => { #value }
            }
        });
    }

    quote! {
        match self {
            #( #discriminants ),*
        }
    }
}

fn expand_write_to_slice_data_enumeration(
    repr: &syn::Path,
    ident: &syn::Ident,
    enumeration: &PackedEnum,
) -> TokenStream {
    let variants = expand_write_to_slice_data_variants(repr, ident, enumeration);

    quote! {
        fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
            #variants
        }
    }
}

fn expand_write_to_slice_data_field(
    field: &PackedField,
    index: syn::Index,
    start: TokenStream,
) -> (TokenStream, TokenStream) {
    let ty = &field.ty;

    let end = quote! {
        #start + <#ty as Packed>::SIZE
    };
    let quote = if let Some(ident) = field.ident.as_ref() {
        quote! {
            self.#ident.unchecked_write_to_slice(&mut slice[(#start)..(#end)])
        }
    } else {
        quote! {
            self.#index.unchecked_write_to_slice(&mut slice[(#start)..(#end)])
        }
    };

    (quote, end)
}

fn expand_write_to_slice_data_fields<'a, I>(fields: I) -> TokenStream
where
    I: IntoIterator<Item = &'a PackedField>,
{
    let mut checks = Vec::new();

    let mut start = quote! { 0 };
    for (index, field) in fields.into_iter().enumerate() {
        let (check, end) =
            expand_write_to_slice_data_field(field, syn::Index::from(index), start.clone());
        checks.push(check);
        start = end;
    }

    quote! { #(#checks);* }
}

fn expand_write_to_slice_data_tuple(tuple: &PackedTuple) -> TokenStream {
    let fields = expand_write_to_slice_data_fields(&tuple.fields);
    quote! {
        fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
            use ::core::convert::TryInto as _;

            #fields
        }
    }
}

fn expand_write_to_slice_data_structure(structure: &PackedStruct) -> TokenStream {
    let fields = expand_write_to_slice_data_fields(&structure.fields);

    quote! {
        fn unchecked_write_to_slice(&self, slice: &mut [u8]) {
            #fields
        }
    }
}

fn expand_write_to_slice(container: &Container) -> TokenStream {
    match &container.data {
        Data::Unit(_) => {
            expand_write_to_slice_data_unit_value(container.attributes.value.as_ref().unwrap())
        }
        Data::Tuple(tuple) => expand_write_to_slice_data_tuple(tuple),
        Data::Struct(structure) => expand_write_to_slice_data_structure(structure),
        Data::Enum(enumeration) => expand_write_to_slice_data_enumeration(
            container
                .attributes
                .repr
                .as_ref()
                .expect("Should have a repr on every enums"),
            container.ident(),
            enumeration,
        ),
    }
}

fn expand_field_accessor(
    field: &PackedField,
    index: usize,
    start: TokenStream,
) -> (TokenStream, TokenStream) {
    let ty = &field.ty;
    let end = quote! {
        #start + <#ty as Packed>::SIZE
    };

    let ident = match &field.attributes.accessor {
        AccessorType::Ignore => return (quote! {}, end),
        AccessorType::Custom(ident) => ident.clone(),
        AccessorType::Default => {
            if let Some(ident) = field.ident.as_ref() {
                ident.clone()
            } else {
                syn::Ident::new(&format!("_{}", index), proc_macro2::Span::call_site())
            }
        }
    };

    let accessor = quote! {
        pub fn #ident<'a>(view: ::packtool::View<'a, Self>) -> ::packtool::View<'a, #ty> {
            ::packtool::View::unchecked_from_slice(&view.as_slice()[#start..#end])
        }
    };

    (accessor, end)
}

fn expand_fields_accessors<'a, I>(fields: I) -> TokenStream
where
    I: IntoIterator<Item = &'a PackedField>,
{
    let mut fields_accessors = Vec::new();

    let mut start = quote! { 0 };
    for (index, field) in fields.into_iter().enumerate() {
        let (accessor, end) = expand_field_accessor(field, index, start.clone());
        fields_accessors.push(accessor);
        start = end;
    }

    quote! {
        #( #fields_accessors )*
    }
}

fn expand_tuple_accessors(tuple: &PackedTuple) -> TokenStream {
    let fields_accessors = expand_fields_accessors(&tuple.fields);

    quote! {
         #fields_accessors
    }
}

fn expand_structure_accessors(structure: &PackedStruct) -> TokenStream {
    let fields_accessors = expand_fields_accessors(&structure.fields);

    quote! {
         #fields_accessors
    }
}

fn expand_accessors(container: &Container) -> TokenStream {
    match &container.data {
        Data::Unit(_) => {
            // no accessor for the unit type
            quote! {}
        }
        Data::Enum(_enumeration) => {
            // no accessor for the enum type
            quote! {}
        }
        Data::Tuple(tuple) => expand_tuple_accessors(tuple),
        Data::Struct(structure) => expand_structure_accessors(structure),
    }
}
