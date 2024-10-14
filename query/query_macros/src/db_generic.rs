use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashSet;
use std::ops::Not;
use syn::ext::IdentExt;

use syn::{parse, parse_str, ItemFn, ItemImpl, ItemStruct, Token};

impl CanParse {
    fn new(input: TokenStream) -> parse::Result<Self> {
        if let Ok(try_this) = syn::parse2::<ItemFn>(input.clone()) {
            return Ok(CanParse::Fn(try_this));
        }

        if let Ok(try_this) = syn::parse2::<ItemImpl>(input.clone()) {
            return Ok(CanParse::Impl(try_this));
        }

        if let Ok(try_this) = syn::parse2::<ItemStruct>(input) {
            return Ok(CanParse::Struct(try_this));
        }

        Err(parse::Error::new(
            proc_macro2::Span::call_site(),
            "this macro only works on functions, impls or structs",
        ))
    }
    fn append_where_clause(&mut self, predicate: &str) {
        match self {
            CanParse::Fn(item_fn) => {
                let where_clause =
                    &mut item_fn.sig.generics.where_clause;

                match where_clause {
                    Some(where_clause) => {
                        where_clause
                            .predicates
                            .push(parse_str(predicate).unwrap());
                    }
                    None => {
                        *where_clause = Some(
                            parse_str(
                                format!("where {}", predicate)
                                    .as_str(),
                            )
                            .unwrap(),
                        );
                    }
                }
            }
            CanParse::Impl(item_impl) => {
                let where_clause =
                    &mut item_impl.generics.where_clause;

                match where_clause {
                    Some(where_clause) => {
                        where_clause
                            .predicates
                            .push(parse_str(predicate).unwrap());
                    }
                    None => {
                        *where_clause = Some(
                            parse_str(
                                format!("where {}", predicate)
                                    .as_str(),
                            )
                            .unwrap(),
                        );
                    }
                }
            }
            CanParse::Struct(item_struct) => {
                let where_clause =
                    &mut item_struct.generics.where_clause;

                match where_clause {
                    Some(where_clause) => {
                        where_clause
                            .predicates
                            .push(parse_str(predicate).unwrap());
                    }
                    None => {
                        *where_clause = Some(
                            parse_str(
                                format!("where {}", predicate)
                                    .as_str(),
                            )
                            .unwrap(),
                        );
                    }
                }
            }
        }
    }
    fn into_stream(self) -> TokenStream {
        match self {
            CanParse::Fn(item_fn) => quote! {#item_fn},
            CanParse::Impl(item_impl) => quote! {#item_impl},
            CanParse::Struct(item_struct) => quote! {#item_struct},
        }
    }
}

impl TryFrom<String> for DBs {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Sqlite" => Ok(DBs::Sqlite),
            "Postgres" => Ok(DBs::Postgres),
            "MySQL" => Ok(DBs::MySQL),
            "MariaDB" => Ok(DBs::MariaDB),
            "MSSQL" => Ok(DBs::MSSQL),
            "Oracle" => Ok(DBs::Oracle),
            _ => {
                return Err(
                    "only `Sqlite`, `MySQL`, `MariaDB`, `MSSQL` or `Oracle` is supported".to_owned(),
                )
            }
        }
    }
}

impl parse::Parse for Input {
    fn parse(input: parse::ParseStream) -> syn::parse::Result<Self> {
        let generic: Ident = input.parse()?;
        let generic = generic.into();
        input.parse::<Token![:]>()?;

        let mut dbs_set = Vec::<DBs>::new();

        let lookahead = input.lookahead1();
        while !input.is_empty() {
            if lookahead.peek(proc_macro2::Ident::peek_any) {
                let db_ident: Ident = input.parse()?;
                match DBs::try_from(db_ident.to_string()) {
                    Ok(res) => dbs_set.push(res),
                    Err(err) => {
                        return Err(syn::Error::new(
                            db_ident.span(),
                            err,
                        ))
                    }
                }
            } else {
                return Err(lookahead.error());
            }

            if input.peek(Token![|]) {
                input.parse::<Token![|]>()?;
            } else {
                break;
            }
        }

        if input.is_empty().not() {
            return Err(syn::Error::new(
                Span::call_site(),
                "format is `db_generic(DB: Sqlite | Postgres)`",
            ));
        }

        Ok(Input { generic, dbs_set })
    }
}

// input is in format of DB: Sqlite | Postgres
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DBs {
    Sqlite,
    Postgres,
    MySQL,
    MariaDB,
    MSSQL,
    Oracle,
}

enum CanParse {
    Fn(syn::ItemFn),
    Impl(syn::ItemImpl),
    Struct(syn::ItemStruct),
}

pub struct Input {
    pub generic: syn::TypeParam,
    pub dbs_set: Vec<DBs>,
}

pub trait WherePredicate {
    fn vec(&self, dbs: DBs) -> Vec<String>;
}

pub fn main(item: TokenStream, input: Input) -> TokenStream {
    let mut item = match CanParse::new(item) {
        Ok(item) => item,
        Err(e) => return e.to_compile_error().into(),
    };

    let generic_ident = input.generic.ident.to_string();

    let rules = crate::db_generic_rules::rules();

    rules.iter().for_each(|rule| {
        let mut wheres =
            input.dbs_set.clone().into_iter().map(|db| {
                rule.vec(db).into_iter().collect::<HashSet<_>>()
            });

        let mut output = wheres.next().unwrap_or_else(HashSet::new);

        while let Some(next) = wheres.next() {
            output.retain(|x| next.contains(x));
        }

        for where_ in output {
            let rule = where_.replace("$$DB", &generic_ident);
            item.append_where_clause(&rule)
        }
    });

    item.into_stream()
}
