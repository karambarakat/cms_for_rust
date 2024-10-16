use std::{marker::PhantomData, mem::take, pin::Pin};

use futures_util::Future;
use queries_for_sqlx::{
    execute_no_cache::ExecuteNoCache, insert_one_st::InsertStOne, prelude::{col, stmt}, quick_query::QuickQuery, select_st::{
        joins::{join_type, Join},
        SelectSt,
    }, update_st::UpdateSt, InitStatement, SupportNamedBind
};
use serde_json::{from_value, json, Value};
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Executor, Pool, Row,
    Type,
};
use tracing::warn;

use crate::{entities::DynEntity, entities::EntityPhantom};

use super::super::operations::Id;

use super::{
    DynRelation, InsertInput, SubmitableRelation, UpdateInput,
};

pub struct OneToMany<Base, Related> {
    pub relation_key: &'static str,
    pub foreign_entity_snake_case: &'static str,
    pub foreign_table: &'static str,
    pub _entities: PhantomData<(Base, Related)>,
}

pub struct OptionalToMany<O, M>(
    pub &'static str,
    pub PhantomData<(O, M)>,
);

impl<O, M> OptionalToMany<O, M> {
    pub fn colomn_link(str: &'static str) -> Self {
        Self(str, PhantomData)
    }
}

impl<S, Base, Related> SubmitableRelation<S>
    for OneToMany<Base, Related>
where
    S: Database,
    EntityPhantom<Base>: DynEntity<S>,
    Base: Send + 'static,
    EntityPhantom<Related>: DynEntity<S>,
    Related: Send + 'static,
    Option<i64>:
        Type<S> + for<'d> Decode<'d, S> + for<'e> Encode<'e, S>,
    i64: Type<S> + for<'d> Encode<'d, S> + for<'d> Decode<'d, S>,
    S: Database,
    for<'s> &'s str: ColumnIndex<S::Row>,
{
    fn schema_key(&self) -> &str {
        self.foreign_entity_snake_case
    }
    fn convert(
        &self,
        origin: &'static str,
        input: serde_json::Value,
    ) -> Box<dyn DynRelation<S>> {
        match origin {
            "from_update_one" => {
                match from_value::<InsertInput<UpdateInput<i64>>>(input) {
                        Ok(InsertInput{
                            id: true,
                            attributes: true,
                            data: UpdateInput::set { id }
                        }) => {
                                Box::new(OneToManyUpdateSet {
                                    input: id,
                                schema_relation_key: self
                                    .foreign_entity_snake_case,
                                relation_key: self.relation_key,
                                related_entity: Box::new(
                                    EntityPhantom::<Related>(
                                        PhantomData,
                                    ),
                                ),
                                output: Default::default(),
                                })
                            },
                        Ok(_) => todo!("update_one's one-to-many relation support the following format: id: true, attributes: true, data: {{ set }}"),
                        Err(err) => panic!("update_one's one-to-many format {:?}", err),
                    }
            }
            "from_get_all" | "from_get_one" => {
                Box::new(OneToManyS {
                    input,
                    relation_key: self.relation_key,
                    related_entity: Box::new(EntityPhantom::<
                        Related,
                    >(
                        PhantomData
                    )),
                    value: Default::default(),
                    schema_relation_key: self
                        .foreign_entity_snake_case,
                })
            }
            "from_insert_one" => {
                match from_value::<InsertInput<Id>>(input) {
                    Ok(InsertInput {
                        id: true,
                        attributes: true,
                        data,
                    }) => {
                        return Box::new(OneToManyInsert {
                            input: data,
                            relation_key: self.relation_key,
                            related_entity: Box::new(
                                EntityPhantom::<Related>(
                                    PhantomData,
                                ),
                            ),
                            output: Default::default(),
                            schema_relation_key: self
                                .foreign_entity_snake_case,
                        });
                    }
                    _ => {
                        panic!("insert_one input only supports {{'id': true, 'attributes': true }}")
                    }
                };
                // match from_value::<InsertInput<()>>(
                //     input,
                // ) {
            }
            _ => {
                todo!("query {} is not supported", origin)
            }
        }
    }
}

pub struct OneToManyUpdateSet<S> {
    pub input: i64,
    pub schema_relation_key: &'static str,
    pub relation_key: &'static str,
    pub related_entity: Box<dyn DynEntity<S>>,
    pub output: Option<Value>,
}

