use syn::{
    parse::{Parse, ParseStream},
    Result, Token,
};

pub struct PackedUnit {
    pub _struct_token: Token!(struct),
    pub ident: syn::Ident,
    pub _semi: Token!(;),
}

impl Parse for PackedUnit {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(PackedUnit {
            _struct_token: input.parse()?,
            ident: input.parse()?,
            _semi: input.parse()?,
        })
    }
}

impl PackedUnit {
    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic = "expected `struct`"]
    fn expected_struct() {
        let _unit: PackedUnit = syn::parse_str("enum Unit;").unwrap();
    }

    #[test]
    #[should_panic = "expected `;`"]
    fn expected_unexpected_tuple() {
        let _unit: PackedUnit = syn::parse_str("struct Unit();").unwrap();
        let _unit: PackedUnit = syn::parse_str("struct Unit(usize);").unwrap();
        let _unit: PackedUnit = syn::parse_str("struct Unit(usize, char);").unwrap();
    }

    #[test]
    #[should_panic = "expected `;`"]
    fn expected_unexpected_regular() {
        let _unit: PackedUnit = syn::parse_str("struct Unit { field: usize }").unwrap();
    }

    #[test]
    fn parse() {
        let _unit: PackedUnit = syn::parse_str("struct Unit;").unwrap();
    }
}
