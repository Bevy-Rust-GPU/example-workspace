use syn::{
    braced,
    parse::{Parse, ParseBuffer},
    punctuated::Punctuated,
    token::{Brace, Colon, Comma, Or},
    Error, Ident,
};

pub struct Parameter {
    pub ident: Ident,
    pub colon: Colon,
    pub variants: Punctuated<Ident, Or>,
}

impl Parse for Parameter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Parameter {
            ident: input.parse()?,
            colon: input.parse()?,
            variants: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

pub struct Parameters {
    pub ident: Ident,
    pub eq: syn::token::Eq,
    pub brace: Brace,
    pub parameters: Punctuated<Parameter, Comma>,
}

impl Parse for Parameters {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content: ParseBuffer;
        let parameters = Parameters {
            ident: input.parse()?,
            eq: input.parse()?,
            brace: braced!(content in input),
            parameters: Punctuated::parse_separated_nonempty(&content)?,
        };

        if !content.is_empty() {
            return Err(Error::new(content.span(), "unexpected token"));
        }

        Ok(parameters)
    }
}
