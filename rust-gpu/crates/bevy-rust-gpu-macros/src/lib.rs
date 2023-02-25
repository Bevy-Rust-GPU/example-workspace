extern crate proc_macro;

use proc_macro::Span;
use quote::{quote, ToTokens};

use syn::{
    braced, parenthesized,
    parse::{Parse, ParseBuffer},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::{Comma, Or},
    Attribute, Error, Expr, FnArg, Ident, ItemFn, Stmt, Token,
};

#[derive(Debug)]
struct MappingVariants {
    field: Ident,
    variants: Vec<Ident>,
}

impl Parse for MappingVariants {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field: Ident = input.parse()?;
        let _colon: Token![:] = input.parse()?;

        let mut variants = vec![];
        loop {
            let Ok(ident) = input.parse() else {
                break;
            };

            variants.push(ident);

            let Ok(_): Result<Or, _> = input.parse() else {
                break
            };
        }

        Ok(MappingVariants { field, variants })
    }
}

#[derive(Debug)]
struct Mappings {
    mappings: Vec<(Ident, Vec<Ident>)>,
}

impl Parse for Mappings {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _ident: Ident = input.parse()?;

        let _eq: Token![=] = input.parse()?;

        let content: ParseBuffer;
        let _brace = braced!(content in input);

        let mut mappings: Vec<(Ident, Vec<Ident>)> = Default::default();
        loop {
            let Ok(variants): Result<MappingVariants, _> = content.parse() else {
                break
            };

            mappings.push((variants.field, variants.variants));

            let Ok(_): Result<Comma, _> = content.parse() else {
                break
            };
        }

        Ok(Mappings { mappings })
    }
}

#[derive(Debug)]
struct Permutations {
    permutations: Vec<PermutationVariant>,
}

impl Parse for Permutations {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _ident: Ident = input.parse()?;

        let _eq: Token![=] = input.parse()?;

        let content: ParseBuffer;
        let _paren = parenthesized!(content in input);

        let permutations =
            Punctuated::<PermutationVariant, Token![,]>::parse_separated_nonempty(&content)?;
        let permutations: Vec<_> = permutations.into_iter().collect();

        Ok(Permutations { permutations })
    }
}

impl Permutations {
    fn to_idents(&self, mappings: &Mappings) -> Vec<Vec<Ident>> {
        let mut idents = vec![];
        for (i, permutation) in self.permutations.iter().enumerate() {
            let mut ids = vec![];
            match permutation {
                PermutationVariant::Explicit(explicit) => ids.extend(explicit.clone()),
                PermutationVariant::Glob => ids.extend(mappings.mappings[i].1.clone()),
            }
            idents.push(ids);
        }
        idents
    }
}

struct MacroInput {
    mappings: Mappings,
    permutations: Permutations,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mappings: Mappings = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let permutations: Permutations = input.parse()?;

        Ok(MacroInput {
            mappings,
            permutations,
        })
    }
}

#[derive(Debug)]
enum PermutationVariant {
    Explicit(Vec<Ident>),
    Glob,
}

impl Parse for PermutationVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(_) = <Token![*]>::parse(input) {
            Ok(PermutationVariant::Glob)
        } else if let Ok(explicit) = Punctuated::<Ident, Token![|]>::parse_separated_nonempty(input)
        {
            Ok(PermutationVariant::Explicit(explicit.into_iter().collect()))
        } else {
            Err(Error::new(input.span(), "Invalid permutation variant"))
        }
    }
}

#[derive(Debug)]
struct MappingConditional {
    lhs: Ident,
    rhs: Ident,
}

impl Parse for MappingConditional {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lhs = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let rhs = input.parse()?;

        Ok(MappingConditional { lhs, rhs })
    }
}

// Split a [`Parse`] type into a list of `Attribute`s and the type itself
struct Attributes<T> {
    attrs: Vec<Attribute>,
    inner: T,
}