impl<S> DynRelation<S> for OneToManyUpdateSet<S>
where
    Option<i64>: Type<S> + for<'d> Decode<'d, S>,
    S: Database,
    i64: Type<S> + for<'d> Encode<'d, S>,
    for<'s> &'s str: ColumnIndex<S::Row>,
{
    fn take2(&mut self, _local_id: i64) -> Option<Value> {
        self.output.take()
    }
    fn on_update(&self, st: &mut UpdateSt<S, QuickQuery<'_>>)
    where
        S: Database + SupportNamedBind,
    {
        let input = self.input;
        st.set(self.relation_key, move || input);
    }
    fn sub_op2<'this>(
        &'this mut self,
        pool: Pool<S>,
        _: i64,
    ) -> Pin<Box<dyn Future<Output = ()> + 'this + Send>>
    where
        for<'s> &'s mut <S>::Connection:
            Executor<'s, Database = S>,
        S: Database + SupportNamedBind,
    {
        Box::pin(async move {
            let mut st =
                stmt::SelectSt::init(self.related_entity.table_name());

            let id = self.input;
            st.where_(col("id").eq(move || id));

            self.related_entity.on_select(&mut st);

            use queries_for_sqlx::execute_no_cache::ExecuteNoCache;

            let res = st
                .fetch_one(&pool, |r| {
                    let value =
                        self.related_entity.from_row2(&r)?;
                    Ok(value)
                })
                .await
                .unwrap();

            self.output = Some(json!({
                "id": id,
                "attributes": res
            }));
        })
    }
    fn on_insert(&self, st: &mut InsertStOne<'_, S>)
    where
        S: Database,
    {
        todo!()
    }
    fn schema_key(&self) -> &str {
        self.schema_relation_key
    }
    fn take(&mut self) -> Option<Value> {
        todo!()
    }

    fn from_row(&mut self, row: &<S>::Row)
    where
        S: Database,
    {
        todo!()
    }
    fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>>)
    where
        S: SupportNamedBind + Database,
    {
        todo!()
    }
}

pub struct OneToManyInsert<S> {
    pub input: Id,
    pub schema_relation_key: &'static str,
    pub relation_key: &'static str,
    pub related_entity: Box<dyn DynEntity<S>>,
    pub output: Option<Value>,
}

impl<S> DynRelation<S> for OneToManyInsert<S>
where
    Option<i64>: Type<S> + for<'d> Decode<'d, S>,
    S: Database,
    i64: Type<S> + for<'d> Encode<'d, S>,
    for<'s> &'s str: ColumnIndex<S::Row>,
{
    fn take2(&mut self, _local_id: i64) -> Option<Value> {
        take(&mut self.output)
    }
    fn sub_op2<'this>(
        &'this mut self,
        pool: Pool<S>,
        _: i64,
    ) -> Pin<Box<dyn Future<Output = ()> + 'this + Send>>
    where
        for<'s> &'s mut <S>::Connection:
            Executor<'s, Database = S>,
        S: Database + SupportNamedBind,
    {
        Box::pin(async move {
            let mut st =
                stmt::SelectSt::init(self.related_entity.table_name());

            // limit here used to work for many-to-many
            // relations, where I add : where base_id = 1
            // this is unnecessary because I have the id
            // in input
            warn!("refactor: depricate limit on insert");

            self.related_entity.on_select(&mut st);

            let id = self.input.id;
            st.where_(col("id").eq(move || id));

            let res = st
                .fetch_one(&pool, |row| {
                    let val = self
                        .related_entity
                        .from_row2(&row)
                        .unwrap();

                    Ok(val)
                })
                .await
                .unwrap();

            self.output = Some(json!({
                "id": self.input.id,
                "attributes": res
            }));
        })
    }
    fn on_insert(&self, st: &mut InsertStOne<'_, S>)
    where
        S: Database,
    {
        let input = self.input.id;

        st.insert(self.relation_key, input);
    }
    fn schema_key(&self) -> &str {
        self.schema_relation_key
    }
    fn take(&mut self) -> Option<Value> {
        None // no-op
    }

    fn from_row(&mut self, row: &<S>::Row)
    where
        S: Database,
    {
        // no-op
    }
    fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>>)
    where
        S: SupportNamedBind + Database,
    {
        // no-op
    }
}

pub struct OneToManyS<S> {
    #[deprecated]
    pub input: Value,
    pub schema_relation_key: &'static str,
    pub relation_key: &'static str,
    pub related_entity: Box<dyn DynEntity<S>>,
    pub value: Option<Value>,
}

impl<S> DynRelation<S> for OneToManyS<S>
where
    Option<i64>: Type<S> + for<'d> Decode<'d, S>,
    S: Database,
    i64: Type<S> + for<'d> Encode<'d, S>,
    for<'s> &'s str: ColumnIndex<S::Row>,
{
    fn on_insert(&self, st: &mut InsertStOne<'_, S>)
    where
        S: Database,
    {
        let input = self
            .input
            .get("data")
            .unwrap()
            .get("id")
            .unwrap()
            .as_i64()
            .unwrap();
        st.insert(self.relation_key, input);
        // panic!("{input}");
        // self.related_entity.into_insert(input, st);
    }
    fn schema_key(&self) -> &str {
        self.schema_relation_key
    }
    fn take(&mut self) -> Option<Value> {
        Some(take(&mut self.value).into())
    }
    fn from_row(&mut self, row: &<S>::Row)
    where
        S: Database,
    {
        let id: Option<i64> = row.get(self.relation_key);
        if let Some(id) = id {
            let value = self
                .related_entity
                .from_row_scoped(row)
                .unwrap();
            self.value =
                Some(json!({"id": id, "attributes": value}));
        }
    }
    fn on_select(&self, st: &mut SelectSt<S, QuickQuery<'_>>)
    where
        S: SupportNamedBind + Database,
    {
        st.join(Join {
            ty: join_type::Left,
            on_table: self.related_entity.table_name(),
            on_column: "id",
            local_column: self.relation_key,
        });

        st.select(col(self.relation_key));

        self.related_entity.on_select_scoped(st);
    }
}
