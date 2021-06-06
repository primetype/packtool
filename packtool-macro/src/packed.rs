use proc_macro2::TokenStream;
use quote::quote;

use crate::ast::AContainer;

pub fn expand_derive_packed(input: &mut syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let mut context = Context::default();

    let container = match AContainer::from_ast(&mut context, input) {
        Some(cont) => cont,
        None => return Err(context.check().unwrap_err()),
    };

    context.check()?;

    let ident = &container.ident;
    let size = container.size();

    let block = quote! {
        impl Packed for #ident {
            const SIZE: usize = #size;
        }
    };

    Ok(block)
}
