use crate::ast::PackedAttributes;
use syn::{parse::ParseStream, Result, Token};

pub struct PackedField {
    pub attributes: PackedAttributes,
    pub _visibility: syn::Visibility,
    pub ident: Option<syn::Ident>,
    pub _colon_token: Option<Token!(:)>,
    pub ty: syn::Type,
}

impl PackedField {
    pub fn parse_named(input: ParseStream) -> Result<Self> {
        let attributes = input.parse()?;
        let _visibility = input.parse()?;
        let ident = Some(input.parse()?);
        let _colon_token = Some(input.parse()?);
        let ty = input.parse()?;

        Ok(PackedField {
            attributes,
            _visibility,
            ident,
            _colon_token,
            ty,
        })
    }

    pub fn parse_unnamed(input: ParseStream) -> Result<Self> {
        let attributes = input.parse()?;
        let _visibility = input.parse()?;
        let ident = None;
        let _colon_token = None;
        let ty = input.parse()?;

        Ok(PackedField {
            attributes,
            _visibility,
            ident,
            _colon_token,
            ty,
        })
    }
}
