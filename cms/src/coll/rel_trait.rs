use std::{future::Future, pin::Pin, sync::Arc};

use queries_for_sqlx::{ident_safety::PanicOnUnsafe, prelude::*, quick_query::QuickQuery};
use serde_json::{Map, Value};
use sqlx::{sqlite::SqliteRow, Pool, Sqlite};

pub trait Relation {}

#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Operation {
    SelectOne,
    SelectMany,
    Insert,
    Update,
    Delete,
}

pub trait DynRelation {
    fn init_insert(
        self: Arc<Self>,
        to: &str,
        value: Value,
    ) -> Result<Box<dyn OnInsertRelation + Send + Sync>, String>
    {
        Err("operation is not supported".to_owned())
    }
    fn init_update(
        self: Arc<Self>,
        to: &str,
        value: Value,
    ) -> Result<Box<dyn OnUpdateRelation + Send + Sync>, String>
    {
        Err("operation is not supported".to_owned())
    }
    fn init_get(
        self: Arc<Self>,
        op: Operation,
        to: &str,
        value: Value,
    ) -> Result<Box<dyn OnGetRelation + Send + Sync>, String>
    {
        Err("operation is not supported".to_owned())
    }
}

pub trait OnUpdateRelation {
    fn key(&self) -> &str;
    fn on_update(
        &mut self,
        stmt: &mut stmt::UpdateSt<Sqlite, QuickQuery>,
    ) {
    }
    fn from_row(&mut self, row: &SqliteRow) {}
    fn sub_op(
        &mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async {})
    }
    fn take(&mut self) -> Value {
        Value::Null
    }
}

pub trait OnGetRelation {
    fn key(&self) -> &str;
    fn on_select(
        &mut self,
        stmt: &mut stmt::SelectSt<Sqlite, QuickQuery, PanicOnUnsafe>,
    ) {
    }

    fn from_row(&mut self, row: &mut SqliteRow) {}
    fn sub_op(
        &mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async {})
    }
    fn take(&mut self, id: i64) -> Value {
        Value::Null
    }
}

pub struct ImplOnGetRelation<
    This,
    Data,
    OnSelect,
    FromRow,
    Take,
    SubOp,
> where
    OnSelect: FnMut(
        &This,
        &mut Data,
        &mut stmt::SelectSt<Sqlite, QuickQuery, PanicOnUnsafe>,
    ),
    SubOp: FnMut(
        &This,
        &mut Data,
        Pool<Sqlite>,
    )
        -> Pin<Box<dyn Future<Output = ()> + Send>>,
    FromRow: FnMut(&This, &mut Data, &mut SqliteRow),
    Take: FnMut(&This, &mut Data, i64) -> Value,
{
    pub this: Arc<This>,
    pub data: Data,
    pub key: String,
    pub from_row: FromRow,
    pub on_select: OnSelect,
    pub sub_op: SubOp,
    pub take: Take,
}

impl<This, Data, FromRow, Take, OnSelect, SubOp> OnGetRelation
    for ImplOnGetRelation<
        This,
        Data,
        OnSelect,
        FromRow,
        Take,
        SubOp,
    >
where
    OnSelect: FnMut(
        &This,
        &mut Data,
        &mut stmt::SelectSt<Sqlite, QuickQuery, PanicOnUnsafe>,
    ),
    SubOp: FnMut(
        &This,
        &mut Data,
        Pool<Sqlite>,
    )
        -> Pin<Box<dyn Future<Output = ()> + Send>>,
    FromRow: FnMut(&This, &mut Data, &mut SqliteRow),
    Take: FnMut(&This, &mut Data, i64) -> Value,
{
    fn key(&self) -> &str {
        self.key.as_str()
    }

    fn on_select(
        &mut self,
        stmt: &mut stmt::SelectSt<Sqlite, QuickQuery, PanicOnUnsafe>,
    ) {
        (self.on_select)(&self.this, &mut self.data, stmt)
    }

    fn from_row(&mut self, row: &mut SqliteRow) {
        (self.from_row)(&self.this, &mut self.data, row)
    }
    fn sub_op(
        &mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        (self.sub_op)(&self.this, &mut self.data, pool)
    }
    fn take(&mut self, id: i64) -> Value {
        (self.take)(&self.this, &mut self.data, id)
    }
}

pub trait OnInsertRelation {
    fn key(&self) -> &str;
    fn on_insert(
        &mut self,
        stmt: &mut stmt::InsertStOne<'_, Sqlite>,
    ) {
    }
    fn sub_op_1(
        &mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async {})
    }
    fn sub_op_2(
        &mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async {})
    }
    fn from_row(&mut self, row: &SqliteRow) {}
    fn take(&mut self) -> Value {
        Value::Null
    }
}
