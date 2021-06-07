use crate::ast::{PackedAttributes, PackedEnum, PackedStruct, PackedTuple, PackedUnit};
use syn::{
    parse::{Parse, ParseStream},
    Result, Token,
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

impl Container {
    pub fn ident(&self) -> &syn::Ident {
        self.data.ident()
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
            todo!("learn to parse union")
        } else {
            Err(syn::Error::new(input.span(), "not handled by `packtool`"))
        }
    }
}
