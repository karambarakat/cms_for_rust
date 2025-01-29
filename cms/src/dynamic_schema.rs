use core::fmt;
use std::{
    collections::HashMap, future::Future, marker::PhantomData,
    mem, pin::Pin, sync::Arc,
};

use inventory::collect;
use queries_for_sqlx::{
    ident_safety::{append_schema, PanicOnUnsafe},
    prelude::*,
    quick_query::QuickQuery,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_value, Value};
use sqlx::{sqlite::SqliteRow, Pool, Sqlite};
use tokio::sync::RwLock;

use crate::traits::Collection;

use super::{
    error::{self, insert::InsertError, GlobalError},
    operations::{
        delete_one::DynDeleteWorker,
        insert_one::InsertOneWorker, select_many::GetAllWorker,
        update_one::UpdateOneWorker,
    },
    queries_bridge::{InsertSt, SelectSt, UpdateSt},
    relations::prelude::GetOneWorker,
};

lazy_static::lazy_static! {
    pub static ref COLLECTIONS: RwLock<HashMap<String, Box<dyn DynCollection>>> = {
        let mut map = HashMap::default();

        for collection in inventory::iter::<SubmitDynCollection> {
            let obj = (collection.obj)();
            map.insert(obj.table_name().to_owned(), obj);
        }

        return RwLock::new(map);
    };
    pub static ref RELATIONS: RwLock<HashMap<String, Vec<Arc<dyn CompleteRelationForServer>>>> = {
        let mut map = HashMap::default();

        for relation in inventory::iter::<SubmitDynRelation> {
            let obj = (relation.obj)();
            let key = obj.list_iteself_under();
            let ve: &mut Vec<_> = map.entry(key).or_default();
            ve.push(obj.clone())
        }

       tracing::debug!("{:?}", map);

        return RwLock::new(map);

    };
}

pub struct SubmitDynRelation {
    pub obj: fn() -> Arc<dyn CompleteRelationForServer>,
}

collect!(SubmitDynRelation);

pub enum DynamicRelationResult<T> {
    Ok(T),
    InvalidInput(String),
    NotFound,
}

pub trait CompleteRelationForServer:
    Send + Sync + 'static
{
    // CamelCase
    fn list_iteself_under(&self) -> String;
    // snake_case
    fn key(&self) -> String;
    fn init_on_update(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<Box<dyn DynUpdateOneWorker>>;
    fn init_on_insert(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<Box<dyn DynInsertOneWorker>>;
    fn init_on_get_all(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<Box<dyn DynGetManyWorker>>;
    fn init_on_delete(
        self: Arc<Self>,
        to: &str,
    ) -> DynamicRelationResult<Box<dyn DynDeleteWorker>> {
        DynamicRelationResult::NotFound
    }
    fn init_on_get(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<Box<dyn DynGetOneWorker>>;
}

pub trait DynUpdateOneWorker: Send + Sync {
    fn on_update(&mut self, st: &mut UpdateSt<Sqlite>);
    fn from_row(&mut self, row: &SqliteRow);
    fn sub_op1<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn sub_op2<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn take(&mut self) -> Value;
}

impl<RW> DynUpdateOneWorker for DynamicWorker<RW::Inner, RW>
where
    RW: Send + Sync,
    RW: UpdateOneWorker,
    RW::Output: Serialize,
{
    fn on_update(&mut self, st: &mut UpdateSt<Sqlite>) {
        RW::on_update(
            self.rw.as_ref().expect("should not be taken"),
            &mut self.inner,
            st,
        )
    }

    fn from_row(&mut self, row: &SqliteRow) {
        RW::from_row(
            self.rw.as_ref().expect("should not be taken"),
            &mut self.inner,
            row,
        );
    }

    fn sub_op1<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async {
            RW::sub_op1(
                self.rw.as_ref().expect("should not be taken"),
                &mut self.inner,
                pool,
            )
            .await
        })
    }

    fn sub_op2<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async {
            RW::sub_op2(
                self.rw.as_ref().expect("should not be taken"),
                &mut self.inner,
                pool,
            )
            .await
        })
    }

    fn take(&mut self) -> Value {
        let taken = RW::take(
            self.rw.take().expect("should not be taken"),
            mem::take(&mut self.inner),
        );
        serde_json::to_value(taken).unwrap()
    }
}

