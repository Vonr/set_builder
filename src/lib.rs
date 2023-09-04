//! # set_builder
//!
#![doc = include_str!("./DOCS.md")]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, Lit, Token,
};

extern crate proc_macro;

type Cst<T> = Punctuated<T, Token![,]>;

enum Bindings {
    One(Ident),
    Many(Cst<Ident>),
}

impl ToTokens for Bindings {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        *tokens = match self {
            Self::One(one) => quote! {
                |#one|
            },
            Self::Many(many) => quote! {
                |(#many)|
            },
        }
    }
}

enum SetBuilderInput {
    Enum {
        literals: Cst<Lit>,
    },
    Full {
        bindings: Bindings,
        set_mappings: Cst<SetMapping>,
        predicate: Option<Expr>,
    },
}

#[derive(Clone)]
struct SetMapping {
    name: Ident,
    set: Expr,
}

impl ToTokens for SetMapping {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { set, .. } = self;
        *tokens = quote! { (#set).into_iter() };
    }
}

mod punc {
    use syn::custom_punctuation;

    custom_punctuation!(In, <-);
}

impl Parse for SetMapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<punc::In>()?;
        let set = input.parse::<Expr>()?;

        Ok(Self { name, set })
    }
}

impl Parse for SetBuilderInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if input.is_empty() || lookahead.peek(Lit) {
            let literals = input.parse_terminated(Lit::parse, Token![,])?;

            Ok(Self::Enum { literals })
        } else if lookahead.peek(Ident) || lookahead.peek(syn::token::Paren) {
            let bindings;

            if input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                bindings = Bindings::Many(content.parse_terminated(Ident::parse, Token![,])?);
            } else {
                bindings = Bindings::One(input.parse::<Ident>()?);
            }

            if input.parse::<Token![:]>().is_err() {
                panic!("expected `:` after bindings, if you were trying to create an array, use `[...]` instead");
            }

            let mut set_mappings: Cst<SetMapping> = Punctuated::new();
            let mut predicate = None;

            while !input.is_empty() {
                if let Ok(mapping) = input.parse::<SetMapping>() {
                    set_mappings.push_value(mapping);
                    if let Some(p) = input.parse()? {
                        set_mappings.push_punct(p);
                    }
                } else {
                    break;
                }
            }

            match bindings {
                Bindings::Many(ref bindings) => {
                    for mapping in &set_mappings {
                        if !bindings.iter().any(|binding| *binding == mapping.name) {
                            panic!("binding to {} is unused", mapping.name);
                        }
                    }

                    for binding in bindings {
                        if !set_mappings.iter().any(|mapping| *binding == mapping.name) {
                            panic!("{} is not bound to any sets", binding);
                        }
                    }
                }
                Bindings::One(ref binding) => {
                    if !set_mappings.iter().any(|mapping| *binding == mapping.name) {
                        panic!("{} is not bound to any sets", binding);
                    }

                    for mapping in &set_mappings {
                        if mapping.name != *binding {
                            panic!("binding to {} is unused", mapping.name);
                        }
                    }
                }
            }

            if !input.is_empty() {
                if let Ok(pred) = input.parse::<Expr>() {
                    predicate = Some(pred);
                } else {
                    panic!("invalid predicate, predicates should evaluate to a `bool`");
                }
            }

            Ok(Self::Full {
                bindings,
                set_mappings,
                predicate,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[doc = include_str!("./DOCS.md")]
#[proc_macro]
pub fn set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    match input {
        SetBuilderInput::Enum { literals } => quote! {
            [ #literals ]
        },
        SetBuilderInput::Full {
            bindings,
            set_mappings,
            predicate,
        } => {
            let mut iter = set_mappings.iter().enumerate().peekable();
            let mut acc = quote!();

            if let Some((_, first)) = iter.next() {
                acc = quote! {
                    #first
                };
            }

            if let Some((idx, second)) = iter.next() {
                let name = set_mappings[idx - 1].name.clone();
                acc = quote! {
                    #acc.flat_map(|#name| {
                        ::std::iter::repeat(#name).zip(#second)
                    })
                };
            }

            for (idx, mapping) in iter {
                let name = set_mappings[idx - 1].name.clone();
                acc = quote! {
                    #acc.flat_map(|#name| {
                        ::core::iter::repeat(#name).zip(#mapping).map(|out| (out.0.0, out.0.1, out.1))
                    })
                };
            }

            match predicate {
                Some(predicate) => quote! {
                    #acc.filter(#bindings #predicate)
                },
                None => quote! { #acc },
            }
        }
    }
    .into()
}