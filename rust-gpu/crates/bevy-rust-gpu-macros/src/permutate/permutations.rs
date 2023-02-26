use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    token::{Bracket, Comma, Or, Paren, Star},
    Error, Ident,
};

use super::parameters::Parameters;

pub struct Permutation {
    _paren: Paren,
    variants: Punctuated<PermutationVariant, Comma>,
}

impl Parse for Permutation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content: ParseBuffer;
        Ok(Permutation {
            _paren: parenthesized!(content in input),
            variants: Punctuated::parse_separated_nonempty(&content)?,
        })
    }
}

impl Permutation {
    /// Ensure that all variants are defined in the provided [`Parameters`]
    fn validate(&self, parameters: &Parameters) -> Result<(), Error> {
        for (i, variant) in self.variants.iter().enumerate() {
            variant.validate(parameters, i)?;
        }

        Ok(())
    }

    fn into_permutations(&self, parameters: &Parameters) -> Vec<Vec<Ident>> {
        let mut idents = vec![];
        for (i, variant) in self.variants.iter().enumerate() {
            let mut ids = vec![];
            match variant {
                PermutationVariant::Explicit(explicit) => ids.extend(explicit.iter().cloned()),
                PermutationVariant::Glob(_) => {
                    ids.extend(parameters.parameters[i].variants.clone())
                }
            }
            idents.push(ids);
        }

        permutations(idents)
    }
}

pub struct Permutations {
    _ident: Ident,
    _eq: syn::token::Eq,
    _bracket: Bracket,
    permutations: Punctuated<Permutation, Comma>,
}

impl Parse for Permutations {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content: ParseBuffer;
        Ok(Permutations {
            _ident: input.parse()?,
            _eq: input.parse()?,
            _bracket: bracketed!(content in input),
            permutations: Punctuated::parse_separated_nonempty(&content)?,
        })
    }
}

impl Permutations {
    /// Ensure that all variants are defined in the provided [`Parameters`]
    pub fn validate(&self, parameters: &Parameters) -> Result<(), Error> {
        for permutation in self.permutations.iter() {
            permutation.validate(parameters)?;
        }

        Ok(())
    }

    pub fn into_permutations(&self, parameters: &Parameters) -> Vec<Vec<Ident>> {
        let mut permutations = vec![];
        for permutation in self.permutations.iter() {
            permutations.extend(permutation.into_permutations(parameters))
        }
        permutations.sort();
        permutations.dedup();
        permutations
    }
}

enum PermutationVariant {
    Explicit(Punctuated<Ident, Or>),
    Glob(Star),
}

impl Parse for PermutationVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Star) {
            Ok(PermutationVariant::Glob(Star::parse(input)?))
        } else {
            Ok(PermutationVariant::Explicit(
                Punctuated::<Ident, Or>::parse_separated_nonempty(input)?,
            ))
        }
    }
}

impl PermutationVariant {
    fn validate(&self, parameters: &Parameters, index: usize) -> Result<(), Error> {
        match self {
            PermutationVariant::Explicit(idents) => {
                for ident in idents {
                    let parameter = &parameters.parameters[index];
                    if !parameter
                        .variants
                        .iter()
                        .find(|candidate| *candidate == ident)
                        .is_some()
                    {
                        let valid_variants = parameter
                            .variants
                            .iter()
                            .map(ToString::to_string)
                            .enumerate()
                            .map(|(i, variant)| {
                                variant
                                    + if i < parameter.variants.len() - 1 {
                                        ", "
                                    } else {
                                        ""
                                    }
                            })
                            .collect::<String>();

                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "Invalid variant {ident:}. Valid variants are [{valid_variants:}].",
                            ),
                        ));
                    }
                }
            }
            _ => (),
        }

        Ok(())
    }
}

/// Convert a list of lists of variants into a list of those variants' permutations
fn permutations<T: Clone>(sets: Vec<Vec<T>>) -> Vec<Vec<T>> {
    permutations_inner(sets.into_iter(), Default::default())
}

fn permutations_inner<It: Clone + Iterator<Item = Vec<T>>, T: Clone>(
    mut sets: It,
    acc: Vec<T>,
) -> Vec<Vec<T>> {
    sets.next()
        .map(|set| {
            set.into_iter()
                .flat_map(|item| {
                    let mut tmp = acc.clone();
                    tmp.push(item);
                    permutations_inner(sets.clone(), tmp)
                })
                .collect()
        })
        .unwrap_or_else(|| vec![acc])
}
