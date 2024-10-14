#![allow(unused)]
use proc_macro::TokenStream;
use syn::{parse, parse_macro_input};

mod db_generic;
mod db_generic_rules;
mod into_query;

#[proc_macro_attribute]
pub fn db_generic(
    input: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let input = parse_macro_input!(input as db_generic::Input);

    db_generic::main(item.into(), input).into()
}

#[proc_macro_derive(IntoMutArguments)]
pub fn into_mut_arguments_derive(
    input: TokenStream,
) -> TokenStream {
    let parsed = parse_macro_input!(input as syn::DeriveInput);

    let mut ts = Default::default();

    into_query::consume_into_argument_impl(&parsed, &mut ts);

    ts.into()
}
