#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Lit, Pat, Token,
};

extern crate proc_macro;

type Cst<T> = Punctuated<T, Token![,]>;

enum SetBuilderInput {
    Enum {
        literals: Cst<Lit>,
    },
    Full {
        map: Expr,
        set_mappings: Cst<SetMapping>,
        predicate: Option<Expr>,
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

mod punc {
    use syn::custom_punctuation;

    custom_punctuation!(In, <-);
    custom_punctuation!(SuchThat, :);
}

impl Parse for SetBuilderInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if input.is_empty() || lookahead.peek(Lit) {
            let literals = input.parse_terminated(Lit::parse, Token![,])?;

            Ok(Self::Enum { literals })
        } else if let Ok(map) = input.parse::<Expr>() {
            if input.parse::<punc::SuchThat>().is_err() {
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

            if !input.is_empty() {
                if let Ok(pred) = input.parse::<Expr>() {
                    predicate = Some(pred);
                } else {
                    panic!(
                        "invalid predicate `{}`, predicates should evaluate to a `bool`",
                        input
                    );
                }
            }

            Ok(Self::Full {
                map,
                set_mappings,
                predicate,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[doc = include_str!("../README.md")]
#[proc_macro]
pub fn set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    match input {
        SetBuilderInput::Enum { literals } => quote! {
            [ #literals ]
        },
        SetBuilderInput::Full {
            map,
            set_mappings,
            predicate,
        } => {
            let mut iter = set_mappings.iter().enumerate().peekable();
            let mut names: Cst<Pat> = Punctuated::new();
            let mut acc = quote!();

            if let Some((_, first)) = iter.next() {
                acc = quote! {
                    #first
                };
            }

            if let Some((idx, second)) = iter.next() {
                let name = set_mappings[idx - 1].name.clone();
                names.push_value(name.clone());
                names.push_punct(syn::token::Comma::default());

                acc = quote! {
                    #acc.flat_map(|#name| {
                        ::core::iter::repeat(#name).zip(#second)
                    })
                };
            }

            for (idx, mapping) in iter {
                let name = set_mappings[idx - 1].name.clone();
                names.push_value(name.clone());
                names.push_punct(syn::token::Comma::default());

                acc = quote! {
                    #acc.flat_map(|#name| {
                        ::core::iter::repeat(#name).zip(#mapping).map(|out| (out.0.0, out.0.1, out.1))
                    })
                };
            }

            if let Some(m) = set_mappings.last() {
                names.push_value(m.name.clone())
            }

            let tuple = quote! {
                (#names)
            };

            if let Some(predicate) = predicate {
                acc = quote! {
                    #acc.filter(|#tuple| #predicate)
                };
            }

            quote! {
                {
                    #[allow(unused_variables)]
                    #acc.map(|#tuple| #map)
                }
            }
        }
    }
    .into()
}
