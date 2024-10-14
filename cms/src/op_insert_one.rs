#![allow(unused)]

use core::fmt;
use serde::Deserialize;
use std::ops::Not;

use either::IntoEither;
use futures_util::future::Either;
use insert_relation::{InsertRelation, InsertRelationTuple};
use query::executable::InnerExecutable;
use query::IntoMutArguments;
use serde_json::{json, Value};
use sqlx::{prelude::*, Arguments};
use sqlx::{Database, Executor};
use tracing::{debug, error};

use crate::{Entity, OpError};

// export OptionalToMany
pub use insert_relation::impl_dyn_migrate::relations_submitable::OptionalToMany;

#[derive(Debug, PartialEq, Eq)]
pub struct Output<E, R, I> {
    pub id: i64,
    pub data: E,
    pub relations: R,
    pub info: I,
}

pub mod submit_relation_prelude {
    pub use super::insert_relation::dynamic_trait::Submitable;
    pub use crate::expr_mod::*;
    pub use core::marker::PhantomData;
    pub use inventory::submit;
    pub use std::boxed::Box;
}

//     cms::migration::Submitable::<CMS_DB> {
//         object: || { Box::new(
//             OptionalToMany
//                 ::<Log, Category>
//                 ::colomn_link("category_id"))
//         },
//     }
// }

pub mod prelude {
    pub use super::insert_relation::InsertRelation;
    pub use crate::expr_mod::*;
    pub use serde_json::json;
    pub use serde_json::Map;
    pub use serde_json::Value;
    pub use sqlx::database::HasArguments;
    pub use sqlx::prelude::*;
    pub use sqlx::Arguments;
    pub use sqlx::Database;
    pub use sqlx::Row;
}

pub mod insert_relation {
    use std::fmt::Arguments;

    use query::SupportNamedBind;
    use serde_json::{Map, Value};
    use sqlx::{database::HasArguments, Database};
    use tracing::error;

    use crate::entity_mod::impl_prelude::MigrateArg;

