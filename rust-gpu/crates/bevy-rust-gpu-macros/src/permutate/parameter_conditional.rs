use syn::{parse::ParseStream, Error, Ident};

use super::parameters::Parameters;

pub struct ParameterConditional {
    pub parameter: Ident,
    pub eq: syn::token::Eq,
    pub variant: Ident,
}

impl ParameterConditional {
    pub fn parse(parameters: &Parameters) -> impl Fn(ParseStream) -> syn::Result<Self> + '_ {
        move |input: ParseStream| {
            let conditional = ParameterConditional {
                parameter: input.parse()?,
                eq: input.parse()?,
                variant: input.parse()?,
            };

            conditional.validate(parameters)?;

            Ok(conditional)
        }
    }

    pub fn validate(&self, parameters: &Parameters) -> Result<(), Error> {
        let parameter = parameters
            .parameters
            .iter()
            .find(|candidate| candidate.ident == self.parameter)
            .ok_or_else(|| {
                Error::new(
                    self.parameter.span(),
                    format!(
                        "Invalid parameter {}. Valid parameters are [{}].",
                        self.parameter,
                        parameters
                            .parameters
                            .iter()
                            .enumerate()
                            .map(|(i, parameter)| parameter.ident.to_string()
                                + if i < parameters.parameters.len() - 1 {
                                    ", "
                                } else {
                                    ""
                                })
                            .collect::<String>(),
                    ),
                )
            })?;

        if parameter
            .variants
            .iter()
            .find(|candidate| **candidate == self.variant)
            .is_none()
        {
            return Err(Error::new(
                self.variant.span(),
                format!(
                    "Invalid variant {}. Valid variants are [{}].",
                    self.variant,
                    parameter
                        .variants
                        .iter()
                        .enumerate()
                        .map(|(i, ident)| ident.to_string()
                            + if i < parameter.variants.len() - 1 {
                                ", "
                            } else {
                                ""
                            })
                        .collect::<String>()
                ),
            ));
        }

        Ok(())
    }
}
