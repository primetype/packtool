use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Result,
};

#[derive(Default)]
pub struct PackedAttributes {
    pub value: Option<ValueType>,
    pub repr: Option<syn::Path>,
    pub accessor: AccessorType,
}

#[derive(Default)]
pub enum AccessorType {
    #[default]
    Default,
    Ignore,
    Custom(syn::Ident),
}

pub enum ValueType {
    Lit(syn::Lit),
    Const(syn::Path),
}

enum PackedAttribute {
    Value(ValueType),
    Repr(syn::Path),
    Accessor(proc_macro2::Span, AccessorType),
}

const ATTRIBUTE_LIST: &[&str] = &[PackedAttribute::VALUE, PackedAttribute::ACCESSOR];

impl ValueType {
    pub fn span(&self) -> proc_macro2::Span {
        match self {
            Self::Lit(lit) => lit.span(),
            Self::Const(con) => con.span(),
        }
    }
}

impl PackedAttributes {
    fn from_iter<T>(attributes: T) -> Result<Self>
    where
        T: IntoIterator<Item = PackedAttribute>,
    {
        let mut result = PackedAttributes::default();

        for attribute in attributes {
            match attribute {
                PackedAttribute::Value(value) => {
                    if result.value.is_some() {
                        return Err(syn::Error::new(value.span(), "Value was already set"));
                    } else {
                        result.value = Some(value);
                    }
                }
                PackedAttribute::Repr(path) => {
                    // leave the understanding of repr to the repr macro
                    // we only use it to detect if it was set in the
                    // case of enum with only unit variants
                    result.repr = Some(path);
                }
                PackedAttribute::Accessor(span, accessor) => {
                    if !matches!(result.accessor, AccessorType::Default) {
                        return Err(syn::Error::new(span, "The accessor has already been set"));
                    } else {
                        result.accessor = accessor;
                    }
                }
            }
        }

        Ok(result)
    }
}

impl Parse for PackedAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input
            .call(syn::Attribute::parse_outer)?
            .into_iter()
            .filter(|attr| attr.path().is_ident("packed") || attr.path().is_ident("repr"))
            .map(|attr| PackedAttribute::from(attr.meta))
            .collect::<Result<Vec<Vec<_>>>>()?
            .into_iter()
            .flatten();

        PackedAttributes::from_iter(attributes)
    }
}

impl PackedAttribute {
    const VALUE: &'static str = "value";
    const ACCESSOR: &'static str = "accessor";

    fn from(meta: syn::Meta) -> Result<Vec<Self>> {
        match meta {
            syn::Meta::Path(path) => Err(syn::Error::new_spanned(
                path,
                format!(
                    "expecting a list of packed parameters ({:?})",
                    ATTRIBUTE_LIST
                ),
            )),
            syn::Meta::NameValue(meta_named_value) => Err(syn::Error::new_spanned(
                meta_named_value,
                format!(
                    "expecting a list of packed parameters ({:?})",
                    ATTRIBUTE_LIST
                ),
            )),
            syn::Meta::List(meta_list) => {
                let is_repr = meta_list.path.is_ident("repr");

                let nested = meta_list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )?;

                let mut list = Vec::with_capacity(nested.len());

                for entry in nested.into_iter() {
                    list.push(Self::from_nested(is_repr, entry).map_err(|mut err| {
                        err.combine(syn::Error::new(
                            err.span(),
                            format!("Expecting one of {:?}", ATTRIBUTE_LIST),
                        ));
                        err
                    })?);
                }

                Ok(list)
            }
        }
    }

    fn from_nested(is_repr: bool, nested: syn::Meta) -> Result<Self> {
        match nested {
            syn::Meta::List(list) => {
                if list.path.is_ident(Self::VALUE) {
                    let inner = list.parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                    )?;
                    if inner.len() > 1 {
                        Err(syn::Error::new_spanned(list, "expecting only one value"))
                    } else if inner.is_empty() {
                        Err(syn::Error::new_spanned(list, "expecting one value"))
                    } else if let Some(syn::Meta::Path(path)) = inner.into_iter().next() {
                        Ok(Self::Value(ValueType::Const(path)))
                    } else {
                        Err(syn::Error::new_spanned(
                            list,
                            "expecting a constant path as value",
                        ))
                    }
                } else {
                    Err(syn::Error::new_spanned(list, "unexpected meta list"))
                }
            }
            meta @ syn::Meta::Path(_) if !is_repr => {
                Err(syn::Error::new_spanned(meta, "unexpected meta path"))
            }
            syn::Meta::Path(path) => Ok(Self::Repr(path)),
            syn::Meta::NameValue(name_value) => {
                let lit = match &name_value.value {
                    syn::Expr::Lit(syn::ExprLit { lit, .. }) => lit.clone(),
                    other => {
                        return Err(syn::Error::new_spanned(
                            other,
                            "expecting a literal value",
                        ))
                    }
                };
                if name_value.path.is_ident(Self::VALUE) {
                    Ok(Self::Value(ValueType::Lit(lit)))
                } else if name_value.path.is_ident(Self::ACCESSOR) {
                    let span = name_value.span();
                    if let syn::Lit::Str(ident) = lit {
                        let ident = syn::Ident::new(&ident.value(), ident.span());
                        Ok(Self::Accessor(span, AccessorType::Custom(ident)))
                    } else if let syn::Lit::Bool(enabled) = lit {
                        if enabled.value {
                            Ok(Self::Accessor(span, AccessorType::Default))
                        } else {
                            Ok(Self::Accessor(span, AccessorType::Ignore))
                        }
                    } else {
                        Err(syn::Error::new_spanned(
                            name_value,
                            "Set the value of the accessor: expecting a string literal",
                        ))
                    }
                } else {
                    Err(syn::Error::new_spanned(
                        name_value,
                        "Unknown meta attribute",
                    ))
                }
            }
        }
    }
}
