use crate::ast::{PackedField, PackedUnit, PackedUnitOrigin};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct PackedStruct {
    pub _struct_token: Token!(struct),
    pub ident: syn::Ident,
    pub generics: syn::Generics,
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
            from: PackedUnitOrigin::Brace,
        }
        // NOTE: this only ever runs for a *non-generic* empty struct. An empty
        // struct that still carries generics is rejected upstream in
        // `Data::parse` (its type parameters could not appear in the layout),
        // so no generics are silently dropped here.
    }
}

impl Parse for PackedStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let _struct_token = input.parse()?;
        let ident = input.parse()?;
        // Type-parameter generics on STRUCTS are supported (the layout/`SIZE`
        // are computed from `<T as Packed>::SIZE`, all slice-based). Parse the
        // generic parameter list and the optional `where` clause (which for a
        // braced struct precedes the body).
        let mut generics: syn::Generics = input.parse()?;
        generics.where_clause = input.parse()?;
        let _parentheses_token = syn::braced!(content in input);
        let fields = content.parse_terminated(PackedField::parse_named, Token![,])?;

        Ok(Self {
            _struct_token,
            ident,
            generics,
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
