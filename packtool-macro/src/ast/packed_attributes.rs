use syn::{
    parse::{Parse, ParseStream},
    Result,
};

#[derive(Default)]
pub struct PackedAttributes {
    pub value: Option<syn::Lit>,
    pub repr: Option<syn::Path>,
}

enum PackedAttribute {
    Value(syn::Lit),
    Repr(syn::Path),
}

const ATTRIBUTE_LIST: &[&str] = &[PackedAttribute::VALUE];

impl PackedAttributes {
    fn from_iter<T>(attributes: T) -> Result<Self>
    where
        T: IntoIterator<Item = PackedAttribute>,
    {
        let mut result = PackedAttributes::default();

        for attribute in attributes {
            match attribute {
                PackedAttribute::Value(lit) => {
                    if result.value.is_some() {
                        return Err(syn::Error::new_spanned(lit, "Value was already set"));
                    } else {
                        result.value = Some(lit);
                    }
                }
                PackedAttribute::Repr(path) => {
                    // leave the understanding of repr to the repr macro
                    // we only use it to detect if it was set in the
                    // case of enum with only unit variants
                    result.repr = Some(path);
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
            .filter(|attr| attr.path.is_ident("packed") || attr.path.is_ident("repr"))
            .map(|attr| attr.parse_meta())
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(PackedAttribute::from);

        PackedAttributes::from_iter(
            attributes
                .collect::<Result<Vec<Vec<_>>>>()?
                .into_iter()
                .flatten(),
        )
    }
}

impl PackedAttribute {
    const VALUE: &'static str = "value";

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
                let mut list = Vec::with_capacity(meta_list.nested.len());

                let is_repr = meta_list.path.is_ident("repr");

                for entry in meta_list.nested.into_iter() {
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

    fn from_nested(is_repr: bool, nested: syn::NestedMeta) -> Result<Self> {
        match nested {
            meta @ syn::NestedMeta::Lit(_) => {
                Err(syn::Error::new_spanned(meta, "Unexpected literal"))
            }
            meta @ syn::NestedMeta::Meta(syn::Meta::List(_)) => {
                Err(syn::Error::new_spanned(meta, "unexpected meta list"))
            }
            meta @ syn::NestedMeta::Meta(syn::Meta::Path(_)) if !is_repr => {
                Err(syn::Error::new_spanned(meta, "unexpected meta path"))
            }
            syn::NestedMeta::Meta(syn::Meta::Path(path)) => Ok(Self::Repr(path)),
            syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                if name_value.path.is_ident(Self::VALUE) {
                    Ok(Self::Value(name_value.lit))
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
