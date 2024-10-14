#![allow(unused)]
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod erroneos;
mod error;
mod invariant_macro;
#[cfg(test)]
mod tests;
// mod tuple_trait;

#[proc_macro]
pub fn invariant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemTrait);
    invariant_macro::main(input).into()
}

// #[proc_macro_attribute]
// pub fn tuple_trait(_: TokenStream, input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as syn::ItemTrait);
//     tuple_trait::main(input).into()
// }
