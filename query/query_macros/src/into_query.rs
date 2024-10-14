use std::ops::Not;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2, visit::Visit, Fields, Ident};
use syn::{
    parse_quote, DeriveInput, GenericParam, Generics, ImplItem,
    ItemImpl, Member,
};

pub fn consume_into_argument_impl(
    input: &DeriveInput,
    tokens: &mut TokenStream,
) {
    #[derive(Default)]
    pub struct I<'i> {
        pub wheres: Vec<&'i syn::Type>,
        pub members: Vec<syn::Member>,
        pub params: Vec<&'i syn::GenericParam>,
        pub wheres2: Vec<&'i syn::WherePredicate>,
        inc: u32,
        pub body: TokenStream,
    }

    impl<'ast> Visit<'ast> for I<'ast> {
        fn visit_generics(&mut self, i: &'ast syn::Generics) {
            if let Some(wc) = &i.where_clause {
                wc.predicates.iter().for_each(|i| {
                    self.wheres2.push(i);
                });
            }
            for i in i.params.iter() {
                self.params.push(i);
            }
        }
        fn visit_type(&mut self, i: &'ast syn::Type) {
            self.wheres.push(i);
        }
        fn visit_field(&mut self, i: &'ast syn::Field) {
            let inc = self.inc;
            self.inc += 1;

            let member = match i.ident.as_ref() {
                Some(ident) => Member::Named(ident.clone()),
                None => Member::Unnamed(syn::Index {
                    index: inc,
                    span: Span::call_site(),
                }),
            };
            self.members.push(member);

            syn::visit::visit_field(self, i);
        }
        fn visit_data_enum(&mut self, i: &'ast syn::DataEnum) {
            panic!("enum are not supported");
        }
        fn visit_data_union(&mut self, i: &'ast syn::DataUnion) {
            panic!("unions are not supported")
        }
    }
    let mut this = I::default();
    this.visit_derive_input(input);

    let wheres = this.wheres;
    let members = this.members;

    let params = this.params;
    let wheres2 = this.wheres2;

    let ident = &input.ident;
    let db = Ident::new("IntoMutArgumentsDB", Span::call_site());

    tokens.extend( quote!( const _: () = {
        use ::queries_for_sqlx::impl_into_mut_arguments_prelude::*;
        impl<
            'q, 
            #db,
            #(#params,)*
        >
            IntoMutArguments<'q, #db>
        for #ident
        where
            #db: Database,
            #(
                #wheres: Type<#db> + Encode<'q, #db> + Send + 'q,
            )*
            #(#wheres2,)*
        {
            // fn members_ord() -> &'static [&'static str] {
            //     &[
            //         #(
            //             stringify!(#members),
            //         )*
            //     ]
            // }
            fn into_arguments(
                self,
                argument: &mut <#db as HasArguments<'q>>::Arguments,
            ) {
                #(
                    argument.add(self.#members);
                )*
            }
        }
    };));
}
