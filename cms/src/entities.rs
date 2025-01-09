pub use crate::cms_macros::service as define;
pub use crate::cms_macros::Entity;
use std::marker::PhantomData;

use queries_for_sqlx::{
    create_table_st::CreateTableSt,
    insert_one_st::InsertStOne,
    prelude::{col, ft, SelectHelpers},
    quick_query::QuickQuery,
    select_st::SelectSt,
    update_st::UpdateSt,
    IntoMutArguments, Query, SupportNamedBind,
};
use serde_json::Value;
use sqlx::Database;

pub trait PartialEntity<S> {
    fn update_st(self, st: &mut UpdateSt<S, QuickQuery<'_>>)
    where
        S: Database,
        S: SupportNamedBind;
}

pub trait Entity<S>
where
    S: Database,
    Self: Sized,
{
    type Partial;

    fn migrate<'q>(stmt: &mut CreateTableSt<S, QuickQuery<'q>>)
    where
        for<'q1> QuickQuery<'q1>: Query<S>;
    fn table_name() -> &'static str;
    fn insert_st(self, st: &mut InsertStOne<S>)
    where
        for<'q> Self: IntoMutArguments<'q, S>,
    {
        st.insert_struct(Self::members().as_slice(), self);
    }
    fn from_row(row: &S::Row) -> Result<Self, sqlx::Error>;
    fn members_scoped() -> Vec<&'static str>;
    fn members() -> Vec<&'static str>;
    fn from_row_scoped(
        row: &S::Row,
    ) -> Result<Self, sqlx::Error>;
}

pub trait DynEntity<S: Database>
where
    Self: Send + Sync,
{
    fn resolve_name_conflict(
        &self,
        all: &Vec<&'static str>,
    ) -> Result<(), ()> {
        if all.iter().any(|e| e.eq(&self.table_name())) {
            Err(())
        } else {
            Ok(())
        }
    }
    fn on_select(&self, st: &mut SelectSt<S, QuickQuery>)
    where
        S: SupportNamedBind,
    {
        for member in self.members() {
            st.select(col(member));
        }
    }
    fn on_select_scoped(&self, st: &mut SelectSt<S, QuickQuery>)
    where
        S: SupportNamedBind,
    {
        for member in self.members() {
            st.select(
                ft(self.table_name())
                    .col(member)
                    .prefix_alias(self.table_name()),
            );
        }
    }
    fn into_insert(
        &self,
        from: &Value,
        insert: &mut InsertStOne<S>,
    );
    fn table_name(&self) -> &'static str;
    fn into_buffer(
        &self,
        from: &Value,
        buffer: &mut <S as sqlx::database::HasArguments>::Arguments,
    ) -> Result<(), sqlx::Error>;
    fn from_row_scoped(
        &self,
        row: &S::Row,
    ) -> Result<Value, sqlx::Error>;
    fn from_row2(
        &self,
        row: &S::Row,
    ) -> Result<Value, sqlx::Error> {
        let mut val = serde_json::json!(null);
        #[allow(deprecated)]
        self.from_row(row, &mut val)?;
        Ok(val)
    }
    #[deprecated = "use from_row2"]
    fn from_row(
        &self,
        row: &S::Row,
        into: &mut Value,
    ) -> Result<(), sqlx::Error>;
    fn members(&self) -> Vec<&'static str>;
    fn clone_self(&self) -> Box<dyn DynEntity<S>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityPhantom<E>(pub PhantomData<E>);

impl<S, T> DynEntity<S> for EntityPhantom<T>
where
    S: Database + SupportNamedBind + Sync + Send,
    T: Entity<S>
        + Send
        + Sync
        + 'static
        + for<'q> IntoMutArguments<'q, S>
        + for<'d> serde::Deserialize<'d>,
    T: serde::Serialize,
{
    fn clone_self(&self) -> Box<dyn DynEntity<S>> {
        Box::new(EntityPhantom(PhantomData::<T>))
    }
    fn into_insert(
        &self,
        from: &Value,
        insert: &mut InsertStOne<S>,
    ) {
        insert.insert_struct(
            T::members().as_slice(),
            serde_json::from_value::<T>(from.clone()).unwrap(),
        );
    }
    fn into_buffer(
        &self,
        from: &Value,
        buffer: &mut <S as sqlx::database::HasArguments>::Arguments,
    ) -> Result<(), sqlx::Error> {
        let t: T = serde_json::from_value(from.clone())
            .expect("failed to deserialize");
        t.into_arguments(buffer);
        Ok(())
    }
    fn members(&self) -> Vec<&'static str> {
        T::members()
    }
    fn table_name(&self) -> &'static str {
        T::table_name()
    }

    fn from_row(
        &self,
        row: &S::Row,
        into: &mut Value,
    ) -> Result<(), sqlx::Error> {
        let t: T = T::from_row(row).unwrap();
        let t = serde_json::to_value(t)
            .expect("failed to serialize");
        *into = t;
        Ok(())
    }
    fn from_row_scoped(
        &self,
        row: &<S as Database>::Row,
    ) -> Result<Value, sqlx::Error> {
        let t = T::from_row_scoped(row).unwrap();

        let t = serde_json::to_value(t)
            .expect("failed to serialize");
        Ok(t)
    }
}

pub struct DynEntitySubmitable<S: Database> {
    pub object: fn() -> Box<dyn DynEntity<S>>,
}

impl<S: Database> inventory::Collect for DynEntitySubmitable<S> {
    fn registry() -> &'static inventory::Registry {
        static REGISTRY: inventory::Registry =
            inventory::Registry::new();
        &REGISTRY
    }
}

pub mod sqlx_extention {
    pub use crate::queries_for_sqlx_extention::*;
}

pub mod derive_prelude {
    pub use cms_macros::Entity;
    pub use cms_macros::IntoMutArguments;
    pub use core::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
    };
    pub use serde::{Deserialize, Serialize};
    pub use sqlx::FromRow;
}

pub mod submit_entity_prelude {
    pub use super::DynEntity;
    pub use super::DynEntitySubmitable as Submitable;
    pub use crate::entities::EntityPhantom;
    pub use core::marker::PhantomData;
    pub use inventory::submit;
    pub use std::boxed::Box;
}

pub mod impl_prelude {
    pub use core::clone::Clone;
    pub use core::option::Option::{self, None, Some};
    pub use core::result::Result::{self, Err, Ok};

    pub type MigrateArg<'q, S> =
        CreateTableSt<S, QuickQuery<'q>>;

    pub use super::Entity;
    pub use super::PartialEntity;
    pub use crate::queries_for_sqlx_extention::*;
    pub use queries_for_sqlx::create_table_st::CreateTableSt;
    pub use queries_for_sqlx::prelude::*;
    pub use queries_for_sqlx::quick_query::QuickQuery;
    pub use queries_for_sqlx::select_st::SelectSt;
    pub use queries_for_sqlx::update_st::UpdateSt;
    pub use queries_for_sqlx::Query;
    pub use queries_for_sqlx::SupportNamedBind;
    pub use serde::Deserialize;
    pub use sqlx;
    pub use sqlx::prelude::*;
    pub use sqlx::ColumnIndex;
    pub use sqlx::Database;
    pub use sqlx::Row;
}
