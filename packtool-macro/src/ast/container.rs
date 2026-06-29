use crate::ast::{PackedAttributes, PackedEnum, PackedStruct, PackedTuple, PackedUnit};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
};

pub struct Container {
    pub attributes: PackedAttributes,
    pub data: Data,
    pub _visibility: syn::Visibility,
}

pub enum Data {
    Unit(PackedUnit),
    Tuple(PackedTuple),
    Struct(PackedStruct),
    Enum(PackedEnum),
    //Union(PackedUnion),
}

impl Data {
    pub fn ident(&self) -> &syn::Ident {
        match self {
            Self::Unit(unit) => unit.ident(),
            Self::Tuple(tuple) => tuple.ident(),
            Self::Struct(structure) => structure.ident(),
            Self::Enum(enumeration) => enumeration.ident(),
        }
    }
}

impl Data {
    /// the generics declared on the type, if any.
    ///
    /// Only type-parameter generics on structs are supported; every other
    /// shape is concrete and yields the empty (default) generics, so the
    /// emitted `impl` is byte-for-byte what it was before generics existed.
    pub fn generics(&self) -> syn::Generics {
        match self {
            Self::Struct(structure) => structure.generics.clone(),
            Self::Unit(_) | Self::Tuple(_) | Self::Enum(_) => syn::Generics::default(),
        }
    }
}

impl Container {
    pub fn ident(&self) -> &syn::Ident {
        self.data.ident()
    }

    pub fn generics(&self) -> syn::Generics {
        self.data.generics()
    }
}

impl Parse for Container {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attributes: input.parse()?,
            _visibility: input.parse()?,
            data: input.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_generic_named_struct() {
        // a named struct whose field depends on `T` is supported.
        let data = syn::parse_str::<Data>("struct Wrapper<T> { inner: T }").unwrap();
        assert!(matches!(data, Data::Struct(_)));
    }

    #[test]
    fn reject_empty_generic_struct() {
        // an empty generic struct has no field to carry `T`; reject it.
        let err = syn::parse_str::<Data>("struct Empty<T> {}")
            .err()
            .expect("empty generic struct must be rejected");
        assert!(
            err.to_string().contains("generic struct with no fields"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn reject_generic_enum() {
        // generics remain scoped OUT of enums (regression lock).
        let err = syn::parse_str::<Data>("enum Either<L, R> { Left(L), Right(R) }")
            .err()
            .expect("generic enum must be rejected");
        assert!(
            err.to_string().contains("generic"),
            "unexpected error: {err}"
        );
    }
}

impl Parse for Data {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token!(struct)) {
            if input.peek3(Token!(;)) {
                input.parse().map(Data::Unit)
            } else if input.peek3(syn::token::Paren) {
                let tuple: PackedTuple = input.parse()?;
                if tuple.equivalent_to_packed_unit() {
                    Ok(Data::Unit(tuple.into_unit()))
                } else {
                    Ok(Data::Tuple(tuple))
                }
            } else {
                let structure: PackedStruct = input.parse()?;
                if structure.equivalent_to_packed_unit() {
                    // A generic struct with no fields is meaningless: the type
                    // parameters cannot appear in the (empty) packed layout, so
                    // reject it rather than silently dropping the generics.
                    if !structure.generics.params.is_empty() {
                        return Err(syn::Error::new(
                            structure.ident.span(),
                            "packtool cannot derive `Packed` for a generic struct with no fields: \
                             the type parameters are unused and the packed layout cannot depend on them",
                        ));
                    }
                    Ok(Data::Unit(structure.into_unit()))
                } else {
                    Ok(Data::Struct(structure))
                }
            }
        } else if input.peek(Token!(enum)) {
            let enumeration: PackedEnum = input.parse()?;
            if enumeration.equivalent_to_packed_unit() {
                Err(syn::Error::new(
                    input.span(),
                    "zero-variant enums cannot be packed. This is because they cannot be instantiated.",
                ))
            } else {
                Ok(Data::Enum(enumeration))
            }
        } else if input.peek(Token!(union)) {
            Err(syn::Error::new(
                input.span(),
                "packtool cannot derive `Packed` for unions",
            ))
        } else {
            Err(syn::Error::new(input.span(), "not handled by `packtool`"))
        }
    }
}
