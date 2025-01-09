#![allow(unused)]
use darling::{util::Flag, FromMeta, Result};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_str, Expr, Lit, MetaNameValue};

#[derive(Debug, FromMeta)]
pub struct Input {
    db: FromString<syn::Type>,
    migrate: Flag,
}

#[derive(Debug)]
pub struct FromString<T>(T);

impl<T: Parse> FromMeta for FromString<T> {
    fn from_meta(item: &syn::Meta) -> Result<Self> {
        if let syn::Meta::NameValue(MetaNameValue {
            path: _,
            eq_token: _,
            value:
                Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(expr),
                    attrs: _,
                }),
        }) = item
        {
            let str = expr.into_token_stream().to_string();

            match parse_str::<T>(&str[1..str.len() - 1]) {
                Ok(t) => return Ok(Self(t)),
                Err(err) => {
                    return Err(darling::Error::custom(format!(
                        "failed to parse: {}",
                        err
                    ))
                    .with_span(expr))
                }
            }
        } else {
            Err(<()>::from_meta(item).unwrap_err())
        }
    }
}

pub fn main(input: Input) -> TokenStream {
    let mut ts = quote!();

    let db = input.db.0;
    ts.extend(quote!(
        #[allow(non_camel_case_types)]
        type CMS_DB = #db;

        #[allow(dead_code)]
        fn _no_aliase() {
            let _: ::std::option::Option<()> = Some(());
            let _: ::std::option::Option<()> = None;
            let _: ::std::option::Option<()> = {
                let r: Option<()> = None;
                r
            };
        }
    ));

    ts
}
