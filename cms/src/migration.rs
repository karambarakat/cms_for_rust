use crate::utils::eventfull_list::Context;
use crate::utils::eventfull_list::EventType;
use crate::utils::eventfull_list::EventfulList;
use inventory::Collect;
use inventory::Registry;
use std::collections::HashMap;

pub mod submit_migration_prelude {
    pub use crate::entities::EntityPhantom;
    pub use crate::migration::Submitable;
    pub use core::marker::PhantomData;
    pub use inventory::submit;
    pub use std::boxed::Box;
}

use queries_for_sqlx::{
    create_table_st::CreateTableSt, prelude::*,
    quick_query::QuickQuery, Query, SupportNamedBind,
};
use sqlx::{Database, Executor, Pool};

pub mod migrations_impl {
    use std::ops::Not;

    use crate::utils::eventfull_list::{Context, EventfulList};
    use queries_for_sqlx::{
        prelude::{stmt, Fk},
        SupportNamedBind,
    };
    use sqlx::{Database, Type};

    use crate::{
        entities::Entity,
        // migration::{DynMigration, TableContext},
        entities::EntityPhantom,
        migration::TableContext,
        relations::{
            many_to_many::ManyToMany,
            optional_to_many::OptionalToMany,
        },
    };

    impl<S, T> EventfulList<TableContext<S>> for EntityPhantom<T>
    where
        S: Database + Sync + Send + SupportNamedBind,
        T: Entity<S> + Send + Sync + 'static,
    {
        fn run(
            &self,
            ctx: &mut Context<TableContext<S>>,
        ) -> Result<(), &'static str> {
            let table_name = T::table_name();
            if ctx.has_event_occured(table_name) {
                let found = ctx.map.get_mut(table_name).unwrap();
                T::migrate(found)
            } else {
                let mut new =
                    stmt::create_table_if_not_exists(table_name);
                T::migrate(&mut new);
                ctx.map.insert(table_name.to_string(), new);
                ctx.event(table_name);
            }

            Ok(())
        }
    }

    impl<S, O, M> EventfulList<TableContext<S>> for ManyToMany<O, M>
    where
        S: Database
            + Sync
            + Send
            + SupportNamedBind
            + queries_for_sqlx::create_table_st::SqlxQuery,
        O: Send + Sync + 'static + Entity<S>,
        M: Send + Sync + 'static + Entity<S>,
        i64: Type<S>,
    {
        fn run(
            &self,
            ctx: &mut Context<TableContext<S>>,
        ) -> Result<(), &'static str> {
            let name = self.conj_table;
            if ctx.has_event_occured(name).not() {
                let new = stmt::create_table_if_not_exists(name);

                ctx.map.insert(name.to_string(), new);
                ctx.event(name);
            }

            let table = ctx.map.get_mut(name).unwrap();

            table.foreign_key(Fk {
                not_null: false,
                column: self.rel_fk,
                refer_table: self.rel_t,
                refer_column: "id",
            });

            table.foreign_key(Fk {
                not_null: false,
                column: self.base_fk,
                refer_table: self.base_t,
                refer_column: "id",
            });

            Ok(())
        }
    }

    impl<S, O, M> EventfulList<TableContext<S>>
        for OptionalToMany<O, M>
    where
        S: Database
            + Sync
            + Send
            + SupportNamedBind
            + queries_for_sqlx::create_table_st::SqlxQuery,
        O: Send + Sync + 'static + Entity<S>,
        M: Send + Sync + 'static + Entity<S>,
    {
        fn run(
            &self,
            ctx: &mut Context<TableContext<S>>,
        ) -> Result<(), &'static str> {
            let name = O::table_name();
            if ctx.has_event_occured(name).not() {
                let new = stmt::create_table_if_not_exists(name);

                ctx.map.insert(name.to_string(), new);
                ctx.event(name);
            }

            let table = ctx.map.get_mut(name).unwrap();

            table.foreign_key(Fk {
                not_null: false,
                column: self.0,
                refer_table: M::table_name(),
                refer_column: "id",
            });

            Ok(())
        }
    }
}

pub struct Submitable<S: Database + SupportNamedBind> {
    pub object: fn() -> Box<dyn EventfulList<TableContext<S>>>,
}

impl<S: Database + SupportNamedBind> Collect for Submitable<S> {
    fn registry() -> &'static Registry {
        static REGISTRY: Registry = Registry::new();
        &REGISTRY
    }
}

pub struct TableContext<DB>
where
    DB: Database,
    for<'q> QuickQuery<'q>: Query<DB>,
{
    pub(crate) map: HashMap<
        String,
        CreateTableSt<'static, DB, QuickQuery<'static>>,
    >,
}

impl<S: Database + SupportNamedBind> EventType
    for TableContext<S>
{
    type Event = &'static str;
}

pub async fn migrate<DB>(db: Pool<DB>)
where
    DB: Sync + Send + Database + SupportNamedBind + 'static,
    for<'c> &'c mut <DB as Database>::Connection:
        Executor<'c, Database = DB>,
{
    let mut map = TableContext {
        map: Default::default(),
    };

    let mut submitables = inventory::iter::<Submitable<DB>>
        .into_iter()
        .map(|e| (e.object)())
        .collect::<Vec<_>>();

    let mut points = vec![];

    while let Some(item) = submitables.pop() {
        item.run(&mut Context::new(
            &mut points,
            &mut submitables,
            &mut map,
        ))
        .expect("failed to run migration on");
    }

    for (table, each) in map.map {
        each.execute(&db).await.expect(&format!(
            "failed to run migration on {}",
            table
        ));
    }
}
