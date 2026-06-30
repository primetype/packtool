use crate::ast::{PackedAttributes, PackedField};
use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
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

    /// the catch-all `#[packed(fallback)]` variant, if one is declared.
    ///
    /// A fallback variant carries the raw `#[repr(...)]` integer for any
    /// discriminant that does not match a known unit variant, making the
    /// packed enum forward-compatible.
    pub fn fallback_variant(&self) -> Option<&PackedVariant> {
        self.variants.iter().find(|v| v.attributes.fallback)
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
        super::reject_generics(input)?;
        let _parentheses_token = syn::braced!(content in input);
        let variants = content.parse_terminated(PackedVariant::parse, Token![,])?;

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
        // capture the per-variant attributes (e.g. `#[packed(fallback)]`)
        let attributes: PackedAttributes = input.parse()?;
        let ident = input.parse()?;

        let fields = if input.peek(syn::token::Brace) {
            let content;
            let _brace_token = syn::braced!(content in input);
            content.parse_terminated(PackedField::parse_named, Token![,])?
        } else if input.peek(syn::token::Paren) {
            let content;
            let _brace_token = syn::parenthesized!(content in input);
            content.parse_terminated(PackedField::parse_unnamed, Token![,])?
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
