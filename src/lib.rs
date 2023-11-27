#![doc = include_str!("../README.md")]

use std::iter::{Enumerate, Peekable};

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse_macro_input,
    punctuated::{Iter, Punctuated},
    Expr, Lit, Pat, Token,
};

extern crate proc_macro;

type Cst<T> = Punctuated<T, Token![,]>;

#[derive(Clone)]
enum Auxiliary {
    SetMapping(SetMapping),
    Predicate(Expr),
}

#[allow(dead_code)]
impl Auxiliary {
    pub fn is_set_mapping(&self) -> bool {
        match self {
            Auxiliary::SetMapping(_) => true,
            Auxiliary::Predicate(_) => false,
        }
    }

    pub fn is_predicate(&self) -> bool {
        match self {
            Auxiliary::SetMapping(_) => false,
            Auxiliary::Predicate(_) => true,
        }
    }

    pub fn into_set_mapping(self) -> SetMapping {
        match self {
            Auxiliary::SetMapping(sm) => sm,
            Auxiliary::Predicate(_) => panic!("Tried to interpret a Predicate as a SetMapping."),
        }
    }

    pub fn into_predicate(self) -> Expr {
        match self {
            Auxiliary::Predicate(pred) => pred,
            Auxiliary::SetMapping(_) => panic!("Tried to interpret a SetMapping as a Predicate."),
        }
    }
}

impl ToTokens for Auxiliary {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Auxiliary::SetMapping(sm) => sm.to_tokens(tokens),
            Auxiliary::Predicate(pred) => pred.to_tokens(tokens),
        }
    }
}

enum SetBuilderInput {
    Enum {
        exprs: Cst<Expr>,
    },
    Full {
        map: Expr,
        auxiliary: Cst<Auxiliary>,
    },
}

#[derive(Clone)]
struct SetMapping {
    name: Pat,
    set: Expr,
}

impl ToTokens for SetMapping {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { set, .. } = self;
        *tokens = quote! { (#set).into_iter() };
    }
}

impl Parse for SetMapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let begin = input.fork();
        let name = Pat::parse_single(&begin)?;
        begin.parse::<punc::In>()?;
        let set = begin.parse::<Expr>()?;
        input.advance_to(&begin);

        Ok(Self { name, set })
    }
}

impl Parse for Auxiliary {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(set_mapping) = SetMapping::parse(input) {
            return Ok(Auxiliary::SetMapping(set_mapping));
        }

        Ok(Auxiliary::Predicate(Expr::parse(input)?))
    }
}

mod punc {
    use syn::custom_punctuation;

    custom_punctuation!(In, <-);
    custom_punctuation!(SuchThat, :);
}

impl Parse for SetBuilderInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if input.is_empty() || lookahead.peek(Lit) {
            let exprs = input.parse_terminated(Expr::parse, Token![,])?;

            Ok(Self::Enum { exprs })
        } else if let Ok(map) = input.parse::<Expr>() {
            if input.parse::<punc::SuchThat>().is_err() {
                abort!(input.span(), "expected `:` after bindings, if you were trying to create an array, use `[...]` instead");
            }

            let mut auxiliary: Cst<Auxiliary> = Punctuated::new();

            while !input.is_empty() {
                if let Ok(aux) = input.parse::<Auxiliary>() {
                    auxiliary.push_value(aux);
                    if let Some(p) = input.parse()? {
                        auxiliary.push_punct(p);
                    }
                } else {
                    break;
                }
            }

            Ok(Self::Full { map, auxiliary })
        } else {
            Err(lookahead.error())
        }
    }
}

#[doc = include_str!("../README.md")]
#[proc_macro_error]
#[proc_macro]
pub fn set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    match input {
        SetBuilderInput::Enum { exprs } => quote! {
            [ #exprs ]
        },
        SetBuilderInput::Full { map, auxiliary } => {
            let mut iter = auxiliary.iter().enumerate().peekable();
            let mut names: Cst<Pat> = Punctuated::new();
            let mut acc = quote!();

            let consume_predicates = |iter: &mut Peekable<Enumerate<Iter<'_, Auxiliary>>>, names: &Cst<Pat>, acc: &mut proc_macro2::TokenStream| {
                while iter.peek().map(|(_, aux)| aux.is_predicate()).unwrap_or_default() {
                    let (_, predicate) = iter.next().unwrap();
                    match names.len() {
                        0 => {
                            *acc = quote! {
                                #acc.filter(|_| #predicate)
                            };
                        },
                        1 => {
                            let name = &names[0];
                            *acc = quote! {
                                #acc.filter(|#name| #predicate)
                            };
                        },
                        _ => {
                            let tuple = quote! {
                                (#names)
                            };
                            *acc = quote! {
                                #acc.filter(|#tuple| #predicate)
                            };
                        }
                    }
                }
            };

            consume_predicates(&mut iter, &names, &mut acc);

            if let Some((_, first)) = iter.next() {
                names.push_value(first.clone().into_set_mapping().name.clone());
                names.push_punct(syn::token::Comma::default());
                acc = quote! {
                    #first
                };
            }

            consume_predicates(&mut iter, &names, &mut acc);

            if let Some((_, second)) = iter.next() {
                let name = names.last().unwrap();

                acc = quote! {
                    #acc.flat_map(|#name| {
                        ::core::iter::repeat(#name).zip(#second)
                    })
                };
                names.push_value(second.clone().into_set_mapping().name);
                names.push_punct(syn::token::Comma::default());
            }

            for (_, aux) in iter {
                let tuple = quote! {
                    (#names)
                };
                let name = names.last().unwrap();
                match aux {
                    Auxiliary::SetMapping(sm) => {
                        acc = quote! {
                            #acc.flat_map(|#name| {
                                ::core::iter::repeat(#name).zip(#aux).map(|(#tuple, new)| (#names new))
                            })
                        };

                        names.push_value(sm.name.clone());
                        names.push_punct(syn::token::Comma::default());
                    },
                    Auxiliary::Predicate(pred) => {
                        acc = quote! {
                            #acc.filter(|#tuple| #pred)
                        };
                    },
                }

            }

            match names.len() {
                0 => {
                    quote! {
                        quote! {
                            {
                                #[allow(unused_variables)]
                                #acc.map(|_| #map)
                            }
                        }
                    }
                },
                1 => {
                    let name = &names[0];
                    quote! {
                        {
                            #[allow(unused_variables)]
                            #acc.map(|#name| #map)
                        }
                    }
                },
                _ => {
                    let tuple = quote! {
                        (#names)
                    };
                    quote! {
                        {
                            #[allow(unused_variables)]
                            #acc.map(|#tuple| #map)
                        }
                    }
                }
            }
        }
    }
    .into()
}
