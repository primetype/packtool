use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;

use crate::ast::{
    AccessorType, Container, Data, PackedAttributes, PackedEnum, PackedField, PackedStruct,
    PackedTuple, PackedUnitOrigin, PackedVariant,
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

    quote! {
        impl #ident {
            #accessors
        }

        impl Packed for #ident {
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
            if enumeration.only_unit_variants() {
                check_only_enum_variants_have_discriminant(enumeration)?;
                if container.attributes.repr.is_none() {
                    return Err(syn::Error::new_spanned(
                        &enumeration._struct_token,
                        "Pure enumeration variants should have a repr(...) attributes to set the size",
                    ));
                }
            }
        }
    }

    Ok(())
}

fn check_only_enum_variants_have_discriminant(enumeration: &PackedEnum) -> Result<()> {
    assert!(enumeration.only_unit_variants());

    for variant in enumeration.variants.iter() {
        if variant.discriminant.is_none() {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "Missing explicit discriminant for packed enum",
            ));
        }
    }

    Ok(())
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
        return Err(syn::Error::new_spanned(
            value,
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

fn expand_size_from_enumeration(enumeration: &PackedEnum) -> TokenStream {
    assert!(
        !enumeration.variants.is_empty(),
        "unit enums should have been converted to a packed_unit"
    );

    if enumeration.only_unit_variants() {
        let ident = enumeration.ident();
        quote! { ::core::mem::size_of::<#ident>() }
    } else {
        todo!("variadic size enumeration not working yet")
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
    }
}

fn expand_size(container: &Container) -> TokenStream {
    match &container.data {
        Data::Unit(_) => expand_size_from_lit(
            container.ident(),
            container
                .attributes
                .value
                .as_ref()
                .expect("all units must have a packed(value = %)"),
        ),
        Data::Tuple(tuple) => expand_size_from_types(&tuple.fields),
        Data::Struct(structure) => expand_size_from_types(&structure.fields),
        Data::Enum(enumeration) => expand_size_from_enumeration(enumeration),
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
                        slice[0] == Some(#byte),
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
        Data::Unit(_) => expand_check_data_unit(
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

fn expand_read_from_slice_data_variants<'a, I>(
    repr: &syn::Path,
    ident: &syn::Ident,
    variants: I,
) -> TokenStream
where
    I: IntoIterator<Item = &'a PackedVariant>,
{
    let mut discriminants = Vec::new();

    for variant in variants.into_iter() {
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

    quote! {
        match #value {
            #( #discriminants ),*
            _ => panic!("Invalid discriminant"),
        }
    }
}

fn expand_read_from_slice_data_enumeration(
    repr: &syn::Path,
    ident: &syn::Ident,
    enumeration: &PackedEnum,
) -> TokenStream {
    let variants = expand_read_from_slice_data_variants(repr, ident, &enumeration.variants);

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
                    slice.copy_from_slice(String::from(#char).as_bytes());
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
    }
}

fn expand_write_to_slice_data_variants<'a, I>(
    repr: &syn::Path,
    ident: &syn::Ident,
    variants: I,
) -> TokenStream
where
    I: IntoIterator<Item = &'a PackedVariant>,
{
    let mut discriminants = Vec::new();

    for variant in variants.into_iter() {
        let (_, discriminant) = if let Some(discriminant) = variant.discriminant.as_ref() {
            discriminant
        } else {
            panic!("should always be a discriminant")
        };
        let variant = &variant.ident;

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
                #ident :: #variant => { #value }
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
    let variants = expand_write_to_slice_data_variants(repr, ident, &enumeration.variants);

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
            expand_write_to_slice_data_unit(container.attributes.value.as_ref().unwrap())
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
