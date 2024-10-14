mod service_migrate {

use std::ops::Not;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, visit::Visit, DeriveInput, Ident};

#[allow(unused)]
pub fn spanned_error<S: Spanned, D: ToTokens>(
    spanned: S,
    msg: D,
) -> TokenStream {
    quote_spanned!(spanned.span()=> compile_error!(#msg))
}

pub fn derive_entity(input: &DeriveInput) -> TokenStream {
    let mut ts = quote!();

    struct I<'ast> {
        data_structure: Ident,
        data_name: String,
        field: Vec<Ident>,
        errs: Vec<TokenStream>,
        ty: Vec<&'ast syn::Type>,
        field_name: Vec<String>,
        field_name_scoped: Vec<String>,
    }

    fn extend(this: I, ts: &mut TokenStream) {
        let I {
            errs,
            data_structure,
            data_name,
            ty,
            field,
            field_name,
            field_name_scoped,
        } = this;

        if errs.is_empty().not() {
            ts.extend(quote! {#(#errs)*});
            return;
        }

        ts.extend(quote! {
            const _: () = {
                use ::cms::entity::impl_prelude::*;

                impl<'r, S> ::cms::Entity<'r, S> for #data_structure 
                    where
                        #(
                            #ty: Type<S> + for<'e> Decode<'e, S>,
                        )*
                        for<'s> &'s str: ColumnIndex<S::Row>,
                        S: SupportNamedBind + Database,
                {
                    fn from_scoped_row(
                       row: &S::Row,
                    ) -> sqlx::Result<Self> {

                        Ok( Self {
                            #(
                                #field: row.try_get(
                                    #field_name_scoped
                                )?,
                            )*
                        })
                    }
                    fn mut_select(
                        stmt: &mut SelectSt<S, QuickQuery<'_>>
                    )
                    {
                        #(
                            stmt.select(
                                ft(#data_name)
                                .col(#field_name)
                                .alias(#field_name_scoped)
                            );
                        )*
                    }
                }
            };
        });
    }

    impl<'ast> Visit<'ast> for I<'ast> {
        fn visit_field(&mut self, i: &'ast syn::Field) {
            let span = i.span();
            match i.ident.as_ref() {
                Some(ident) => {
                    let ident = ident.clone();
                    let ident_s = ident.to_string();
                    let ident_scoped = format!("{}_{}", self.data_name, ident_s);
                    self.ty.push(&i.ty);
                    self.field.push(ident);
                    self.field_name.push(ident_s);
                    self.field_name_scoped.push(ident_scoped);
                }
                None => {
                    self.errs.push(quote_spanned!(span=>
                        compile_error!("unamed fields are not supported")
                    ));
                }
            }
        }
        fn visit_data_enum(&mut self, i: &'ast syn::DataEnum) {
            let span = i.enum_token.span();
            self.errs.push(quote_spanned!(span=>
                compile_error!("enums are not supported")
            ));
        }
        fn visit_data_union(&mut self, i: &'ast syn::DataUnion) {
            let span = i.union_token.span();
            self.errs.push(quote_spanned!(span=>
                compile_error!("unions are not supported")
            ));
        }
    }

    let mut inner = I {
        data_structure: input.ident.clone(),
        data_name: input.ident.to_string().to_lowercase(),
        ty: Default::default(),
        errs: Default::default(),
        field: Default::default(),
        field_name: Default::default(),
        field_name_scoped: Default::default(),
    };

    inner.visit_derive_input(input);
    extend(inner, &mut ts);

    ts

}

pub fn migrate_service(input: &DeriveInput) -> TokenStream {
    let mut ts = quote!();

    struct I {
        errs: Vec<TokenStream>,
        name: Ident,
        field: Vec<(String, syn::Type)>,
    }

    let mut i = I {
        errs: Default::default(),
        name: input.ident.clone(),
        field: Default::default(),
    };

    impl<'a> Visit<'a> for I {
        fn visit_field(&mut self, i: &'a syn::Field) {
            let span = i.span();
            match i.ident.as_ref() {
                Some(ident) => {
                    let ident = ident.to_string();
                    let ty = i.ty.clone();
                    self.field.push((ident, ty));
                }
                None => {
                    self.errs.push(quote_spanned!(span=>
                        compile_error!("unamed fields are not supported")
                    ));
                }
            }
        }
        fn visit_data_enum(&mut self, i: &'a syn::DataEnum) {
            let span = i.enum_token.span();
            self.errs.push(quote_spanned!(span=>
                compile_error!("enums are not supported")
            ));
        }
        fn visit_data_union(&mut self, i: &'a syn::DataUnion) {
            let span = i.union_token.span();
            self.errs.push(quote_spanned!(span=>
                compile_error!("unions are not supported")
            ));
        }
    }

        fn extend(this: I, ts: &mut TokenStream) {
            if this.errs.is_empty().not() {
                let errs = this.errs;
                ts.extend(quote! {#(#errs)*});
            } else {
                let name = this.name;
                let (fi, ty): (Vec<_>, Vec<_>) =
                    this.field.into_iter().unzip();

                ts.extend(quote! {
                    const _: () = {
                        use cms::migration::migration_prelude::*;

                        submit!(SubmitMigration(
                            stringify!(#name),
                            |stmt: &mut CTS<DB>| {
                                #(
                                    stmt.column::<#ty>(#fi, ());
                                )*
                                stmt.column("id", primary_key());
                            }
                        ));
                    };
                })
            }
        }

    i.visit_derive_input(input);
    extend(i, &mut ts);

    ts
}

pub fn entity(input: &DeriveInput) -> TokenStream {
    let mut ts = quote!();

    struct I {
        errs: Vec<TokenStream>,
        name: Ident,
        field: Vec<Ident>,
    }

        fn extend(this: I, ts: &mut TokenStream) {
            if this.errs.is_empty().not() {
                let errs = this.errs;
                ts.extend(quote! {#(#errs)*});
            } else {
                let ident_s = this.name;
                let name = ident_s.to_string();
            let name_lc = name.to_lowercase();
                let ident = this.field;

                let field_s = ident
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>();

                let field2 = ident
                    .iter()
                    .map(|a| format!("{}.{}", name, a));
                // let field = ident.iter().map(|a| format!("{}.{}", name, a));
                let aliase = ident.iter().map(|a| {
                    format!("{}_{}", name.to_lowercase(), a)
                });
                // let aliase2 = ident.iter().map(|a| format!("{}_{}", name.to_lowercase(), a));

                let id = format!("{}.id", name);

                ts.extend(quote! {

            const _: () = {
                use cms::http_server::entity_prelude::*;
                inventory::submit! {
                    EntityVTable::<DB> {
                        name: #name,
                        mut_select: |stmt: MutSelectVF<DB>| {
                            let stmt = unsafe {stmt.as_mut().unwrap()};
                            stmt.select(col(#id).alias("local_id"));
                            #(
                                stmt.select(col(#field2).alias(#aliase));
                            )*
                            Ok(())
                        },
                        from_row: |row: FromRowVF<DB>, value: &mut Value| {
                            let row= unsafe {row.as_ref().unwrap()};
                            let id: i64 = row.get("local_id");
                            let data = <#ident_s as Entity<'_, DB>>::from_scoped_row(
                                row, 
                            )?;
                            *value = json!({
                                "id": id,
                                "data": {
                                    #(
                                        #field_s: data.#ident,
                                    )*
                                },
                                "relations": {},
                            });
                            Ok(())
                        },
                    }
                }
            };
        })
            }
        }

    impl<'a> Visit<'a> for I {
        fn visit_field(&mut self, i: &'a syn::Field) {
            let span = i.span();
            match i.ident.as_ref() {
                Some(ident) => {
                    self.field.push(ident.clone());
                }
                None => {
                    self.errs.push(quote_spanned!(span=>
                        compile_error!("unamed fields are not supported")
                    ));
                }
            }
        }
        fn visit_data_enum(&mut self, i: &'a syn::DataEnum) {
            let span = i.enum_token.span();
            self.errs.push(quote_spanned!(span=>
                compile_error!("enums are not supported")
            ));
        }
        fn visit_data_union(&mut self, i: &'a syn::DataUnion) {
            let span = i.union_token.span();
            self.errs.push(quote_spanned!(span=>
                compile_error!("unions are not supported")
            ));
        }
    }

    let mut i = I {
        errs: Default::default(),
        name: input.ident.clone(),
        field: Default::default(),
    };

    i.visit_derive_input(input);
    extend(i, &mut ts);

    ts
}
}

mod relation_mod {
#![allow(unused)]
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

use crate::service_migrate::spanned_error;

pub enum Ty {
    HasOptionalToMany(kw::has_optional_to_many),
    HasManyToMany(kw::has_many_to_many),
}

pub struct Input {
    pub relation: Vec<(Ident, Ty, Ident)>,
}

pub mod kw {
    syn::custom_keyword!(has_optional_to_many);
    // syn::custom_keyword!(has_optional);
    // syn::custom_keyword!(has_one);
    // syn::custom_keyword!(has_one_to_many);
    syn::custom_keyword!(has_many_to_many);
}

impl Parse for Ty {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(kw::has_optional_to_many) {
            return Ok(Ty::HasOptionalToMany(
                input.parse::<kw::has_optional_to_many>()?,
            ));
        } else if input.peek(kw::has_many_to_many) {
            return Ok(Ty::HasManyToMany(
                input.parse::<kw::has_many_to_many>()?,
            ));
        }
        Err(syn::Error::new(
            input.span(),
            "unknown relation type, expected `has_optional_to_many` or `has_many_to_many`",
        ))
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut relation = Vec::new();

        while !input.is_empty() {
            relation.push((
                input.parse()?,
                input.parse()?,
                input.parse()?,
            ));
            if input.peek(syn::Token![;]) {
                input.parse::<syn::Token![;]>()?;
            }
        }

        Ok(Self { relation })
    }
}

pub fn main(input: Input) -> TokenStream {
    let mut ts = quote!();

    // let input = syn::parse_macro_input!(input as Input);
    for (name, ty, target) in input.relation {
        match ty {
            Ty::HasOptionalToMany(_) => {
                optional_to_many(&name, &target, &mut ts);
            }
            Ty::HasManyToMany(sp) => {
                spanned_error(sp, "relation is not supported");
            }
        }
    }

    ts
}

fn optional_to_many(
    name: &Ident,
    target: &Ident,
    ts: &mut TokenStream,
) {
    let name_str = name.to_string();
    let target_s = target.to_string().to_lowercase();
    let id = target.to_string().to_lowercase() + "_id";

    ts.extend(quote! {
        const _: () = {
        use ::cms::migration::migration_prelude::*;

        submit!(SubmitMigration(stringify!(#name), |stmt: &mut CTS<DB>| {
            stmt.foreign_key(Fk {
                not_null: false,
                column: #id,
                refer_table: stringify!(#target),
                refer_column: "id",
            });
        }));
    };});

    ts.extend(quote! {
        const _: () = {
        use ::cms::http_server::relation_on_select_prelude::*;

        submit!(RelationVTable::<DB> {
            base_entity: stringify!(#name),
            related_entity: #target_s,
            mut_select: |stmt: MutSelect<DB>| {
                let stmt = unsafe { stmt.as_mut().unwrap() };
                stmt.join(Join {
                    ty: join_type::Left,
                    on_table: stringify!(#target),
                    on_column: "id",
                    local_column: #id,
                });
                stmt.select(ft(stringify!(#target)).col("id").alias(#id));
                #target::mut_select(stmt);
                Ok(())
            },
            from_row: |row: FromRow<DB>, value: &mut Value| {
                let row = unsafe { row.as_ref().unwrap() };
                let id: Option<i64> = row.get(#id);
                if let Some(id) = id {
                    let data = <#target as Entity<'_, DB>>::from_scoped_row(
                        row
                    )?;
                    *value = json!({
                        "id": id,
                        "data": data,
                        "relations": {},
                    });
                }
                Ok(())
            },
        });
    };});
}
}
