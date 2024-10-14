#![allow(unused)]
use std::collections::HashSet;
use std::mem::take;
use std::ops::Not;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use case::CaseExt;
use futures_util::Future;
use impl_prelude::*;
use inventory::Collect;
use inventory::Registry;
use query::impl_into_mut_arguments_prelude::Type;
use query::insert_one_st::InsertStOne;
use query::prelude::*;
use query::update_st::UpdateSt;
use query::{
    quick_query::QuickQuery, select_st::SelectSt,
    SupportNamedBind,
};
use serde::Deserialize;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use sqlx::ColumnIndex;
use sqlx::Database;
use sqlx::Decode;
use sqlx::Encode;
use sqlx::Executor;
use sqlx::Pool;
use sqlx::Row;
use tracing::warn;

use crate::entity_mod::DynEntity;
use crate::entity_mod::DynEntitySubmitable;



#[derive(Debug, Deserialize)]
pub struct Id {
    pub id: i64,
}



pub trait Relation<Base, S>
where
    S: Database,
{
    type RelatedEntity;
}

pub mod impl_prelude {
    pub use super::Relation;
    pub use crate::expr_mod::*;
    pub use sqlx::Database;
}












mod impls {
    use std::mem::take;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::{marker::PhantomData, ops::Not, pin::Pin};

    use futures_util::Future;
    use query::prelude::*;
    use query::update_st::UpdateSt;
    use query::{
        quick_query::QuickQuery, select_st::SelectSt,
        SupportNamedBind,
    };
    use serde_json::{json, Value};
    use sqlx::{
        ColumnIndex, Database, Decode, Encode, Executor, Pool,
        Row, Type,
    };
    use stmt::string_query;
    use tracing::warn;

    use crate::entity_mod::DynEntity;

    use super::{relation_types::ManyToMany, DynRelation};

    #[deprecated]
    pub struct ManyToManyGetOne<S> {
        pub schema_key: &'static str,
        pub conj_table: &'static str,
        pub ft: &'static str,
        pub fk: &'static str,
        pub lk: &'static str,
        pub data: Vec<(i64, i64, Value)>,
        pub related_entity: Box<dyn DynEntity<S>>,
        pub _db: PhantomData<S>,
    }

    impl<S: Send> DynRelation<S> for ManyToManyGetOne<S> {
        fn take(&mut self) -> Option<Value> {
            None
        }
        fn from_row(&mut self, row: &<S>::Row)
        where
            S: Database,
        {
            // no op
        }
        fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>>)
        where
            S: SupportNamedBind + Database,
        {
            // no op
        }
        fn schema_key(&self) -> &str {
            self.schema_key
        }
        fn sub_op<'this>(
            &'this mut self,
            _pool: Pool<S>,
            _limit: Arc<dyn super::SelectStLimit<S>>,
        ) -> Pin<Box<dyn Future<Output = ()> + 'this + Send>>
        where
            for<'s> &'s mut <S>::Connection:
                Executor<'s, Database = S>,
            S: Database + SupportNamedBind,
        {
            Box::pin(async {})
        }
    }




    




    type Limit<'q, S> = (
        dyn Fn(&str, &mut SelectSt<S, QuickQuery<'q>>)
            + Sync
            + Send
            + 'static,
    );


}



// impl<S> Collect for Submitable<S>
// where
//     S: Database,
// {
//     fn registry() -> &'static Registry {
//         static REGISTRY: Registry = Registry::new();
//         &REGISTRY
//     }
// }

pub mod relation_types {
    use std::marker::PhantomData;

    use case::CaseExt;
    use query::impl_into_mut_arguments_prelude::Type;
    use serde_json::{from_value, Value};
    use sqlx::{ColumnIndex, Database, Decode, Encode};

    use crate::{
        entity_mod::{DynEntity, DynEntitySubmitable},
        relations_v2::RelationStructure,
    };

    use super::{
        get_entity,
        impls::{ManyToManyGetAll, ManyToManyGetOne, ManyToManyInsertOne, ManyToManyUpdateConnect},
        DynRelation, EntityPhantom, Id, InsertInput, Relation,
        SubmitableRelation,
    };

    pub struct OptionalToMany<O, M>(

        pub &'static str,
        pub PhantomData<(O, M)>,
    );
            impl<O, M> OptionalToMany<O, M> {
                pub fn colomn_link(str: &'static str) -> Self {
                    Self(str, PhantomData)
                }
            }



    #[derive(Debug)]
    pub struct ManyToMany<Base, Related> {
        pub schema_key: &'static str,
        pub base_fk: &'static str,
        pub base_t: &'static str,
        pub rel_fk: &'static str,
        pub rel_t: &'static str,
        pub conj_table: &'static str,
        pub _entities: PhantomData<(Base, Related)>,
    }

    // impl<S, B, R> DynMigration<S> for ManyToMany<B, R>
    // where
    //     S: Database,
    // {
    //     fn migrate(
    //         &self,
    //     ) -> Box<
    //         dyn utils::ilist::EventfulList<
    //             crate::migration::TableContext<S>,
    //         >,
    //     > {
    //         todo!()
    //     }
    // }
    // z












pub mod submit_relation_prelude {
    pub use super::relation_types;
    pub use super::Submitable;
    pub use crate::expr_mod::*;
    pub use core::marker::PhantomData;
    pub use inventory::submit;
    pub use std::boxed::Box;
}

// all relations here are assumed to be one-to-many for now
pub trait StatufullRelation<S>
where
    Self: Send + 'static,
{
    fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>>)
    where
        S: SupportNamedBind + Database,
    {
        todo!()
    }
}

fn get_entity<S>(
    entity_key: &str,
) -> Result<Box<dyn DynEntity<S>>, &str>
where
    S: Database + SupportNamedBind,
{
    for entity in inventory::iter::<DynEntitySubmitable<S>> {
        let entity = (entity.object)();
        if entity.table_name().eq(entity_key) {
            return Ok(entity);
        }
    }
    Err(entity_key)
}
