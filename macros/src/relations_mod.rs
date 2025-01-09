use case::CaseExt;
use proc_macro_error::abort;
use quote::quote_spanned;
use std::mem::take;
use syn::Ident;

use proc_macro2::{TokenStream, TokenTree};
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

struct TwoEntities {
    first: Ident,
    second: Ident,
}

impl Parse for TwoEntities {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inner = Self {
            first: input.parse()?,
            second: input.parse()?,
        };
        if !input.is_empty() {
            return Err(input.error(
                "only two Entities can be specified",
            ));
        }
        Ok(inner)
    }
}

fn many_to_many(
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let mut ts = quote::quote!();


    let inner = match syn::parse2::<TwoEntities>(input) {
        Ok(v) => v,
        Err(err) => {
            return Err(err.to_compile_error());
        }
    };

    let base = inner.first;
    let related = inner.second;
    let schema_key = format!("{}s", related.to_string().to_snake());
    let base_fk = format!("{}_id", base.to_string().to_snake());
    let rel_fk = format!("{}_id", related.to_string().to_snake());
    let conj_table= format!("{}s{}s", base, related);

    let submitable = quote::quote!(
            relation_types::ManyToMany {
                schema_key: #schema_key,
                base_fk: #base_fk,
                base_t: stringify!(#base),

                rel_fk: #rel_fk,
                rel_t: stringify!(#related),

                conj_table: #conj_table,

                _entities: PhantomData::<(#base, #related)>,
            }
    );

    // extend: submit relation
    ts.extend(quote::quote!(
        const _ : () = {
            use ::cms_for_rust::relations::submit_prelude::*;

            submit! {
                Submitable::<CMS_DB> {
                    object: |base| { 
                        if base == stringify!(#base) {
                            Some(Box::new(#submitable))
                        } else {
                            None
                        }
                    },
                }
            }
        };

        const _ : () = {
            use ::cms_for_rust::migration::submit_migration_prelude::*;
            use ::cms_for_rust::relations::submit_prelude::relation_types;

            submit! {
                Submitable::<CMS_DB> {
                    object: || { Box::new( #submitable) },
                }
            }
        };
    ));

    return Ok(ts);
}

fn optional_to_many(
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let mut ts = quote::quote!();

    let inner = match syn::parse2::<TwoEntities>(input) {
        Ok(v) => v,
        Err(err) => {
            return Err(err.to_compile_error());
        }
    };

    let base = inner.first;
    let related = inner.second;
    let sql_relation_key = format!("{}_id", related.to_string().to_snake());
    let related_snake = related.to_string().to_snake();




    // extend: submit relation
    ts.extend(quote::quote!(
        const _ : () = {
            use ::cms_for_rust::relations::submit_prelude::*;

            submit! {
                Submitable::<CMS_DB> {
                    object: |base| { 
                        if base == stringify!(#base) {
                            Some(Box::new(
            relation_types::OneToMany {
                relation_key: #sql_relation_key,
                foreign_table: stringify!(#related),
                foreign_entity_snake_case: #related_snake,
                _entities: PhantomData::<(#base, #related)>,
            }
                            ))
                        } else {
                            None
                        }
                    },
                }
            }
        };
        const _ : () = {
            use ::cms_for_rust::migration::submit_migration_prelude::*;
            use ::cms_for_rust::relations::submit_prelude::relation_types;

            submit! {
                Submitable::<CMS_DB> {
                    object: || { Box::new(
                        relation_types::OptionalToMany::<#base, #related>::colomn_link(#sql_relation_key)
                    ) },
                }
            }
        };
    ));

    // // extend: submit migration
    // ts.extend(quote::quote!(
    // ));

    return Ok(ts);
}

#[derive(Debug)]
pub struct Input {
    ident: syn::Ident,
    rest: Vec<TokenTree>,
}

pub struct Inputs {
    pub inputs: Vec<Input>,
}

impl Parse for Inputs {
    fn parse(
        input: syn::parse::ParseStream,
    ) -> syn::Result<Self> {
        let mut inputs = Vec::new();
        let mut ty = None;
        let mut rest = vec![];

        while !input.is_empty() {
            if input.peek(Token![;]) {
                inputs.push(Input {
                    ident: ty.take().unwrap(),
                    rest: take(&mut rest),
                });
                input.parse::<Token![;]>()?;
                continue;
            }
            if ty.is_none() {
                ty = Some(input.parse()?);
            } else {
                input.step(|cursor| {
                    let mut c = *cursor;

                    while let Some((tt, next)) = c.token_tree() {
                        match tt {
                            TokenTree::Punct(ref punt)
                                if punt.as_char() == ';' =>
                            {
                                break;
                            }
                            _ => {
                                c = next;
                                rest.push(tt);
                            }
                        }
                    }

                    Ok(((), c))
                })?;
            }
        }

        if ty.is_some() {
            inputs.push(Input {
                ident: ty.take().unwrap(),
                rest: take(&mut rest),
            });
        }

        Ok(Self { inputs })
    }
}

pub fn main(input: Vec<Input>) -> TokenStream {
    let mut ts = quote::quote!();
    let mut input = input.into_iter();
    while let Some(input) = input.next() {
        match input.ident.to_string().as_str() {
            "optional_to_many" => match optional_to_many(
                input.rest.into_iter().collect(),
            ) {
                Ok(ok) => ts.extend(
                    quote_spanned!(input.ident.span()=> #ok),
                ),
                Err(err) => return err,
            },

            "many_to_many" => match many_to_many(
                input.rest.into_iter().collect(),
            ) {
                Ok(ok) => ts.extend(
                    quote_spanned!(input.ident.span()=> #ok),
                ),
                Err(err) => return err,
            },
            _ => {
                abort!(
                    input.ident.span(),
                    "unknown relation type: {}",
                    input.ident
                );
            }
        }
    }

    ts
}
