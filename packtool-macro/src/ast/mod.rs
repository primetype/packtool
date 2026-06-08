use syn::{parse::ParseStream, Result, Token};

mod container;
mod packed_attributes;
mod packed_enum;
mod packed_field;
mod packed_structure;
mod packed_tuple;
mod packed_unit;

/// packtool only supports concrete (non-generic) types: the packed layout
/// and `SIZE` are fixed at the type level. Detect a generic parameter list
/// following the type name and reject it with a clear error rather than
/// letting parsing fail later with a cryptic "expected curly braces".
pub(crate) fn reject_generics(input: ParseStream) -> Result<()> {
    if input.peek(Token![<]) {
        Err(syn::Error::new(
            input.span(),
            "packtool cannot derive `Packed` for generic types",
        ))
    } else {
        Ok(())
    }
}

pub use self::{
    container::{Container, Data},
    packed_attributes::{AccessorType, PackedAttributes, ValueType},
    packed_enum::{PackedEnum, PackedVariant},
    packed_field::PackedField,
    packed_structure::PackedStruct,
    packed_tuple::PackedTuple,
    packed_unit::{PackedUnit, PackedUnitOrigin},
};
