use std::{marker::PhantomData, mem::take, pin::Pin, sync::Arc};

use futures_util::Future;
use queries_for_sqlx::{
    execute_no_cache::ExecuteNoCache, ident_safety::PanicOnUnsafe, insert_one_st::InsertStOne, prelude::{col, ft, stmt, SelectHelpers}, quick_query::QuickQuery, select_st::{
        joins::{join_type, Join},
        SelectSt,
    }, update_st::UpdateSt, InitStatement, SupportNamedBind
};
use serde::Deserialize;
use serde_json::{from_value, json, Value};
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Executor, Pool, Row, Type
};
use tracing::warn;

use crate::{entities::DynEntity, entities::EntityPhantom};

use super::super::operations::Id;

use super::{
    DynRelation, InsertInput, SubmitableRelation, UpdateInput,
};

#[derive(Deserialize)]
struct RelationStructure<R> {
    id: bool,
    attributes: bool,
    // todo: should only skip if ()
    #[serde(skip)]
    relations: R,
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

    impl<S, Base, Related> SubmitableRelation<S>
        for ManyToMany<Base, Related>
    where
        S: Database,
        EntityPhantom<Base>: DynEntity<S>,
        Base: Send + 'static,
        EntityPhantom<Related>: DynEntity<S>,
        Related: Send + 'static,
        for<'s> &'s str: ColumnIndex<S::Row>,
        S: Database,
        Option<i64>: Type<S>
            + for<'d> Decode<'d, S>
            + for<'e> Encode<'e, S>,
        i64: Type<S>
            + for<'d> Encode<'d, S>
            + for<'d> Decode<'d, S>,
    {
        fn schema_key(&self) -> &str {
            self.schema_key
        }
        fn convert(
            &self,
            origin: &'static str,
            input: serde_json::Value,
        ) -> Box<dyn DynRelation<S>> {
            match origin {
                "from_update_one" => {
                    match from_value::<InsertInput<UpdateInput<Vec<i64>>>>(input) {
                        Ok(InsertInput{
                            id: true,
                            attributes: true,
                            data: UpdateInput::connect { id }
                        }) => {
                                    return Box::new(ManyToManyUpdateConnect {
                                        _db: PhantomData,
                                        schema_key: self.schema_key,
                                        conj_table: self.conj_table,
                                        ft: self.rel_t,
                                        fk: self.rel_fk,
                                        lk: self.base_fk,
                                        input: id,
                                        related_entity: Box::new(
                                            EntityPhantom::<Related>(
                                                PhantomData,
                                            ),
                                        ),
                                        output: Default::default(),

                                    });
                            },
                            
                        Ok(_) => todo!("update_one's many-to-many support: id: true, attributes: true, data: {{ connect }}"),
                        Err(err) => panic!("update_one's many-to-many error: {:?}", err),
                    }
                },

                "from_insert_one" => {
                    match from_value::<InsertInput<Vec<Id>>>(
                        input,
                    ) {

                            Ok(InsertInput {
                            id: true, 
                            attributes: true,
                            data,}) => {
                                    return Box::new(ManyToManyInsertOne {
                                        _db: PhantomData,
                                        schema_key: self.schema_key,
                                        conj_table: self.conj_table,
                                        ft: self.rel_t,
                                        fk: self.rel_fk,
                                        lk: self.base_fk,
                                        input: data.into_iter().map(|e| e.id).collect(),
                                        related_entity: Box::new(
                                            EntityPhantom::<Related>(
                                                PhantomData,
                                            ),
                                        ),
                                        output: Default::default(),

                                    });
                                }
                        _ => panic!("insert_one's many-to-many relation support the following format {}", "")
                    }
                }

                "from_get_all" | "from_get_one" => {
                    match from_value::<RelationStructure<()>>(
                        input,
                    ) {
                        Ok(RelationStructure {
                            id: true,
                            attributes: true,
                            ..
                        }) => {
                            return Box::new(ManyToManyGetAll {
                                _db: PhantomData,
                                schema_key: self.schema_key,
                                conj_table: self.conj_table,
                                data: Default::default(),
                                related_entity: Box::new(
                                    EntityPhantom::<Related>(
                                        PhantomData,
                                    ),
                                ),
                                ft: self.rel_t,
                                fk: self.rel_fk,
                                lk: self.base_fk,
                            });
                        }
                        _ => {
                            panic!("get_all input only supports {{'id': true, 'attributes': true }}")
                        }
                    };
                }
                _ => {
                    todo!("unsupported query {}", origin)
                }
            }
        }
    }