impl<T> Attributes<T> {
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

impl<T> Parse for Attributes<T>
where
    T: Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let inner: T = input.parse()?;
        Ok(Attributes { attrs, inner })
    }
}

impl<T> ToTokens for Attributes<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        for attr in self.attrs.iter() {
            attr.to_tokens(tokens);
        }
        self.inner.to_tokens(tokens);
    }
}

#[proc_macro_attribute]
pub fn permutate(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse function attribute into mappings
    let MacroInput {
        mappings,
        permutations: ps,
    } = parse_macro_input!(attr as MacroInput);

    // Parse function
    let item_fn = parse_macro_input!(item as ItemFn);

    let mut out = vec![];

    let permutations = permutations(ps.to_idents(&mappings));
    for permutation in permutations {
        // Create a new copy of the function
        let mut item_fn = item_fn.clone();

        // Set name based on permutation
        let ident = item_fn.sig.ident.to_string()
            + &permutation
                .iter()
                .map(ToString::to_string)
                .map(|t| "__".to_string() + &t)
                .collect::<String>();

        let span = Span::call_site().into();
        let ident = Ident::new(&ident, span);
        item_fn.sig.ident = ident;

        // Strip attributes and parse their contents
        let span = Span::call_site().into();
        let ident = Ident::new("permutate", span);

        item_fn.sig.inputs = item_fn
            .sig
            .inputs
            .into_iter()
            .filter_map(|input| {
                let mut attrs: Attributes<FnArg> = parse_quote! {
                    #input
                };

                if let Some(attr) = attrs.remove(&ident) {
                    let mapping_conditional =
                        attr.parse_args_with(MappingConditional::parse).unwrap();

                    let idx = mappings
                        .mappings
                        .iter()
                        .position(|(name, _)| *name == mapping_conditional.lhs)
                        .unwrap();

                    return if permutation[idx] == mapping_conditional.rhs {
                        let input: FnArg = parse_quote!(#attrs);
                        Some(input)
                    } else {
                        None
                    };
                }

                Some(input)
            })
            .collect();

        // Process block statements
        item_fn.block.stmts = item_fn
            .block
            .stmts
            .into_iter()
            .filter_map(|mut stmt| {
                let s = proc_macro::TokenStream::from(quote!(#stmt));

                if let Ok(mut attrs) = syn::parse::<Attributes<Stmt>>(s) {
                    if let Some(attr) = attrs.remove(&ident) {
                        let mapping_conditional =
                            attr.parse_args_with(MappingConditional::parse).unwrap();

                        let idx = mappings
                            .mappings
                            .iter()
                            .position(|(name, _)| *name == mapping_conditional.lhs)
                            .unwrap();

                        if permutation[idx] != mapping_conditional.rhs {
                            return None;
                        };
                    }

                    stmt = parse_quote!(#attrs);
                };

                match &mut stmt {
                    Stmt::Expr(expr) | Stmt::Semi(expr, _) => match expr {
                        syn::Expr::Call(call) => {
                            call.args = call.args.iter().cloned().filter_map(|arg| {
                                let a = quote!(#arg);
                                let Ok(mut attrs) = syn::parse::<Attributes<Expr>>(a.into()) else {
                                    return Some(arg);
                                };


                                if let Some(attr) = attrs.remove(&ident) {
                                    let mapping_conditional =
                                        attr.parse_args_with(MappingConditional::parse).unwrap();


                                    let idx = mappings
                                        .mappings
                                        .iter()
                                        .position(|(name, _)| *name == mapping_conditional.lhs)
                                        .unwrap();

                                    return if permutation[idx] == mapping_conditional.rhs {
                                        let expr: Expr = parse_quote!(#attrs);
                                        Some(expr)
                                    } else {
                                        None
                                    };
                                }

                                Some(arg)
                            }).collect();
                        }
                        _ => (),
                    },
                    _ => (),
                }

                Some(stmt)
            })
            .collect();

        out.push(item_fn);
    }

    let output = quote! {
        #(#out)*
    };

    output.into()
}

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