pub trait DynInsertOneWorker: Send + Sync {
    fn on_insert(&mut self, st: &mut InsertSt<Sqlite>);
    fn from_row(&mut self, row: &SqliteRow);
    fn sub_op1<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn sub_op2<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn take(&mut self) -> Value;
}

impl<RW> DynInsertOneWorker for DynamicWorker<RW::Inner, RW>
where
    RW: Send + Sync,
    RW: InsertOneWorker,
    RW::Output: Serialize,
{
    fn on_insert(&mut self, st: &mut InsertSt<Sqlite>) {
        RW::on_insert(
            self.rw.as_ref().expect("should not be taken"),
            &mut self.inner,
            st,
        )
    }

    fn from_row(&mut self, row: &SqliteRow) {
        RW::from_row(
            self.rw.as_ref().expect("should not be taken"),
            &mut self.inner,
            row,
        );
    }

    fn sub_op1<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async {
            RW::sub_op1(
                self.rw.as_ref().expect("should not be taken"),
                &mut self.inner,
                pool,
            )
            .await
        })
    }

    fn sub_op2<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async {
            RW::sub_op2(
                self.rw.as_ref().expect("should not be taken"),
                &mut self.inner,
                pool,
            )
            .await
        })
    }

    fn take(&mut self) -> Value {
        let taken = RW::take(
            self.rw.take().expect("should not be taken"),
            mem::take(&mut self.inner),
        );
        serde_json::to_value(taken).unwrap()
    }
}

impl fmt::Debug for dyn CompleteRelationForServer {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("complete_rel")
            .field(
                "list_itself_under",
                &self.list_iteself_under(),
            )
            .field("key", &self.key())
            .finish()
    }
}

pub trait DynGetManyWorker: Send + Sync {
    fn on_select(&mut self, st: &mut SelectSt<Sqlite>);
    fn from_row(&mut self, row: &SqliteRow);
    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn take(&mut self, current_id: i64) -> Value;
}
pub trait DynGetOneWorker: Send + Sync {
    fn on_select(&mut self, st: &mut SelectSt<Sqlite>);
    fn from_row(&mut self, row: &SqliteRow);
    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>>;
    fn take(&mut self) -> Value;
}

pub struct DynamicWorker<Inner, RW> {
    pub(crate) arc: Arc<dyn CompleteRelationForServer>,
    pub(crate) inner: Inner,
    pub(crate) rw: Option<RW>,
}

impl<I: Default, RW> DynamicWorker<I, RW> {
    pub fn new(
        arc: Arc<dyn CompleteRelationForServer>,
        worker: RW,
    ) -> Box<DynamicWorker<I, RW>> {
        Box::new(DynamicWorker {
            arc,
            inner: I::default(),
            rw: Some(worker),
        })
    }
}

impl<RW> DynGetOneWorker for DynamicWorker<RW::Inner, RW>
where
    RW: Send + Sync,
    RW: GetOneWorker,
    RW::Output: Serialize,
{
    fn on_select(&mut self, st: &mut SelectSt<Sqlite>) {
        RW::on_select(
            self.rw.as_ref().expect("should not be taken"),
            &mut self.inner,
            st,
        );
    }

    fn from_row(&mut self, row: &SqliteRow) {
        RW::from_row(
            self.rw.as_ref().expect("should not be taken"),
            &mut self.inner,
            row,
        );
    }

    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move {
            RW::sub_op(
                self.rw.as_ref().expect("should not be taken"),
                &mut self.inner,
                pool,
            )
            .await
        })
    }

    fn take(&mut self) -> Value {
        let taken = RW::take(
            self.rw.take().expect("should not be taken"),
            mem::take(&mut self.inner),
        );
        serde_json::to_value(taken).unwrap()
    }
}

