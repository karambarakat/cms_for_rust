use case::CaseExt;
use proc_macro::Span;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote};
use syn::{spanned::Spanned, visit::Visit, DeriveInput, Ident};

pub fn main(input: &DeriveInput) -> TokenStream {
    let mut ts = quote!();

    #[derive(Default, Debug)]
    struct Inner<'ast> {
        fields: Vec<(&'ast syn::Type, &'ast syn::Ident)>,
    }


    impl<'ast> Visit<'ast> for Inner<'ast> {
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
                    self.fields.push((&field.ty, ident));
                }
                None => {
                    abort! (field.span(), 
                        "unamed fields are not supported"
                    );
                }
            }
        }
    }


    let mut inner = Inner::default();
    inner.visit_derive_input(input);


    let name = &input.ident;
    let (field_ty, field_ident) = inner.fields.iter().cloned().unzip::<_, _, Vec<_>, Vec<_>>();
    let field_str = field_ident.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let field_str_scoped = field_ident.iter().map(|s| format!("{}_{}", name.to_string().to_snake(), s)).collect::<Vec<_>>();

    let partial_ident = Ident::new(&format!("{}Partial", name), Span::call_site().into());

    // extend: entity trait
    ts.extend(quote!(
        const _: () = {
            use ::cms_for_rust::entities::impl_prelude::*;

            #[derive(Deserialize, Clone)]
            pub struct #partial_ident {
                #(
                    pub #field_ident: Option<#field_ty>,
                )*
            }

            impl<S> PartialEntity<S> for #partial_ident
            where
                S: SupportNamedBind + Sync + SqlxQuery,
                #(
                    #field_ty: Type<S> + for<'d> Encode<'d, S>,
                )*
            {
                fn update_st(
                    self,
                    st: &mut UpdateSt<S, QuickQuery<'_>>,
                ) { #(
                    match self.#field_ident {
                        Some(v) => {
                            st.set(#field_str, move || v);
                        }
                        None => {}
                    };
                )* }
            }

            impl<S> Entity<S> for #name 
            where
                S: SupportNamedBind + Sync + SqlxQuery,
                for<'s> &'s str: ColumnIndex<S::Row>,
                #(
                    #field_ty: Type<S> + for<'d> Decode<'d, S>,
                )*
            {
                type Partial = #partial_ident;

                fn migrate<'q>(
                    stmt: &mut MigrateArg<'q, S>,
                ) {
                    stmt.column("id", primary_key());
                    #(
                        stmt.column::<#field_ty>(#field_str, ());
                    )*
                }
                fn table_name() -> &'static str {
                    stringify!(#name)
                }
                fn members_scoped() -> Vec<&'static str> {
                    vec![
                        #(
                           #field_str_scoped,
                        )*
                    ]
                }
                fn members() -> Vec<&'static str> {
                    vec![ #( #field_str,)* ]
                }
                fn from_row(row: &S::Row) -> Result<Self, sqlx::Error> {
                    Ok(Self {
                        #(
                            #field_ident: row.try_get(#field_str)?,
                        )*
                    })
                }
                fn from_row_scoped(row: &S::Row) -> Result<Self, sqlx::Error> {
                    Ok(Self {
                        #(
                            #field_ident: row.try_get(#field_str_scoped)?,
                        )*
                    })
                }
            }
        };
    ));

    // extend: submit entity
    ts.extend(quote!(
        const _: () = {
            use ::cms_for_rust::entities::submit_entity_prelude::*;

            submit! {
                Submitable::<CMS_DB> {
                    object: || { Box::new(EntityPhantom::<#name>(PhantomData)) },
                }
            }
        };
    ));

    // extend: submit migration
    ts.extend(quote!(
        const _: () = {
            use ::cms_for_rust::migration::submit_migration_prelude::*;

            submit! {
                Submitable::<CMS_DB> {
                    object: || { Box::new(EntityPhantom::<#name>(PhantomData)) },
                }
            }
        };
    ));
            

    ts
}
