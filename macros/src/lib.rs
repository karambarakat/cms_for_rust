use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
mod collection_derive;
mod entity_derive;
mod into_mut_argument_derive;
mod relation;
mod relations_mod;
mod schema_macro;

// #[proc_macro_derive(IntoMutArguments)]
// pub fn into_mut_arguments_derive(
//     input: TokenStream,
// ) -> TokenStream {
//     let parsed = parse_macro_input!(input as syn::DeriveInput);
//
//     let mut ts = Default::default();
//
//     into_mut_argument_derive::consume_into_argument_impl(&parsed, &mut ts);
//
//     ts.into()
// }

#[proc_macro_derive(Collection)]
#[proc_macro_error]
pub fn collection(input: TokenStream) -> TokenStream {
    let derive = match syn::parse::<syn::DeriveInput>(input) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    collection_derive::main(derive).into()
}

#[proc_macro_attribute]
pub fn standard_collection(
    _: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let struct_ =
        syn::parse_macro_input!(item as syn::ItemStruct);

    quote::quote!(
        #[derive(
    ::cms_for_rust::cms_macros::Collection,
    ::std::fmt::Debug,
    ::std::clone::Clone,
    ::std::cmp::PartialEq,
    ::std::cmp::Eq,
    ::cms_for_rust::macro_prelude::serde::Deserialize,
    ::cms_for_rust::macro_prelude::serde::Serialize,
        )]
        #struct_
    )
    .into()
}

// #[proc_macro]
// pub fn schema(input: TokenStream) -> TokenStream {
//     use darling::{ast::NestedMeta, Error, FromMeta};
//     use schema_macro::*;
//
//     let input = match NestedMeta::parse_meta_list(input.into()) {
//         Ok(v) => v,
//         Err(err) => {
//             return Error::from(err).write_errors().into()
//         }
//     };
//
//     let input: Input = match Input::from_list(&input) {
//         Ok(v) => v,
//         Err(err) => {
//             return TokenStream::from(err.write_errors())
//         }
//     };
//
//     main(input).into()
// }
//

#[proc_macro]
#[proc_macro_error]
pub fn relation(input: TokenStream) -> TokenStream {
    let input = match syn::parse::<relation::Input>(input) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    relation::main(input).into()
}
