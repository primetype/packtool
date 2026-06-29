use syn::{Result, Token, parse::ParseStream};

mod container;
mod packed_attributes;
mod packed_enum;
mod packed_field;
mod packed_structure;
mod packed_tuple;
mod packed_unit;

/// Reject type-parameter generics on an enum.
///
/// Type-parameter generics ARE supported on named structs (the layout and
/// `SIZE` are computed through `<T as Packed>::SIZE`; see `PackedStruct`).
/// Enums remain concrete — their discriminant layout is fixed at the type
/// level — so a generic parameter list following the enum name is rejected
/// here with a clear error rather than failing later with a cryptic
/// "expected curly braces". (A generic tuple struct is routed to the struct
/// parser, where the missing brace after `<...>` rejects it.)
pub(crate) fn reject_generics(input: ParseStream) -> Result<()> {
    if input.peek(Token![<]) {
        Err(syn::Error::new(
            input.span(),
            "packtool cannot derive `Packed` for generic enums",
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
