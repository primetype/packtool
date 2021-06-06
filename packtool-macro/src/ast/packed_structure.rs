use crate::ast::{PackedField, PackedUnit};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result, Token,
};

pub struct PackedStruct {
    pub _struct_token: Token!(struct),
    pub ident: syn::Ident,
    pub _parentheses_token: syn::token::Brace,
    pub fields: Punctuated<PackedField, Token!(,)>,
}

impl PackedStruct {
    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub fn equivalent_to_packed_unit(&self) -> bool {
        self.fields.is_empty()
    }

    /// function will panic if it is not a valid equivalence to
    /// a [`PackedUnit`]
    pub fn into_unit(self) -> PackedUnit {
        assert!(self.fields.is_empty(), "Unit structures have no fields");

        PackedUnit {
            _struct_token: self._struct_token,
            ident: self.ident,
            _semi: syn::token::Semi::default(),
        }
    }
}

impl Parse for PackedStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let _struct_token = input.parse()?;
        let ident = input.parse()?;
        let _parentheses_token = syn::braced!(content in input);
        let fields = content.parse_terminated(PackedField::parse_named)?;

        Ok(Self {
            _struct_token,
            ident,
            _parentheses_token,
            fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equivalence_to_packed_unit() {
        let tuple: PackedStruct = syn::parse_str("struct Unit {}").unwrap();
        assert!(tuple.equivalent_to_packed_unit());

        let _unit: PackedUnit = tuple.into_unit();
    }

    #[test]
    fn parse() {
        let _tuple: PackedStruct = syn::parse_str("struct Unit { value : u8 }").unwrap();
        let _tuple: PackedStruct = syn::parse_str("struct Unit { f1: u8, f2: Type }").unwrap();
        let _tuple: PackedStruct =
            syn::parse_str("struct Unit { f1: u8, f2: Type, f3: Generic<Type> }").unwrap();
    }
}
