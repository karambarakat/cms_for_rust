use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
mod entity_derive;
mod relations_mod;
mod schema_macro;

#[proc_macro_derive(Entity, attributes(service))]
#[proc_macro_error]
pub fn entity_derive(input: TokenStream) -> TokenStream {
    use entity_derive::*;

    let derive = match syn::parse::<syn::DeriveInput>(input) {
        Ok(data) => data,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    main(&derive).into()
}

#[proc_macro_attribute]
pub fn service(
    _: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let struct_ =
        syn::parse_macro_input!(item as syn::ItemStruct);

    quote::quote!(
        #[derive(
          ::cms_for_rust::entities::derive_prelude::Debug,
            ::cms_for_rust::entities::derive_prelude::Clone,
            ::cms_for_rust::entities::derive_prelude::PartialEq,
            ::cms_for_rust::entities::derive_prelude::Eq,
            ::cms_for_rust::entities::derive_prelude::Serialize,
            ::cms_for_rust::entities::derive_prelude::Deserialize,
            ::cms_for_rust::entities::derive_prelude::FromRow,
            ::cms_for_rust::entities::derive_prelude::Entity,
            ::cms_for_rust::entities::derive_prelude::IntoMutArguments,
        )]
        #struct_
    )
    .into()
}

#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    use darling::{ast::NestedMeta, Error, FromMeta};
    use schema_macro::*;

    let input = match NestedMeta::parse_meta_list(input.into()) {
        Ok(v) => v,
        Err(err) => {
            return Error::from(err).write_errors().into()
        }
    };

    let input: Input = match Input::from_list(&input) {
        Ok(v) => v,
        Err(err) => {
            return TokenStream::from(err.write_errors())
        }
    };

    main(input).into()
}

#[proc_macro]
#[proc_macro_error]
pub fn relations(input: TokenStream) -> TokenStream {
    match syn::parse::<relations_mod::Inputs>(input) {
        Ok(v) => relations_mod::main(v.inputs).into(),
        Err(err) => err.to_compile_error().into(),
    }
}
