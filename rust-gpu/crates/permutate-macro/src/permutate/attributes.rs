use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, Attribute, Ident};

// Split a list of `Attribute`s out from a [`Parse`] type
pub struct Attributes<T> {
    attrs: Vec<Attribute>,
    inner: T,
}

impl<T> Parse for Attributes<T>
where
    T: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Attributes {
            attrs: Attribute::parse_outer(input)?,
            inner: input.parse()?,
        })
    }
}

impl<T> ToTokens for Attributes<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in self.attrs.iter() {
            attr.to_tokens(tokens);
        }
        self.inner.to_tokens(tokens);
    }
}

impl<T> Attributes<T>
where
    T: Parse + ToTokens,
{
    pub fn position(&self, ident: &Ident) -> Option<usize> {
        self.attrs.iter().position(|attr| {
            let Some(candidate) = attr.path.get_ident() else {
        return false;
    };

            candidate == ident
        })
    }

    pub fn remove(&mut self, ident: &Ident) -> Option<Attribute> {
        self.position(ident).map(|i| self.attrs.remove(i))
    }
}

