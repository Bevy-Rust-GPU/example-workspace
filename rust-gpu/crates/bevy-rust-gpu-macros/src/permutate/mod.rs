pub mod attributes;
pub mod keywords;
pub mod parameter_conditional;
pub mod parameters;
pub mod permutate_attribute;
pub mod permutations;

use proc_macro::Span;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use syn::{parse::Parse, parse_quote, Error, Ident, ItemFn, Stmt};

use self::{
    attributes::Attributes, parameter_conditional::ParameterConditional, parameters::Parameters,
};

pub fn macro_impl(
    attr: permutate_attribute::PermutateAttribute,
    item_fn: ItemFn,
) -> Result<TokenStream, Error> {
    let parameters = attr.parameters()?;

    eprintln!("Parameters");

    // Calculate permutations
    let permutations = attr
        .permutations()?
        .into_permutations(&item_fn.sig.ident, parameters);

    eprintln!("Permutations");

    // Iterate permutations
    let mut out = vec![];

    for permutation in permutations.into_iter() {
        // Create a new copy of the function
        let mut item_fn = item_fn.clone();

        // Generate name from permutation
        let ident = item_fn.sig.ident.to_string()
            + &permutation
                .iter()
                .map(ToString::to_string)
                .map(|t| "__".to_string() + &t)
                .collect::<String>();

        let ident = Ident::new(&ident, item_fn.sig.ident.span());
        item_fn.sig.ident = ident;

        // Macro ident for matching inner attributes
        let span = Span::call_site().into();
        let ident = Ident::new("permutate", span);

        // Process function inputs
        let mut inputs = vec![];

        for item in item_fn.sig.inputs.into_iter() {
            if let Some(item) = apply_parameter_conditional(parameters, &permutation, &ident, item)?
            {
                inputs.push(item);
            }
        }

        item_fn.sig.inputs = inputs.into_iter().collect();

        // Process block statements
        let mut stmts = vec![];

        for stmt in item_fn.block.stmts.into_iter() {
            if let Some(item) = apply_parameter_conditional(parameters, &permutation, &ident, stmt)?
            {
                stmts.push(item);
            }
        }

        item_fn.block.stmts = stmts.into_iter().collect();

        // Process call expression arguments
        for mut stmt in item_fn.block.stmts.iter_mut() {
            match &mut stmt {
                Stmt::Expr(expr) | Stmt::Semi(expr, _) => match expr {
                    syn::Expr::Call(call) => {
                        let mut args = vec![];

                        for arg in call.args.iter() {
                            if let Some(arg) = apply_parameter_conditional(
                                parameters,
                                &permutation,
                                &ident,
                                arg.clone(),
                            )? {
                                args.push(arg);
                            }
                        }

                        call.args = args.into_iter().collect();
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        out.push(item_fn);
    }

    Ok(quote! {
        #(#out)*
    })
}

/// Returns a filter function that will parse the given attribute,
/// extract a parameter conditional from its arguments,
/// and return None if the conditional evaluates false
fn apply_parameter_conditional<'a, T>(
    parameters: &'a Parameters,
    permutation: &'a Vec<Ident>,
    attr_ident: &'a Ident,
    mut input: T,
) -> Result<Option<T>, Error>
where
    T: Parse + ToTokens,
{
    if let Ok(mut attrs) = syn::parse::<Attributes<T>>(quote!(#input).into()) {
        if let Some(attr) = attrs.remove(&attr_ident) {
            let parameter_conditional =
                attr.parse_args_with(ParameterConditional::parse(parameters))?;

            let idx = parameters
                .parameters
                .iter()
                .position(|parameter| parameter.ident == parameter_conditional.parameter)
                .unwrap();

            if permutation[idx] != parameter_conditional.variant {
                return Ok(None);
            };
        }

        input = parse_quote!(#attrs);
    }

    Ok(Some(input))
}
