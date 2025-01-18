use std::{
    collections::HashMap, error::Error, marker::PhantomData,
};

use case::CaseExt;
use inventory::collect;
use queries_for_sqlx::{
    clonable_query::ClonablQuery,
    create_table_st::{CreateTableHeader, CreateTableSt},
    prelude::ExecuteNoCache,
};
use sqlx::{Pool, Sqlite};

use crate::traits::Collection;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Events {
    TableCreated(&'static str),
}

pub type CreatTableSt<S> =
    queries_for_sqlx::create_table_st::CreateTableSt<
        S,
        ClonablQuery<'static, S>,
    >;

pub(crate) struct Store(pub(crate) HashMap<String, CreatTableSt<Sqlite>>);

pub struct MigrationCtx<'l> {
    pub(crate) events: &'l mut Vec<Events>,
    pub(crate) executables: &'l mut Vec<Box<dyn DynMigration>>,
    pub(crate) store: &'l mut Store,
}

pub trait DynMigration {
    fn migrate(
        &self,
        ctx: &mut MigrationCtx,
    ) -> Result<(), String>;
}

impl<T> DynMigration for PhantomData<T>
where
    T: Collection<Sqlite>,
{
    fn migrate(
        &self,
        ctx: &mut MigrationCtx,
    ) -> Result<(), String> {
        let table_name = T::table_name();
        if ctx.events.contains(&Events::TableCreated(table_name))
        {
            let found = ctx.store.0.get_mut(table_name).unwrap();
            T::on_migrate(found)
        } else {
            let mut new = CreateTableSt::init((
                CreateTableHeader::IfNotExists,
                table_name,
            ));
            T::on_migrate(&mut new);
            ctx.store.0.insert(table_name.to_string(), new);
            ctx.events.push(Events::TableCreated(table_name));
        }

        Ok(())
    }
}

pub struct SubmitDynMigrate {
    pub obj: fn() -> Box<dyn DynMigration>,
}

collect!(SubmitDynMigrate);

pub async fn run_migration(
    db: Pool<Sqlite>,
) -> Result<(), Box<dyn Error>> {
    let mut store = Store(Default::default());

    let mut execs = inventory::iter::<SubmitDynMigrate>
        .into_iter()
        .map(|e| ((e.obj)()))
        .collect::<Vec<_>>();

    let mut events: Vec<Events> = vec![];

    while let Some(item) = execs.pop() {
        item.migrate(&mut MigrationCtx {
            events: &mut events,
            executables: &mut execs,
            store: &mut store,
        })?
    }

    for (name, table) in store.0 {
        table.execute(&db).await.map_err(|e| {
            format!("failed to run migration on {}: {}", name, e)
        })?;
    }

    Ok(())
}