    pub struct ManyToManyUpdateConnect<S> {
        pub schema_key: &'static str,
        pub conj_table: &'static str,
        pub ft: &'static str,
        pub fk: &'static str,
        pub lk: &'static str,
        pub input: Vec<i64>,
        pub output: Vec<Value>,
        pub related_entity: Box<dyn DynEntity<S>>,
        pub _db: PhantomData<S>,
    }

    impl<S> DynRelation<S> for ManyToManyUpdateConnect<S>
    where
        S: Send + Database,
        Option<i64>: Type<S>
            + for<'d> Decode<'d, S>
            + for<'e> Encode<'e, S>,
        i64: Type<S>
            + for<'d> Encode<'d, S>
            + for<'d> Decode<'d, S>,
        for<'s> &'s str: ColumnIndex<S::Row>,
    {
        fn sub_op2<'this>(
            &'this mut self,
            pool: Pool<S>,
            id: i64,
        ) -> Pin<Box<dyn Future<Output = ()> + 'this + Send>>
        where
            for<'s> &'s mut <S>::Connection:
                Executor<'s, Database = S>,
            S: Database + SupportNamedBind,
        {
            Box::pin(async move {
                let str = format!("INSERT INTO {} ({}, {}) VALUES {}",
                    self.conj_table,
                    self.fk,
                    self.lk,
                    self.input.iter().map(|e| format!("({}, {})", e, id)).collect::<Vec<_>>().join(", ")
                );

                let res = stmt::StringQuery { sql: str, input: () }.execute(&pool).await.unwrap();

                let str = format!("SELECT * FROM {} LEFT JOIN {} ON id = {} WHERE {}.{} = {id}",
                    self.related_entity.table_name(),
                    // self.input.iter().map(|e| format!("id = {}", e)).collect::<Vec<_>>().join(" OR ")
                    self.conj_table,
                    self.fk,
                    self.conj_table,
                    self.lk,
                );

                let res = stmt::StringQuery { sql: str, input: () }.fetch_all(&pool, |row| {
                    let value = self.related_entity.from_row2(&row)?;
                    let id : i64 = row.get("id");
                    return Ok(json!({
                        "id": id,
                        "attributes": value,
                    })); 
                }).await.unwrap();

                self.output = res;
            })
        }
        fn take2(&mut self, base_id: i64) -> Option<Value> {
            warn!("deprication: I don't need base_id");
            Some(take(&mut self.output).into())
        }
        fn schema_key(&self) -> &str {
            self.schema_key
        }
        fn on_update(&self, st: &mut UpdateSt<S, QuickQuery<'_>>)
    where
        S: Database + SupportNamedBind {
// no op
        }
        fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>,PanicOnUnsafe>)
        where
            S: SupportNamedBind + Database,
        {
            // no op
        }

        fn from_row(&mut self, row: &S::Row)
        where
            S: Database,
        {
            // no op
        }

