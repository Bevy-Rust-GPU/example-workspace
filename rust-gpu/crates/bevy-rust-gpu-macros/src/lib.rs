extern crate proc_macro;

use proc_macro::Span;
use quote::{quote, ToTokens, TokenStreamExt};
use std::collections::BTreeMap;

use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseBuffer},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::{Comma, Or},
    Attribute, Expr, FieldValue, FnArg, GenericArgument, Ident, ItemFn, Member, PathArguments,
    PathSegment, Stmt, Token,
};

#[derive(Debug)]
struct MappingVariants {
    field: Ident,
    variants: Vec<Ident>,
}

impl Parse for MappingVariants {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        eprintln!("MappingVariants::parse");
        let field: Ident = input.parse()?;
        eprintln!("Field: {field:?}");
        let _colon: Token![:] = input.parse()?;
        eprintln!("Colon");

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

        eprintln!("Variants");
        Ok(MappingVariants { field, variants })
    }
}

#[derive(Debug)]
struct Mappings {
    mappings: Vec<(Ident, Vec<Ident>)>,
}

impl Parse for Mappings {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        eprintln!("Parsing mappings");

        let _ident: Ident = input.parse()?;
        eprintln!("Mappings Ident");

        let _eq: Token![=] = input.parse()?;
        eprintln!("Mappings Eq");

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
    permutations: Vec<PermutationTuple>,
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
struct PermutationTuple {
    tuple: Vec<Ident>,
}

impl Parse for PermutationTuple {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inner: ParseBuffer;
        let _paren = parenthesized!(inner in input);

        let tuple: Punctuated<_, Token![,]> = inner.parse_terminated(Ident::parse)?;
        let tuple: Vec<_> = tuple.into_iter().collect();

        Ok(PermutationTuple { tuple })
    }
}

impl Parse for Permutations {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        eprintln!("Parsing Permutations");

        let _ident: Ident = input.parse()?;
        eprintln!("Permutations Ident");

        let _eq: Token![=] = input.parse()?;
        eprintln!("Permutations Eq");

        let content: ParseBuffer;
        let _bracket = bracketed!(content in input);

        let permutations: Punctuated<PermutationTuple, Token![,]> =
            content.parse_terminated(PermutationTuple::parse)?;
        let permutations: Vec<_> = permutations.into_iter().collect();

        Ok(Permutations { permutations })
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
        permutations,
    } = parse_macro_input!(attr as MacroInput);

    eprintln!("Mappings: {mappings:#?}");
    eprintln!("Permutations: {permutations:#?}");

    // Parse function
    let item_fn = parse_macro_input!(item as ItemFn);

    let mut out = vec![];
    for permutation in permutations.permutations {
        // Create a new copy of the function
        let mut item_fn = item_fn.clone();

        // Set name based on permutation
        let ident = item_fn.sig.ident.to_string()
            + &permutation
                .tuple
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
                    eprintln!("Fn Sig Attr {:#?}", attr.path.get_ident());
                    let mapping_conditional =
                        attr.parse_args_with(MappingConditional::parse).unwrap();
                    eprintln!("Sn Sig Expr {:#?}", mapping_conditional);

                    let idx = mappings
                        .mappings
                        .iter()
                        .position(|(name, _)| *name == mapping_conditional.lhs)
                        .unwrap();

                    return if permutation.tuple[idx] == mapping_conditional.rhs {
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
        eprintln!("Parsing statements");
        item_fn.block.stmts = item_fn
            .block
            .stmts
            .into_iter()
            .filter_map(|mut stmt| {
                eprintln!("Fn Block Stmt");
                let s = proc_macro::TokenStream::from(quote!(#stmt));
                eprintln!("{s:}");

                if let Ok(mut attrs) = syn::parse::<Attributes<Stmt>>(s) {
                    if let Some(attr) = attrs.remove(&ident) {
                        eprintln!("Fn Block Attr {:#?}", attr.path.get_ident());
                        let mapping_conditional =
                            attr.parse_args_with(MappingConditional::parse).unwrap();
                        eprintln!("Fn Block Expr {:#?}", mapping_conditional);

                        let idx = mappings
                            .mappings
                            .iter()
                            .position(|(name, _)| *name == mapping_conditional.lhs)
                            .unwrap();

                        if permutation.tuple[idx] != mapping_conditional.rhs {
                            return None;
                        };
                    }

                    stmt = parse_quote!(#attrs);
                };

                eprintln!("Fn Block Inner");

                match &mut stmt {
                    Stmt::Expr(expr) | Stmt::Semi(expr, _) => match expr {
                        syn::Expr::Call(call) => {
                            eprintln!("Fn Block Call Expr");

                            call.args = call.args.iter().cloned().filter_map(|arg| {
                                eprintln!("Fn Block Call Arg");
                                let a = quote!(#arg);
                                let Ok(mut attrs) = syn::parse::<Attributes<Expr>>(a.into()) else {
                                    return Some(arg);
                                };

                                eprintln!("Fn Blok Call Arg Attrs");

                                if let Some(attr) = attrs.remove(&ident) {
                                    eprintln!("Fn Block Call Arg Attr {:#?}", attr.path.get_ident());
                                    let mapping_conditional =
                                        attr.parse_args_with(MappingConditional::parse).unwrap();

                                    eprintln!("Fn Block Call Arg Expr {:#?}", mapping_conditional);

                                    let idx = mappings
                                        .mappings
                                        .iter()
                                        .position(|(name, _)| *name == mapping_conditional.lhs)
                                        .unwrap();

                                    return if permutation.tuple[idx] == mapping_conditional.rhs {
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

    eprintln!("Output: {output:}");

    output.into()
}
