use proc_macro::Span;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Error,
};

use super::{keywords, Parameters, permutations::Permutations};

pub enum PermutateArgument {
    Parameters(Parameters),
    Permutations(Permutations),
}

impl Parse for PermutateArgument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(keywords::parameters) {
            Ok(PermutateArgument::Parameters(Parameters::parse(input)?))
        } else if lookahead.peek(keywords::permutations) {
            Ok(PermutateArgument::Permutations(Permutations::parse(input)?))
        } else {
            Err(Error::new(
                input.span(),
                "Invalid parameter. Valid parameters are [parameters, permutations].",
            ))
        }
    }
}

pub struct PermutateAttribute {
    parameters: Punctuated<PermutateArgument, Comma>,
}

impl Parse for PermutateAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = PermutateAttribute {
            parameters: Punctuated::parse_separated_nonempty(input)?,
        };

        attr.permutations()?.validate(attr.parameters()?)?;

        Ok(attr)
    }
}

impl PermutateAttribute {
    pub fn parameters(&self) -> Result<&Parameters, Error> {
        self.parameters
            .iter()
            .find_map(|param| {
                if let PermutateArgument::Parameters(parameters) = param {
                    Some(parameters)
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::new(Span::call_site().into(), "Missing parameters"))
    }

    pub fn permutations(&self) -> Result<&Permutations, Error> {
        self.parameters
            .iter()
            .find_map(|param| {
                if let PermutateArgument::Permutations(permutations) = param {
                    Some(permutations)
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::new(Span::call_site().into(), "Missing permutations"))
    }
}