    pub trait InsertRelation<Base, S> {
        type ThisEntity;
        type Output;
        fn foriegn_key() -> &'static str;
        fn foriegn_table() -> &'static str;
        fn on_input(v: &mut Vec<&'static str>) {
            error!("depricate on_input");
            // v.extend(Self::members());
            v.push(Self::foriegn_key());
        }
        fn on_output(v: &mut Vec<&'static str>);
        fn on_buffer<'e>(
            self,
            buffer: &mut <S as HasArguments<'e>>::Arguments,
        ) where
            S: Database;
        fn dyn_id_related_snake_case(base: &str) -> bool {
            todo!()
            // base.eq(Base::table_name())
        }
        fn dyn_is_related<'a>(
            relation: &'a Map<String, Value>,
        ) -> Option<(&'a str, &'a Value)>;
        fn on_buffer_input<'e>(
            value: &Map<String, Value>,
            output: &mut Map<String, Value>,
            buffer: &mut <S as HasArguments<'e>>::Arguments,
        ) where
            S: Database;
    }

    pub mod dynamic_trait {
        use std::any::Any;
        use std::marker::PhantomData;
        use std::ops::Not;

        use case::CaseExt;
        use futures_util::Future;
        use inventory::Collect;
        use inventory::Registry;
        use query::insert_one_st::InsertStOne;
        use query::prelude::col;
        use query::quick_query::QuickQuery;
        use query::select_st::SelectSt;
        use query::update_st::UpdateSt;
        use query::SupportNamedBind;
        use serde::Deserialize;
        use serde_json::json;
        use serde_json::Map;
        use serde_json::Value;
        use sqlx::prelude::*;
        use sqlx::ColumnIndex;
        use sqlx::{database::HasArguments, Database};
        use tracing::error;

        use crate::entity_mod::DynEntity;
        use crate::entity_mod::DynEntitySubmitable;
        use crate::relations_v2::StatufullRelation;
        use crate::Entity;

        use super::InsertRelation;

        pub trait PopulateRelated<S>: Send {
            fn on_select(
                &self,
                stmt: &mut SelectSt<S, QuickQuery<'_>>,
            ) where
                S: Database,
                S: SupportNamedBind,
            {
                todo!()
            }
            fn sub_op<'this>(
                &'this mut self,
            ) -> std::pin::Pin<
                Box<
                    dyn Future<Output = Result<(), sqlx::Error>>
                        + 'this
                        + Send,
                >,
            > {
                Box::pin(async move { todo!() })
            }
            fn on_row(
                &mut self,
                row: &S::Row,
            ) -> Result<(), sqlx::Error>
            where
                S: Database,
            {
                todo!()
            }
        }

        pub struct PopulateOnSelect<'v, S> {
            f_table: &'static str,
            rel_key: &'static str,
            members: Vec<&'static str>,
            input: &'v Value,
            output: &'v mut Value,
            dyn_entity: Box<dyn DynEntity<S>>,
            ph: PhantomData<S>,
        }

        unsafe impl<'v, S> Send for PopulateOnSelect<'v, S> {}

        impl<'v, S> PopulateRelated<S> for PopulateOnSelect<'v, S>
        where
            for<'r> &'r str: ColumnIndex<S::Row>,
            S: Database + SupportNamedBind,
            Option<i64>: for<'e> Decode<'e, S> + Type<S>,
        {
            fn sub_op<'this>(
                &'this mut self,
            ) -> std::pin::Pin<
                Box<
                    dyn Future<Output = Result<(), sqlx::Error>>
                        + 'this
                        + Send,
                >,
            > {
                Box::pin(async move {
                    return Ok(());
                })
            }
            fn on_select(
                &self,
                stmt: &mut SelectSt<S, QuickQuery<'_>>,
            ) {
                use query::prelude::*;
                stmt.select(col(self.rel_key));
                stmt.join(Join {
                    ty: join_type::Left,
                    on_table: self.f_table,
                    on_column: "id",
                    local_column: self.rel_key,
                });
                // panic!("{}",self.f_table);
                for member in &self.members {
                    stmt.select(ft(self.f_table).col(member))
                }
            }
            fn on_row(
                &mut self,
                row: &<S>::Row,
            ) -> Result<(), sqlx::Error>
            where
                S: Database,
            {
                let id: Option<i64> = row.get(self.rel_key);
                // S::decode_option_i64(row, self.rel_key);

                if let Some(id) = id {
                    *self.output = json!({
                        "id": id,
                        "data": null,
                    });

                    self.dyn_entity.from_row(
                        row,
                        &mut self
                            .output
                            .get_mut("data")
                            .unwrap(),
                    );
                }

                Ok(())
            }
        }

        struct InsertPopulate<'v, S> {
            id: i64,
            output: &'v mut Value,
            relation_key: &'static str,
            members: Vec<&'static str>,
            foreign_table: &'static str,
            dyn_entity: Box<dyn DynEntity<S>>,
        }

        impl<S> InitInsert<S> for InsertPopulate<'_, S>
        where
            S: Database + SupportNamedBind,
            i64: for<'d> Encode<'d, S> + Type<S>,
            for<'c> &'c mut S::Connection:
                Executor<'c, Database = S>,
        {
            fn on_insert(&self, stmt: &mut InsertStOne<S>) {
                stmt.insert(self.relation_key, self.id);
            }
            fn after_insert(&mut self) {
                *self.output = json!({
                    "id": self.id,
                    "data": null,
                });
            }
            fn sub_op<'this>(
                &'this mut self,
                pool: Pool<S>,
            ) -> std::pin::Pin<
                Box<
                    dyn Future<Output = Result<(), sqlx::Error>>
                        + 'this
                        + Send,
                >,
            > {
                use query::prelude::*;
                Box::pin(async move {
                    let mut st =
                        stmt::select(self.foreign_table);

                    for member in &self.members {
                        st.select(col(member));
                    }

                    let id = self.id;
                    st.where_(col("id").eq(move || id));

                    let res = st
                        .build()
                        .fetch_one(&pool, |row| {
                            self.dyn_entity.from_row(
                                &row,
                                &mut self
                                    .output
                                    .get_mut("data")
                                    .unwrap(),
                            );
                            Ok(())
                        })
                        .await;
                    Ok(())
                })
            }
        }

        struct InitInsertImplNoPopulate<'v> {
            id: i64,
            output: &'v mut Value,
            relation_key: &'static str,
        }
        use sqlx::Pool;

        pub trait InitInsert<S>
        where
            Self: Send,
            S: Database,
        {
            fn on_insert(&self, stmt: &mut InsertStOne<S>);
            fn after_insert(&mut self);
            fn sub_op<'this>(
                &'this mut self,
                pool: Pool<S>,
            ) -> std::pin::Pin<
                Box<
                    dyn Future<Output = Result<(), sqlx::Error>>
                        + 'this
                        + Send,
                >,
            > {
                // default to no-op
                Box::pin(async { Ok(()) })
            }
        }

        impl<'v, S> InitUpdate<S> for InsertPopulate<'v, S>
        where
            S: Database + SupportNamedBind,
            i64: for<'d> Encode<'d, S> + Type<S>,
            for<'c> &'c mut S::Connection:
                Executor<'c, Database = S>,
        {
            fn on_update(
                &self,
                stmt: &mut UpdateSt<S, QuickQuery<'_>>,
            ) {
                let id = self.id;
                stmt.set(self.foreign_table, move || id);
            }
            fn after_update(
                &mut self,
                row: &<S as Database>::Row,
            ) {
                *self.output = json!({
                    "id": self.id,
                    "data": null,
                });
            }
            fn sub_op<'this>(
                &'this mut self,
                pool: Pool<S>,
            ) -> std::pin::Pin<
                Box<
                    dyn Future<Output = Result<(), sqlx::Error>>
                        + 'this
                        + Send,
                >,
            > {
                use query::prelude::*;
                Box::pin(async move {
                    let mut st =
                        stmt::select(self.foreign_table);

                    for member in &self.members {
                        st.select(col(member));
                    }

                    let id = self.id;
                    st.where_(col("id").eq(move || id));

                    let res = st
                        .build()
                        .fetch_one(&pool, |row| {
                            self.dyn_entity.from_row(
                                &row,
                                &mut self
                                    .output
                                    .get_mut("data")
                                    .unwrap(),
                            );
                            Ok(())
                        })
                        .await;
                    Ok(())
                })
            }
        }

        impl<'v, S> InitUpdate<S> for InitInsertImplNoPopulate<'v>
        where
            S: Database + SupportNamedBind,
            i64: for<'d> Encode<'d, S> + Type<S>,
        {
            fn on_update(
                &self,
                stmt: &mut UpdateSt<S, QuickQuery<'_>>,
            ) {
                let id = self.id;
                stmt.set(self.relation_key, move || id);
            }
            fn after_update(&mut self, row: &S::Row) {
                *self.output = json!({
                    "id": self.id,
                    "data": null,
                });
            }
        }

        impl<'v, S> InitInsert<S> for InitInsertImplNoPopulate<'v>
        where
            S: Database,
            i64: for<'d> Encode<'d, S> + Type<S>,
        {
            fn on_insert(&self, stmt: &mut InsertStOne<S>) {
                stmt.insert(self.relation_key, self.id);
            }
            fn after_insert(&mut self) {
                *self.output = json!({
                    "id": self.id,
                });
            }
        }

        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct InsertRelationObj<P> {
            id: i64,
            populate: P,
        }

        pub trait InitUpdate<S>
        where
            Self: Send,
            S: Database + SupportNamedBind,
        {
            fn on_update(
                &self,
                stmt: &mut UpdateSt<S, QuickQuery<'_>>,
            );
            fn after_update(&mut self, row: &S::Row);
            fn sub_op<'this>(
                &'this mut self,
                pool: Pool<S>,
            ) -> std::pin::Pin<
                Box<
                    dyn Future<Output = Result<(), sqlx::Error>>
                        + 'this
                        + Send,
                >,
            > {
                // default to no-op
                Box::pin(async { Ok(()) })
            }
        }

        pub trait DynInsertRelation<S>: Send + 'static {
            fn alt_obj(
                &self,
                from_many: &'static str,
                value: Value,
            ) -> Box<dyn StatufullRelation<S>> {
                panic!("depricate")
            }
            fn members(&self) -> Vec<&'static str>;
            fn foriegn_entity(&self) -> Box<dyn DynEntity<S>>;
            fn foriegn_table(&self) -> &'static str;
            fn init_update<'v>(
                &self,
                data: &'v Value,
                output: &'v mut Value,
            ) -> Result<Box<dyn InitUpdate<S> + 'v>, &'static str>
            where
                S: Database + SupportNamedBind,
                for<'r> &'r str: ColumnIndex<S::Row>,
                i64: for<'d> Encode<'d, S> + Type<S>,
                for<'c> &'c mut S::Connection:
                    Executor<'c, Database = S>,
            {
                let vl =
                    InsertRelationObj::<Option<&'_ str>>::deserialize(
                        data,
                    )
                    .unwrap();

                match vl.populate {
                    Some(input) => {
                        if input.eq("populate_one_level") {
                            Ok(Box::new(InsertPopulate {
                                id: vl.id,
                                output,
                                relation_key: self.foriegn_key(),
                                members: self.members(),
                                foreign_table: self
                                    .foriegn_table(),
                                dyn_entity: self
                                    .foriegn_entity(),
                            }))
                        } else {
                            Err("only populate_one_level is supported for now")
                        }
                    }
                    None => {
                        Ok(Box::new(InitInsertImplNoPopulate {
                            relation_key: self.foriegn_key(),
                            id: vl.id,
                            output,
                        }))
                    }
                }
            }
            fn init_insert<'v>(
                &self,
                data: &'v Value,
                output: &'v mut Value,
            ) -> Result<Box<dyn InitInsert<S> + 'v>, &'static str>
            where
                S: Database + SupportNamedBind,
                for<'r> &'r str: ColumnIndex<S::Row>,
                i64: for<'d> Encode<'d, S> + Type<S>,
                for<'c> &'c mut S::Connection:
                    Executor<'c, Database = S>,
            {
                let vl =
                    InsertRelationObj::<Option<&'_ str>>::deserialize(
                        data,
                    )
                    .unwrap();

                match vl.populate {
                    Some(input) => {
                        if input.eq("populate_one_level") {
                            Ok(Box::new(InsertPopulate {
                                id: vl.id,
                                output,
                                relation_key: self.foriegn_key(),
                                members: self.members(),
                                foreign_table: self
                                    .foriegn_table(),
                                dyn_entity: self
                                    .foriegn_entity(),
                            }))
                        } else {
                            Err("only populate_one_level is supported for now")
                        }
                    }
                    None => {
                        Ok(Box::new(InitInsertImplNoPopulate {
                            relation_key: self.foriegn_key(),
                            id: vl.id,
                            output,
                        }))
                    }
                }
            }
            fn init_select<'v>(
                &self,
                input: &'v Value,
                output: &'v mut Value,
            ) -> Result<
                // todo: should I just return PopulateOnSelect?
                Box<dyn PopulateRelated<S> + 'v>,
                &'static str,
            >
            where
                S: Database + SupportNamedBind,
                for<'r> &'r str: ColumnIndex<S::Row>,
                i64: Type<S> + for<'d> Decode<'d, S>,
            {
                let res = if let Value::String(str) = input {
                    if str.eq("populate_one_level").not() {
                        None
                    } else {
                        Some(Box::new(PopulateOnSelect {
                            rel_key: self.foriegn_key(),
                            output,
                            input,
                            members: self.members(),
                            f_table: self.foriegn_table(),
                            dyn_entity: self.foriegn_entity(),
                            ph: PhantomData,
                        }))
                    }
                } else {
                    None
                };

                match res {
                    Some(e) => Ok(e),
                    None => Err("only populate_one_level is supported for now")
                }
            }
            fn on_select(
                &self,
                stmt: &mut SelectSt<S, QuickQuery<'_>>,
            ) where
                S: Database,
                S: SupportNamedBind,
            {
                use query::prelude::*;
                let mut this = vec![];
                self.on_input(&mut this);
                for each in this {
                    stmt.select(col(each));
                }
            }
            fn foriegn_key(&self) -> &'static str;
            fn on_input(&self, v: &mut Vec<&'static str>);
            fn on_output(&self, v: &mut Vec<&'static str>);
            fn is_related<'a>(
                &self,
                entity: &str,
                relation_key: &'a Map<String, Value>,
            ) -> Option<(&'a str, &'a Value)>;
            fn is_related2(&self, entity: &str) -> Option<&str>;
            fn on_buffer_input<'e>(
                &mut self,
                value: &Map<String, Value>,
                output: &mut Map<String, Value>,
                buffer: &mut <S as HasArguments<'e>>::Arguments,
            ) where
                S: Database;
        }

        impl<S, Base, T> DynInsertRelation<S> for PhantomData<(Base, T)>
        where
            Self: Send + 'static,
            T: InsertRelation<Base, S> + Clone,
            S: Database,
            i64: for<'d> Encode<'d, S> + Type<S>,
            T::ThisEntity: Entity<S>,
            // T: DynEntity<S>,
            Base: Entity<S>,
        {
            fn is_related2(&self, entity: &str) -> Option<&str> {
                if entity.eq(&Base::table_name().to_snake()) {
                    Some(T::ThisEntity::table_name())
                } else {
                    None
                }
            }
            fn foriegn_entity(&self) -> Box<dyn DynEntity<S>> {
                for entity in
                    inventory::iter::<DynEntitySubmitable<S>>
                {
                    let entity = (entity.object)();
                    if entity
                        .table_name()
                        .eq(self.foriegn_table())
                    {
                        return entity;
                    }
                }
                panic!(
                    "did not find entity {}",
                    self.foriegn_table()
                )
            }
            fn members(&self) -> Vec<&'static str> {
                <T::ThisEntity as Entity<S>>::members()
            }
            fn on_input(&self, v: &mut Vec<&'static str>) {
                T::on_input(v);
            }

            fn on_output(&self, v: &mut Vec<&'static str>) {
                T::on_output(v);
            }

            fn foriegn_table(&self) -> &'static str {
                T::foriegn_table()
            }
            fn foriegn_key(&self) -> &'static str {
                T::foriegn_key()
            }
            // fn is_related2(&self, entity: &str) -> bool {
            //     entity.eq(Base::table_name())
            // }
            fn is_related<'a>(
                &self,
                base_entity: &str,
                relation_key: &'a Map<String, Value>,
            ) -> Option<(&'a str, &'a Value)> {
                if base_entity != Base::table_name() {
                    return None;
                }

                T::dyn_is_related(relation_key)
            }

            fn on_buffer_input<'e>(
                &mut self,
                value: &serde_json::Map<String, Value>,
                output: &mut serde_json::Map<String, Value>,
                buffer: &mut <S as HasArguments<'e>>::Arguments,
            ) where
                S: Database,
            {
                T::on_buffer_input(value, output, buffer)
            }
        }

        pub struct Submitable<S> {
            pub object: fn() -> Box<dyn DynInsertRelation<S>>,
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
    }

    pub trait InsertRelationTuple<E, S> {
        type Output;
        fn on_input(v: &mut Vec<&'static str>);
        fn on_output(v: &mut Vec<&'static str>);
        fn on_buffer<'e>(
            self,
            buffer: &mut <S as HasArguments<'e>>::Arguments,
        ) where
            S: Database;
    }

    #[rustfmt::skip]
    pub mod impl_insert_relation_tuple {
        use super::InsertRelation;
        use super::InsertRelationTuple;
        use crate::Entity;
        use sqlx::{database::HasArguments, Database};

        macro_rules! impl_trait {
            ($([$tt:tt, $ident:ident]),*) => {
                impl<$($ident,)* S, E> InsertRelationTuple<E, S> for ($($ident,)*)
                where
                    S: Database,
                    E: Entity<S>,
                    $($ident: InsertRelation<E, S>,)*
                {
                    type Output = ($($ident::Output,)*);
                    fn on_input(v: &mut Vec<&'static str>) {
                        $(
                            $ident::on_input(v);
                        )*
                    }

                    fn on_output(v: &mut Vec<&'static str>) {
                        $(
                            $ident::on_output(v);
                        )*
                    }

                    fn on_buffer<'e>(
                        self,
                        buffer: &mut <S as HasArguments<'e>>::Arguments,
                    ) where
                        S: Database
                    {
                       // use paste::paste macro
                        $(
                            paste::paste! {
                                self.$tt.on_buffer(buffer);
                            }
                        )*
                    }
                }
            }
        }

        impl<S, Any> InsertRelationTuple<Any, S> for () {
            type Output = ();
            fn on_buffer<'e>(
                self,
                buffer: &mut <S as HasArguments<'e>>::Arguments,
            ) where
                S: Database,
            {
            }
            fn on_input(v: &mut Vec<&'static str>) {}
            fn on_output(v: &mut Vec<&'static str>) {}
        }

        impl_trait!([0, A0]);
        impl_trait!([0, A0], [1, A1]);
        impl_trait!([0, A0], [1, A1], [2, A2]);
        impl_trait!([0, A0], [1, A1], [2, A2], [3, A3]);
        impl_trait!([0, A0], [1, A1], [2, A2], [3, A3], [4, A4]);
        impl_trait!([0, A0], [1, A1], [2, A2], [3, A3], [4, A4], [5, A5]);
    }

    pub mod impl_dyn_migrate {
        pub mod relations_submitable {
            use std::{marker::PhantomData, ops::Not};

            use query::{
                create_table_st::CreateTableSt,
                prelude::{stmt, Fk},
                quick_query::QuickQuery,
                SqlxQuery, SupportNamedBind,
            };
            use sqlx::{Database, Type};
            use utils::ilist::{
                Context, EventfulList, IListError,
            };

            use crate::{
                // migration::{DynMigration, TableContext},
                migration::TableContext,
                relations_v2::relation_types::ManyToMany,
                Entity,
            };

            pub struct OptionalToMany<O, M>(
                &'static str,
                PhantomData<(O, M)>,
            );


            impl<S> TableContext<S>
            where
                S: Database + SupportNamedBind,
            {
                pub fn create_table_if_not_exist<'c>(
                    this: &'c mut Context<Self>,
                    name: &'static str,
                ) {
                }
            }

            impl<S, O, M> EventfulList<TableContext<S>> for ManyToMany<O, M>
            where
                S: Database
                    + Sync
                    + Send
                    + SupportNamedBind
                    + SqlxQuery,
                O: Send + Sync + 'static + Entity<S>,
                M: Send + Sync + 'static + Entity<S>,
                i64: Type<S>,
            {
                fn run(
                    &self,
                    ctx: &mut Context<TableContext<S>>,
                ) -> Result<(), IListError> {
                    let name = self.conj_table;
                    if ctx.has_event_occured(name).not() {
                        let mut new =
                            stmt::create_table_if_not_exists(
                                name,
                            );

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
                    + SqlxQuery,
                O: Send + Sync + 'static + Entity<S>,
                M: Send + Sync + 'static + Entity<S>,
            {
                // fn migrate(
                //     &self,
                // ) -> Box<dyn EventfulList<TableContext<S>>>
                // {
                //     struct InternalImpl<O, M>(
                //         &'static str,
                //         PhantomData<(O, M)>,
                //     );
                //     impl<S, O, M> EventfulList<TableContext<S>>
                //         for InternalImpl<O, M>
                //     where
                //         O: Entity<S>,
                //         M: Entity<S>,
                //         S: Database
                //             + SupportNamedBind
                //             + Send
                //             + Sync
                //             + SqlxQuery,
                //     {
                fn run(
                    &self,
                    ctx: &mut Context<TableContext<S>>,
                ) -> Result<(), IListError> {
                    let name = O::table_name();
                    if ctx.has_event_occured(name).not() {
                        let mut new =
                            stmt::create_table_if_not_exists(
                                name,
                            );

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
                //     }
                //
                //     Box::new(InternalImpl(
                //         self.0,
                //         PhantomData::<(O, M)>,
                //     ))
                // }
            }
        }
    }
}

#[deprecated]
pub async fn insert_one_json<S, Se>(
    input: Value,
    exec: Se,
) -> Result<Value, OpError>
where
    S: Database,
    Se: for<'e> Executor<'e, Database = S>,
    for<'s> &'s str: sqlx::ColumnIndex<S::Row>,
    for<'d> i64: Decode<'d, S> + Type<S>,
    for<'d> i64: sqlx::Encode<'d, S>,
{
    use crate::entity_mod::DynEntitySubmitable as submited_entities;

    let entity_name = input
        .get("entity")
        .ok_or_else(|| "entity field is required")?
        .as_str()
        .ok_or_else(|| "entity field must be a string")?;

    let entity_obj = inventory::iter::<submited_entities<S>>
        .into_iter()
        .find_map(|entity| {
            let obj = (entity.object)();
            if entity_name == obj.table_name() {
                Some(obj)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            format!("entity {} not found", entity_name)
        })?;

    let relations = input
        .get("relations")
        .ok_or_else(|| "relations field is required")?
        .as_object()
        .ok_or_else(|| "relations field must be an object")?;

    use crate::op_insert_one::insert_relation::dynamic_trait::Submitable as submited_relations;

    let mut relations_keys =
        relations.keys().collect::<Vec<_>>();

    let mut related = inventory::iter::<submited_relations<S>>
        .into_iter()
        .filter_map(|relation| {
            let obj = (relation.object)();
            if let Some(f) =
                obj.is_related(entity_name, relations)
            {
                relations_keys.retain(|k| k.eq(&f.0).not());
                Some((obj, f.1))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for key in relations_keys {
        return Err(format!(
            "relation {} not found for entity {}",
            key, entity_name
        )
        .into());
    }

    let inert_st = InsertSt4 {
        input: {
            let mut v = entity_obj.members();
            for (relation, _) in &related {
                relation.on_input(&mut v);
            }
            v
        },
        output: {
            let mut v = vec!["id"];
            v.extend(entity_obj.members());
            for (relation, _) in &related {
                relation.on_output(&mut v);
            }
            Some(v)
        },
        column: 1,
        from: entity_name,
    }
    .build();

    let mut res = InnerExecutable {
        stmt: inert_st.as_str(),
        buffer: Default::default(),
        persistent: false,
    };

    let mut output = json!({
        "data": {},
        "relations": {}
    });

    entity_obj.into_buffer(
        &input.get("data").unwrap(),
        &mut res.buffer,
    );

    for (mut relation, value) in related {
        relation.on_buffer_input(
            &relations,
            &mut output
                .get_mut("relations")
                .unwrap()
                .as_object_mut()
                .unwrap(),
            &mut res.buffer,
        );
    }

    let res = res.fetch_one_row(exec).await?;

    let id: i64 = res.get("id");

    output
        .as_object_mut()
        .unwrap()
        .insert("id".to_string(), id.into());

    entity_obj.from_row(
        &res,
        &mut output
            .as_object_mut()
            .unwrap()
            .get_mut("data")
            .unwrap(),
    );

    Ok(output)
}

#[deprecated]
pub async fn insert_one<S, Se, E, R, I>(
    entity: E,
    relations: R,
    info: I,
    exec: Se,
) -> Result<Output<E, R::Output, I>, Either<sqlx::Error, String>>
where
    S: Database,
    Se: for<'e> Executor<'e, Database = S>,
    E: Entity<S> + for<'e> IntoMutArguments<'e, S> + Clone,
    R: InsertRelationTuple<E, S> + Clone,
    // temp hack, I might need to figure out a way to extreact output from anoter sql query
    R: InsertRelationTuple<E, S, Output = R>,
    for<'s> &'s str: sqlx::ColumnIndex<S::Row>,
    for<'d> i64: Decode<'d, S> + Type<S>,
{
    let insert_st = InsertSt3 {
        input: {
            let mut v = E::members();

            R::on_input(&mut v);

            v
        },
        output: Some({
            let mut v = vec!["id"];

            R::on_output(&mut v);

            v
        }),
        column: 1,
        from: E::table_name(),
    }
    .build();

    let mut res = InnerExecutable {
        stmt: insert_st.as_str(),
        buffer: Default::default(),
        persistent: false,
    };

    entity.clone().into_arguments(&mut res.buffer);
    relations.clone().on_buffer(&mut res.buffer);

    let res = res.fetch_one_row(exec).await;

    let res = match res {
        Ok(res) => res,
        Err(err) => return Err(Either::Left(err)),
    };

    use sqlx::prelude::*;

    Ok(Output {
        id: res.get("id"),
        data: entity,
        info,
        relations,
    })
}

pub struct InsertSt4<'a> {
    pub input: Vec<&'a str>,
    pub output: Option<Vec<&'a str>>,
    pub column: usize,
    pub from: &'a str,
}
pub struct InsertSt3 {
    pub input: Vec<&'static str>,
    pub output: Option<Vec<&'static str>>,
    pub column: usize,
    pub from: &'static str,
}

impl<'q> InsertSt4<'q> {
    fn build(&self) -> String {
        let str = format!(
            "INSERT INTO {} ({}) VALUES {}{}",
            self.from,
            self.input.join(", "),
            {
                let mut binds = 1;
                let mut s = Vec::new();

                for _ in 0..self.column {
                    let mut s_inner = Vec::new();
                    for _ in 0..self.input.len() {
                        s_inner.push(format!("${}", binds));
                        binds += 1;
                    }
                    s.push(format!("({})", s_inner.join(", ")));
                }
                s.join(", ")
            },
            {
                match &self.output {
                    Some(output) => format!(
                        " RETURNING {}",
                        output.join(", ")
                    ),
                    None => "".to_string(),
                }
            }
        );

        debug!("Generated insert statement: {}", str);

        str
    }
}
impl InsertSt3 {
    fn build(&self) -> String {
        let str = format!(
            "INSERT INTO {} ({}) VALUES {}{}",
            self.from,
            self.input.join(", "),
            {
                let mut binds = 1;
                let mut s = Vec::new();

                for _ in 0..self.column {
                    let mut s_inner = Vec::new();
                    for _ in 0..self.input.len() {
                        s_inner.push(format!("${}", binds));
                        binds += 1;
                    }
                    s.push(format!("({})", s_inner.join(", ")));
                }
                s.join(", ")
            },
            {
                match &self.output {
                    Some(output) => format!(
                        " RETURNING {}",
                        output.join(", ")
                    ),
                    None => "".to_string(),
                }
            }
        );

        debug!("Generated insert statement: {}", str);

        str
    }
}

// #[allow(unused)]
// mod test {
//     use std::ops::Not;
//
//     use axum::{routing::get, Router};
//     use better_testing::{axum::invoking, expect, ToBe};
//     use utils::testing_prelude;
//     use cms::{
//         axum_router::AxumRouter,
//         expr::*,
//         migration::migrate,
//         op_insert_one::{insert_one, insert_one_json, Output},
//     };
//     use query::{
//         executable::verbatim_query::Order, prelude::stmt,
//     };
//     use serde_json::json;
//     use sqlx::{Database, FromRow};
//
//     use crate::*;
//
//     #[tokio::test]
//     async fn main_test() {
//         let pool = booststrap_migrate().await;
//
//         let cat_1 = insert_one(
//             Category {
//                 title: "cat_2".to_string(),
//             },
//             (),
//             (),
//             &pool,
//         )
//         .await
//         .unwrap()
//         .id;
//
//         let res = insert_one(
//             Log {
//                 title: "log_1".to_string(),
//                 amount: 11,
//                 color: Some("red".to_string()),
//             },
//             (id::<Category>(cat_1),),
//             (),
//             &pool,
//         )
//         .await
//         .unwrap();
//
//         expect(&res).to_be(&Output {
//             id: 1,
//             data: Log {
//                 title: "log_1".to_string(),
//                 amount: 11,
//                 color: Some("red".to_string()),
//             },
//             relations: (id::<Category>(cat_1),),
//             info: (),
//         });
//
//         let res = insert_one_json(
//             json! {{
//                 "entity": "Log",
//                 "data": {
//                     "title": "log_2",
//                     "amount": 42,
//                     "color": "green",
//                 },
//                 "relations": {
//                     "category": cat_1,
//                 },
//             }},
//             &pool,
//         )
//         .await
//         .unwrap();
//
//         expect(&res).to_be(&json! {{
//             "id": 2,
//             "data": {
//                 "title": "log_2",
//                 "amount": 42,
//                 "color": "green",
//             },
//             "relations": {
//                 "category": {
//                     "id": cat_1,
//                 },
//             },
//         }});
//     }
//
//     #[tokio::test]
//     async fn http_server() {
//         let pool =
//             sqlx::Pool::<Sqlite>::connect("sqlite::memory:")
//                 .await
//                 .unwrap();
//
//         cms::migrate(pool.clone()).await.unwrap();
//         dumpy_data(pool.clone()).await;
//
//         tracing_subscriber::FmtSubscriber::builder()
//             .with_max_level(tracing::Level::DEBUG)
//             .init();
//
//         let app = axum::Router::new()
//             .route("/", get(|| async { "Server is running" }))
//             .nest("/log", Log::router())
//             .nest("/category", Category::router());
//
//         let app: Router<()> = app.with_state(pool.clone());
//
//         use testing_prelude::*;
//
//         let res = app
//             .clone()
//             .oneshot(
//                 Request::builder()
//                     .uri("/log/one")
//                     .method(Method::GET)
//                     .json_body(json!({
//                         "query": {
//                             "id": 2
//                         },
//                         "relations": {
//                             "category": "populate_one_level"
//                         },
//                     }))
//                     .expect("request"),
//             )
//             .await
//             .expect("oneshot")
//             .into_json()
//             .await
//             .expect("into json");
//
//         expect(&res).to_be(&json!({
//             "id": 2,
//             "data": {
//                 "title": "log_2",
//                 "amount": 42,
//                 "color": "green",
//             },
//             "relations": {
//                 "category": {
//                     "id": 1,
//                     "data": {
//                         "title": "cat_1"
//                     },
//                 }
//             },
//         }));
//
//         let res = app
//             .clone()
//             .oneshot(
//                 Request::builder()
//                     .uri("/log")
//                     .method(Method::POST)
//                     .json_body(json!({
//                         "data": {
//                             "title": "log_5",
//                             "amount": 182,
//                             "color": "cyan",
//                         },
//                         "relations": {
//                             "category": {
//                                 "id": 3,
//                                 "populate": "populate_one_level"
//                             }
//                         },
//                     }))
//                     .expect("request"),
//             )
//             .await
//             .expect("oneshot")
//             .into_json()
//             .await
//             .expect("into json");
//
//         expect(&res).to_be(&json!({
//             "id": 5,
//             "data": {
//                 "title": "log_5",
//                 "amount": 182,
//                 "color": "cyan",
//             },
//             "relations": {
//                 "category": {
//                     "id": 3,
//                     "data": {
//                         "title": "cat_3"
//                     },
//                 }
//             },
//         }));
//
//         let res = app
//             .clone()
//             .oneshot(
//                 Request::builder()
//                     .uri("/log/one")
//                     .method(Method::PUT)
//                     .json_body(json!({
//                         "query": { "id": 3, },
//                         "return_data": true,
//                         "data": {
//                             "title": "update",
//                         },
//                         "relations": {
//                             "category": {
//                                 "id": 2,
//                                 "populate": "populate_one_level"
//                             }
//                         },
//                     }))
//                     .expect("request"),
//             )
//             .await
//             .expect("oneshot");
//
//         expect(&res.status()).to_be(&StatusCode::OK);
//
//         let res = res.into_json().await.unwrap();
//
//         expect(&res).to_be(&json!({
//             "id": 3,
//             "data": {
//                 "title": "update",
//                 "amount": 10,
//                 "color": "blue",
//             },
//             "relations": {
//                 "category": {
//                     "id": 2,
//                     "data": {
//                         "title": "cat_2"
//                     },
//                 }
//             },
//         }));
//
//         let res = app
//             .clone()
//             .oneshot(
//                 Request::builder()
//                     .uri("/log/one")
//                     .method(Method::DELETE)
//                     .json_body(json!({
//                         "query": { "id": 4, },
//                         "return_data": true,
//                     }))
//                     .expect("request"),
//             )
//             .await
//             .expect("oneshot");
//
//         expect(&res.status()).to_be(&StatusCode::OK);
//
//         let res = res.into_json().await.unwrap();
//
//         expect(&res).default().to_be(&json!({
//             "id": 4,
//             "data": {
//                 "title": "log_4",
//                 "amount": 1,
//                 "color": "yellow",
//             },
//             "relations_keys": {
//                 "category_id": 3
//             },
//         }));
//     }
// }
