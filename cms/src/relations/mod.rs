#![allow(unused)]
#![allow(deprecated)]
pub mod many_to_many;
pub mod optional_to_many;
pub mod other;
pub use cms_macros::relations as define;
use std::{collections::HashSet, ops::Not, pin::Pin, sync::Arc};

use case::CaseExt;
use futures_util::Future;
use inventory::{Collect, Registry};
use queries_for_sqlx::{
    insert_one_st::InsertStOne, quick_query::QuickQuery,
    select_st::SelectSt, update_st::UpdateSt, SupportNamedBind,
};
use serde::Deserialize;
use serde_json::{Map, Value};
use sqlx::{Database, Executor, Pool};

pub trait SubmitableRelation<S>
where
    Self: Send + 'static,
{
    fn convert(
        &self,
        origin: &'static str,
        input: Value,
    ) -> Box<dyn DynRelation<S>>;
    fn schema_key(&self) -> &str;
}

pub trait DynRelation<S>: Send {
    fn schema_key(&self) -> &str;
    fn on_update(&self, st: &mut UpdateSt<S, QuickQuery<'_>>)
    where
        S: Database + SupportNamedBind,
    {
        // no_op
    }
    fn on_insert(&self, st: &mut InsertStOne<'_, S>)
    where
        S: Database,
    {
        // no_op
    }
    fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>>)
    where
        S: SupportNamedBind + Database,
    { // no_op
    }
    #[deprecated(
        note = "this method should be combined with take"
    )]
    fn from_row(&mut self, row: &S::Row)
    where
        S: Database;
    #[deprecated(
        note = "this method should be combined with from_row"
    )]
    fn take(&mut self) -> Option<Value>;
    fn sub_op2<'this>(
        &'this mut self,
        _pool: Pool<S>,
        returned_id: i64,
    ) -> Pin<Box<dyn Future<Output = ()> + 'this + Send>>
    where
        for<'s> &'s mut S::Connection:
            Executor<'s, Database = S>,
        S: Database + SupportNamedBind,
    {
        Box::pin(async {
            // no op
        })
    }
    fn sub_op<'this>(
        &'this mut self,
        _pool: Pool<S>,
        _limit: Arc<dyn SelectStLimit<S>>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'this + Send>>
    where
        for<'s> &'s mut S::Connection:
            Executor<'s, Database = S>,
        S: Database + SupportNamedBind,
    {
        Box::pin(async {
            // no op
        })
    }
    fn take2(&mut self, _local_id: i64) -> Option<Value> {
        None
    }
}

pub trait SelectStLimit<S>
where
    Self: Send + Sync + 'static,
{
    fn limit(&self, st: &mut SelectSt<S, QuickQuery<'_>>)
    where
        S: Database + SupportNamedBind;
}

pub struct Submitable<S> {
    pub object:
        fn(base: &str) -> Option<Box<dyn SubmitableRelation<S>>>,
}

impl<S> Collect for Submitable<S>
where
    S: Database,
{
    fn registry() -> &'static Registry {
        static REGISTRY: Registry = Registry::new();
        &REGISTRY
    }
}

pub fn get_relation<S>(
    base_entity: &str,
    origin: &'static str,
    mut input: Map<String, Value>,
) -> Result<Vec<Box<dyn DynRelation<S>>>, Vec<String>>
where
    crate::relations::Submitable<S>: Collect,
{
    let mut left_keys =
        input.keys().map(|e| e.clone()).collect::<HashSet<_>>();

    let relations_obj = inventory::iter::<
            Submitable<S>,
        >()
        .into_iter()
        .filter_map(|e| {
            // panic!("{}", base_entity);
            match (e.object)(&base_entity.to_camel()) {
                Some(e) => {
                    let foriegn_entity = e.schema_key();

                if left_keys.contains(foriegn_entity) {
                        left_keys.remove(foriegn_entity);
                    Some(e.convert(
                            origin,
                        input.remove(foriegn_entity)
                                .expect("multiple relations claimed the same relation key"),
                    ))
                } else {
                    // other relation claimed to be related
                    None
                }
                },
                None => None
            }
            })
        .collect::<Vec<_>>();

    if left_keys.is_empty().not() {
        return Err(left_keys.into_iter().collect());
    }

    Ok(relations_obj)
}

#[derive(Deserialize)]
struct InsertInput<D> {
    // "id": true,
    id: bool,
    // "attributes": true
    attributes: bool,
    // "data": { "id": 2 }, input data
    data: D,
    // relations: R, // populating data on insert is not supported for now
}

#[derive(Debug, serde::Deserialize)]
#[allow(non_camel_case_types)]
pub enum UpdateInput<I> {
    set { id: I },
    connect { id: I },
    disconnect { id: I },
}

pub mod submit_prelude {
    pub use super::Submitable;
    pub use crate::entities::*;
    pub use core::marker::PhantomData;
    pub use inventory::submit;
    pub use std::boxed::Box;
    pub mod relation_types {
        pub use super::super::many_to_many::ManyToMany;
        pub use super::super::optional_to_many::OneToMany;
        pub use super::super::optional_to_many::OptionalToMany;
    }
}