impl<RW> DynGetManyWorker for DynamicWorker<RW::Inner, RW>
where
    RW: Send + Sync,
    RW: GetAllWorker,
    RW::Output: Serialize,
{
    fn on_select(&mut self, st: &mut SelectSt<Sqlite>) {
        RW::on_select(
            self.rw.as_ref().expect("should not be taken"),
            &mut self.inner,
            st,
        );
    }

    fn from_row(&mut self, row: &SqliteRow) {
        RW::from_row(
            self.rw.as_ref().expect("should not be taken"),
            &mut self.inner,
            row,
        );
    }

    fn sub_op<'this>(
        &'this mut self,
        pool: Pool<Sqlite>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'this>> {
        Box::pin(async move {
            RW::sub_op(
                self.rw.as_ref().expect("should not be taken"),
                &mut self.inner,
                pool,
            )
            .await
        })
    }

    fn take(&mut self, id: i64) -> Value {
        let taken = RW::take(
            self.rw.as_mut().expect("should not be taken"),
            id,
            &mut self.inner,
        );
        serde_json::to_value(taken).unwrap()
    }
}

pub trait DynCollection: Send + Sync + 'static {
    fn members_no_scope(&self) -> &'static [&'static str];
    fn table_name(&self) -> &str;
    // all scoped
    fn on_select(&self, stmt: &mut SelectSt<Sqlite>);
    // all scoped, no modification
    fn from_row_scoped(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Value;
    fn from_row_noscope(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Value;

    fn on_insert(
        &self,
        input: Value,
        stmt: &mut InsertSt<Sqlite>,
    ) -> Result<(), ValidatedAndTyped>;
    fn on_update(
        &self,
        input: Value,
        stmt: &mut UpdateSt<Sqlite>,
    ) -> Result<(), ValidatedAndTyped>;
}

impl fmt::Debug for dyn DynCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dyn_Collection")
            .field("table_name", &self.table_name())
            .finish_non_exhaustive()
    }
}

#[must_use]
pub enum ValidatedAndTyped {
    TypeError(serde_json::Error),
    ValidationError(String),
}

impl From<ValidatedAndTyped> for GlobalError {
    fn from(value: ValidatedAndTyped) -> Self {
        match value {
            ValidatedAndTyped::TypeError(err) => {
                error::to_refactor(&format!(
                    "input invalid: {}",
                    err
                ))
            }
            ValidatedAndTyped::ValidationError(err) => {
                error::to_refactor(&format!(
                    "input failed validation: {}",
                    err
                ))
            }
        }
    }
}

impl From<ValidatedAndTyped> for InsertError {
    fn from(value: ValidatedAndTyped) -> Self {
        <GlobalError as From<ValidatedAndTyped>>::from(value)
            .into()
    }
}

impl<T> DynCollection for PhantomData<T>
where
    T: Collection<Sqlite> + Serialize + 'static,
    T: DeserializeOwned,
    T::PartailCollection: DeserializeOwned,
{
    fn members_no_scope(&self) -> &'static [&'static str] {
        T::members()
    }
    fn on_update(
        &self,
        input: Value,
        stmt: &mut UpdateSt<Sqlite>,
    ) -> Result<(), ValidatedAndTyped> {
        let v = from_value::<T::PartailCollection>(input)
            .map_err(|e| ValidatedAndTyped::TypeError(e))?;

        T::on_update(stmt, v).map_err(|e| {
            ValidatedAndTyped::ValidationError(e)
        })?;

        Ok(())
    }

    fn on_insert(
        &self,
        input: Value,
        stmt: &mut InsertSt<Sqlite>,
    ) -> Result<(), ValidatedAndTyped> {
        let v = from_value::<T>(input)
            .map_err(|e| ValidatedAndTyped::TypeError(e))?;

        T::on_insert(v, stmt).map_err(|e| {
            ValidatedAndTyped::ValidationError(e)
        })?;

        Ok(())
    }
    fn table_name(&self) -> &str {
        T::table_name()
    }

    fn on_select(&self, stmt: &mut SelectSt<Sqlite>) {
        T::on_select(stmt)
    }
    fn from_row_scoped(&self, row: &SqliteRow) -> Value {
        let t = T::from_row_scoped(row);
        serde_json::to_value(t).unwrap()
    }
    fn from_row_noscope(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Value {
        let t = T::from_row_noscope(row);
        serde_json::to_value(t).unwrap()
    }
}

pub struct SubmitDynCollection {
    pub obj: fn() -> Box<dyn DynCollection>,
}

collect!(SubmitDynCollection);
