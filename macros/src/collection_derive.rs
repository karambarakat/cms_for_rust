use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{spanned::Spanned, visit::Visit, DeriveInput};

pub fn main(input: DeriveInput) -> TokenStream {
    let mut ts = quote!();

    struct Memeber<'ast> {
        ty: &'ast syn::Type,
        name: &'ast Ident,
        name_scoped: String,
    }

    struct MainDerive<'ast> {
        fields: Vec<Memeber<'ast>>,
        table_lower_case: String
    }

    impl<'ast> Visit<'ast> for MainDerive<'ast> {
        fn visit_generics(&mut self, i: &'ast syn::Generics) {
            if i.lt_token.is_some() {
                abort!(i.span(),
                    "geneerics are not supported"
                    );
            }
        }
        fn visit_field(&mut self, field: &'ast syn::Field) {
            match field.ident.as_ref() {
                Some(ident) => {
                    self.fields.push(Memeber {
                        ty: &field.ty,
                        name: ident,
                        name_scoped: format!("{}_{}", self.table_lower_case, ident)
                    })
                }
                None => {
                    abort! (field.span(), 
                        "unamed fields are not supported"
                    );
                }
            }
        }
    }

    let d_ident = &input.ident;
    let partial_ident = Ident::new(&format!("{}Partial", d_ident), proc_macro2::Span::call_site());

    let mut main_derive = MainDerive { 
        fields: vec![],
        table_lower_case: d_ident.to_string().to_lowercase(),
    };
    main_derive.visit_derive_input(&input);

    let m_ty = main_derive.fields.iter().map(|m| m.ty.clone()).collect::<Vec<_>>();
    let m_name = main_derive.fields.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
    let m_name_scoped =
        main_derive.fields.iter().map(|m| m.name_scoped.clone()).collect::<Vec<_>>();



    ts.extend(quote!(
        #[derive(::cms_for_rust::macro_prelude::serde::Deserialize)]
        pub struct #partial_ident {
            #(pub #m_name: ::cms_for_rust::macro_prelude::derive_collection::Update<#m_ty>,)*
        }
    ));

    ts.extend(quote!( const _: () = {
        use ::cms_for_rust::macro_prelude::derive_collection::*;

        submit! {SubmitDynCollection {
            obj: || Box::new(PhantomData::<#d_ident>)
        }}
        submit!(SubmitDynMigrate{
            obj: || Box::new(PhantomData::<#d_ident>)
        });

        impl<S> Collection<S> for #d_ident 
            where 
        S: Database + SupportNamedBind + SqlxQuery,
        for<'s> &'s str: ColumnIndex<<S as Database>::Row>,
        #(
            #m_ty: Type<S> + for<'c> Decode<'c, S> + for<'e> Encode<'e, S>,
        )*
        {
            type PartailCollection = #partial_ident;

            fn on_migrate(stmt: &mut CreatTableSt<S>) {
                stmt.column("id", primary_key::<S>());
                #(
                stmt.column(
                    stringify!(#m_name),
                    col_type_check_if_null::<#m_ty>(),
                );
                )*
            }
            fn on_update(
                stmt: &mut UpdateSt<S>,
                this: Self::PartailCollection,
            ) -> Result<(), String>
            {
                #(
                if let Update::set(val) = this.#m_name {
                    stmt.set(stringify!(#m_name).to_string(), {
                        val
                    });
                };)*
                Ok(())
            }
        
            fn members() -> &'static [&'static str] {
                 &[
                     #(
                         stringify!(#m_name),
                     )*
                 ]
            }
        
            fn members_scoped() -> &'static [&'static str] {
                 &[
                     #(
                         #m_name_scoped,
                     )*
                 ]
            }
        
            fn table_name() -> &'static str {
                stringify!(#d_ident)
            }
        
            fn on_select(stmt: &mut SelectSt<S>)
            {
                #(
                   stmt.select(stringify!(#m_name_scoped));
                )*
            }
        
            fn from_row_noscope(row: &<S as Database>::Row) -> Self
            {
                Self { #(
                    #m_name: row.get(stringify!(#m_name)),
                )*}
            }
        
            fn from_row_scoped(row: &<S as Database>::Row) -> Self
            {
                Self { #(
                        #m_name: row.get(#m_name_scoped),
                )*}
            }
        
            fn on_insert(
                self,
                stmt: &mut InsertSt<S>,
            ) -> Result<(), String>
            {
                #(
                    stmt.insert(stringify!(#m_name).to_owned(), {
                        self.#m_name
                    });
                )*
                Ok(())
            }
            
        }
    };));

    ts
}