        fn take(&mut self) -> Option<Value> {
            None
        }
    }
    pub struct ManyToManyInsertOne<S> {
        pub schema_key: &'static str,
        pub conj_table: &'static str,
        pub ft: &'static str,
        pub fk: &'static str,
        pub lk: &'static str,
        pub input: Vec<i64>,
        pub output: Vec<Value>,
        pub related_entity: Box<dyn DynEntity<S>>,
        pub _db: PhantomData<S>,
    }


    impl<S> DynRelation<S> for ManyToManyInsertOne<S>
    where
        S: Send + Database,
        Option<i64>: Type<S>
            + for<'d> Decode<'d, S>
            + for<'e> Encode<'e, S>,
        i64: Type<S>
            + for<'d> Encode<'d, S>
            + for<'d> Decode<'d, S>,
        for<'s> &'s str: ColumnIndex<S::Row>,
    {
        fn sub_op2<'this>(
            &'this mut self,
            pool: Pool<S>,
            id: i64,
        ) -> Pin<Box<dyn Future<Output = ()> + 'this + Send>>
        where
            for<'s> &'s mut <S>::Connection:
                Executor<'s, Database = S>,
            S: Database + SupportNamedBind,
        {
            Box::pin(async move {
                let str = format!("INSERT INTO {} ({}, {}) VALUES {}",
                    self.conj_table,
                    self.fk,
                    self.lk,
                    self.input.iter().map(|e| format!("({}, {})", e, id)).collect::<Vec<_>>().join(", ")
                );

                let res = stmt::StringQuery { sql: str, input: ()}.execute(&pool).await.unwrap();

                let str = format!("SELECT * FROM {} WHERE {}",
                    self.related_entity.table_name(),
                    self.input.iter().map(|e| format!("id = {}", e)).collect::<Vec<_>>().join(" OR ")
                );

                let res = stmt::StringQuery { sql: str, input: ()}.fetch_all(&pool, |row| {
                    let value = self.related_entity.from_row2(&row)?;
                    let id : i64 = row.get("id");
                    return Ok(json!({
                        "id": id,
                        "attributes": value,
                    })); 
                }).await.unwrap();

                self.output = res;
            })
        }
        fn take2(&mut self, base_id: i64) -> Option<Value> {
            warn!("deprication: I don't need base_id");
            Some(take(&mut self.output).into())
        }
        fn schema_key(&self) -> &str {
            self.schema_key
        }
        fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>, PanicOnUnsafe>)
        where
            S: SupportNamedBind + Database,
        {
            // no op
        }

        fn from_row(&mut self, row: &S::Row)
        where
            S: Database,
        {
            // no op
        }

        fn take(&mut self) -> Option<Value> {
            None
        }
    }
    pub struct ManyToManyGetAll<S> {
        pub schema_key: &'static str,
        pub conj_table: &'static str,
        pub ft: &'static str,
        pub fk: &'static str,
        pub lk: &'static str,
        pub data: Vec<(i64, i64, Value)>,
        pub related_entity: Box<dyn DynEntity<S>>,
        pub _db: PhantomData<S>,
    }

    impl<S> DynRelation<S> for ManyToManyGetAll<S>
    where
        S: Send + Database,
        Option<i64>: Type<S>
            + for<'d> Decode<'d, S>
            + for<'e> Encode<'e, S>,
        i64: Type<S>
            + for<'d> Encode<'d, S>
            + for<'d> Decode<'d, S>,
        for<'s> &'s str: ColumnIndex<S::Row>,
    {
        fn sub_op<'this>(
            &'this mut self,
            pool: Pool<S>,
            limit: Arc<dyn super::SelectStLimit<S>>,
        ) -> Pin<Box<dyn Future<Output = ()> + 'this + Send>>
        where
            for<'s> &'s mut <S>::Connection:
                Executor<'s, Database = S>,
            S: Database + SupportNamedBind,
        {
            Box::pin(async move {
                let mut st = stmt::SelectSt::init(self.conj_table);

                self.related_entity.on_select(&mut st);

                st.select(
                    ft(self.conj_table.to_string())
                        .col(self.fk.to_string())
                        .alias("related_key"),
                );
                st.select(
                    ft(self.conj_table.to_string())
                        .col(self.lk.to_string())
                        .alias("base_key"),
                );

                limit.limit(&mut st);

                st.join(Join {
                    ty: join_type::Left,
                    on_table: self.ft,
                    on_column: "id",
                    local_column: self.fk,
                });

                let vals = st
                    .fetch_all(&pool, |row| {
                        let val = self
                            .related_entity
                            .from_row2(&row)?;
                        Ok((
                            row.get("related_key"),
                            row.get("base_key"),
                            val,
                        ))
                    })
                    .await
                    .unwrap();

                self.data = vals;
            })
        }
        fn take2(&mut self, base_id: i64) -> Option<Value> {
            let val = self
                .data
                .iter()
                .filter_map(|e| {
                    if e.1 != base_id {
                        return None;
                    }
                    Some(json! ({
                        "id": e.0,
                        "attributes": e.2
                    }))
                })
                .collect::<Vec<_>>();
            Some(val.into())
        }
        fn schema_key(&self) -> &str {
            self.schema_key
        }
        fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>, PanicOnUnsafe>)
        where
            S: SupportNamedBind + Database,
        {
            // no op
        }

        fn from_row(&mut self, row: &S::Row)
        where
            S: Database,
        {
            // no op
        }

        fn take(&mut self) -> Option<Value> {
            None
        }
    }
