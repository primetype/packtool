use crate::ast::{PackedAttributes, PackedField};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result, Token,
};

pub struct PackedEnum {
    pub _struct_token: Token!(enum),
    pub ident: syn::Ident,
    pub _parentheses_token: syn::token::Brace,
    pub variants: Punctuated<PackedVariant, Token!(,)>,
}

pub struct PackedVariant {
    pub attributes: PackedAttributes,
    pub ident: syn::Ident,
    pub fields: Punctuated<PackedField, Token!(,)>,
    pub discriminant: Option<(syn::token::Eq, syn::Expr)>,
}

impl PackedEnum {
    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub fn only_unit_variants(&self) -> bool {
        self.variants.iter().all(|v| v.fields.is_empty())
    }

    pub fn equivalent_to_packed_unit(&self) -> bool {
        self.variants.is_empty()
    }
}

impl Parse for PackedEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let _struct_token = input.parse()?;
        let ident = input.parse()?;
        let _parentheses_token = syn::braced!(content in input);
        let variants = content.parse_terminated(PackedVariant::parse)?;

        Ok(Self {
            _struct_token,
            ident,
            _parentheses_token,
            variants,
        })
    }
}

impl Parse for PackedVariant {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes: PackedAttributes = input.parse()?;
        let ident = input.parse()?;

        let fields = if input.peek(syn::token::Brace) {
            let content;
            let _brace_token = syn::braced!(content in input);
            content.parse_terminated(PackedField::parse_named)?
        } else if input.peek(syn::token::Paren) {
            let content;
            let _brace_token = syn::parenthesized!(content in input);
            content.parse_terminated(PackedField::parse_unnamed)?
        } else {
            Punctuated::new()
        };

        let discriminant = if input.peek(Token![=]) {
            let eq_token: Token![=] = input.parse()?;
            let discriminant: syn::Expr = input.parse()?;
            Some((eq_token, discriminant))
        } else {
            None
        };

        Ok(Self {
            attributes,
            ident,
            fields,
            discriminant,
        })
    }
}
