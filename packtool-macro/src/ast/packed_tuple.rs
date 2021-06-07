use crate::ast::{PackedField, PackedUnit, PackedUnitOrigin};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result, Token,
};

pub struct PackedTuple {
    pub _struct_token: Token!(struct),
    pub ident: syn::Ident,
    pub _parentheses_token: syn::token::Paren,
    pub fields: Punctuated<PackedField, Token!(,)>,
    pub _semi: Token!(;),
}

impl PackedTuple {
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
            _semi: self._semi,
            from: PackedUnitOrigin::Tuple,
        }
    }
}

impl Parse for PackedTuple {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let _struct_token = input.parse()?;
        let ident = input.parse()?;
        let _parentheses_token = syn::parenthesized!(content in input);
        let fields = content.parse_terminated(PackedField::parse_unnamed)?;
        let _semi = input.parse()?;

        #[allow(clippy::eval_order_dependence)]
        Ok(Self {
            _struct_token,
            ident,
            _parentheses_token,
            fields,
            _semi,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equivalence_to_packed_unit() {
        let tuple: PackedTuple = syn::parse_str("struct Unit();").unwrap();
        assert!(tuple.equivalent_to_packed_unit());

        let _unit: PackedUnit = tuple.into_unit();
    }

    #[test]
    fn parse() {
        let _tuple: PackedTuple = syn::parse_str("struct Unit(u8);").unwrap();
        let _tuple: PackedTuple = syn::parse_str("struct Unit(u8, Type);").unwrap();
        let _tuple: PackedTuple = syn::parse_str("struct Unit(u8, Type, Generic<Type>);").unwrap();
    }
}
