mod ast;
mod expand;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::ast::Container;

#[proc_macro_derive(Packed, attributes(packed))]
pub fn derive_packed(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Container);

    expand::packed_definitions(input).into()
}
