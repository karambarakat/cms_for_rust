use std::collections::HashMap;

use case::CaseExt;
use queries_for_sqlx::{
    create_table_st::CreateTableHeader,
    insert_many_st::insert_many,
};
use serde::de::DeserializeOwned;
use serde_json::from_value;
use stmt::InsertMany;

use crate::{
    dynamic_schema::{
        DynInsertOneWorker, DynUpdateOneWorker,
        DynamicRelationResult,
    },
    migration2::DynMigration,
    operations::{
        insert_one::InsertOneWorker, select_many::GetAllWorker,
        update_one::UpdateOneWorker, SimpleOutput,
    },
    queries_bridge::{DeleteSt, SelectSt},
    relations::ManyWorker,
};

use super::{
    prelude::*, LinkIdWorker, LinkSpecCanInsert,
    LinkSpecCanUpdate, UpdateIdInput, UpdateIdWorker,
};

#[derive(Clone)]
pub struct ManyToMany {
    pub conjuction_table: String,
    pub base_id: String,
    pub destination_id: String,
}

impl DynMigration for ManyToMany {
    fn panic_on_unsafe_schema(&self) {
        queries_for_sqlx::ident_safety::append_schema(
            &self.conjuction_table,
            &[&self.base_id, &self.destination_id],
        )
    }
    fn migrate(
        &self,
        ctx: &mut crate::migration2::MigrationCtx,
    ) -> Result<(), String> {
        let name = self.conjuction_table.clone();
        let new = stmt::CreateTableSt::init((
            CreateTableHeader::IfNotExists,
            &name,
        ));

        match ctx.store.0.insert(name.to_string(), new) {
            None => {}
            Some(_) => {
                panic!("should not contain old table")
            }
        };

        let table = ctx.store.0.get_mut(&name);

        todo!();

        Ok(())
    }
}

impl LinkSpec for ManyToMany {}
impl LinkSpecCanInsert for ManyToMany {
    type Input = Vec<i64>;
}
impl LinkSpecCanUpdate for ManyToMany {
    type Input = Vec<UpdateIdInput>;
}

pub struct ManyToManyDynamic<From, To> {
    pub(crate) list_itself_under: String,
    pub(crate) key: String,
    pub(crate) rel_spec: ManyToMany,
    pub(crate) _pd: PhantomData<(From, To)>,
}

impl<From, To> ManyToManyDynamic<From, To>
where
    From: Collection<Sqlite>,
    To: Collection<Sqlite>,
    From: Linked<To, Spec = ManyToMany>,
{
    pub fn new() -> Self {
        Self {
            list_itself_under: From::table_name().to_string(),
            key: To::table_name().to_snake(),
            rel_spec: From::spec(),

            _pd: PhantomData,
        }
    }
}

impl<F, T> CompleteRelationForServer for ManyToManyDynamic<F, T>
where
    F: Collection<Sqlite> + 'static,
    T: Collection<Sqlite> + 'static + Serialize + DeserializeOwned,
{
    fn list_iteself_under(&self) -> String {
        self.list_itself_under.clone()
    }

    fn key(&self) -> String {
        self.key.clone()
    }

    fn init_on_insert(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<Box<dyn DynInsertOneWorker>> {
        if to != self.key {
            return DynamicRelationResult::NotFound;
        }

        #[allow(non_camel_case_types)]
        #[derive(Deserialize)]
        enum ValidInput<T> {
            set_id_to_and_populate(Vec<i64>),
            set_id_to(Vec<i64>),
            create_new(Vec<T>),
        }

        let input = match from_value::<ValidInput<T>>(input) {
            Ok(ok) => ok,
            Err(err) => {
                return DynamicRelationResult::InvalidInput(
                    err.to_string(),
                )
            }
        };

        let ret: Box<dyn DynInsertOneWorker> = match input {
            ValidInput::set_id_to_and_populate(vec) => {
                DynamicWorker::new(
                    self.clone(),
                    LinkIdWorker {
                        input: vec,
                        spec: self.rel_spec.clone(),
                        _pd: PhantomData::<(F, T)>,
                    },
                )
            }
            _ => {
                todo!("only set_id_to_and_populate is supported for now")
            }
        };

        return DynamicRelationResult::Ok(ret);
    }

    fn init_on_get(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<Box<dyn DynGetOneWorker>> {
        if to != self.key {
            return DynamicRelationResult::NotFound;
        }

        #[derive(Deserialize)]
        struct ValidInput {}

        let input = match from_value::<ValidInput>(input) {
            Ok(_) => {}
            Err(err) => {
                return DynamicRelationResult::InvalidInput(
                    err.to_string(),
                )
            }
        };

        DynamicRelationResult::Ok(Box::new(DynamicWorker {
            rw: Some(RelationWorker {
                rel_spec: self.rel_spec.clone(),
                _pd: self._pd,
            }),
            arc: self.clone(),
            inner: Default::default(),
        }))
    }

    fn init_on_get_all(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<
        Box<dyn crate::dynamic_schema::DynGetManyWorker>,
    > {
        if to != self.key {
            return DynamicRelationResult::NotFound;
        }

        #[derive(Deserialize)]
        struct ValidInput {}

        let input = match from_value::<ValidInput>(input) {
            Ok(_) => {}
            Err(err) => {
                return DynamicRelationResult::InvalidInput(
                    err.to_string(),
                )
            }
        };

        let ret = DynamicWorker::new(
            self.clone(),
            ManyWorker {
                spec: self.rel_spec.clone(),
                _pd: PhantomData::<(F, T)>,
            },
        );

        return DynamicRelationResult::Ok(ret);
    }

    fn init_on_update(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<Box<dyn DynUpdateOneWorker>> {
        if to != self.key {
            return DynamicRelationResult::NotFound;
        }

        let input = match from_value::<Vec<UpdateIdInput>>(input)
        {
            Ok(ok) => ok,
            Err(err) => {
                return DynamicRelationResult::InvalidInput(
                    err.to_string(),
                )
            }
        };

        return DynamicRelationResult::Ok(DynamicWorker::new(
            self.clone(),
            UpdateIdWorker {
                input,
                spec: self.rel_spec.clone(),
                _pd: PhantomData::<(F, T)>,
            },
        ));
    }
}

impl<Base, Destination> GetOneWorker
    for RelationWorker<ManyToMany, Base, Destination>
where
    Base: Collection<Sqlite>,
    Destination: Collection<Sqlite>,
{
    type Inner = (Option<i64>, Vec<(i64, Destination)>);
    type Output = Vec<SimpleOutput<Destination>>;

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        *&mut data.0 = Some(row.get("local_id"))
    }

    fn sub_op<'t>(
        &'t self,
        data: &'t mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 't {
        async move {
            let id = data.0.unwrap();

            let mut st = stmt::SelectSt::init(
                self.rel_spec.conjuction_table.to_string(),
            );

            st.select_aliased(
                &self.rel_spec.conjuction_table,
                &self.rel_spec.destination_id,
                "dest_id",
            );

            Destination::on_select(&mut st);

            st.where_(col(self.rel_spec.base_id.clone()).eq(id));

            st.left_join(join {
                on_table: Destination::table_name().to_string(),
                on_column: "id".to_string(),
                local_column: self
                    .rel_spec
                    .destination_id
                    .clone(),
            });

            let vals = st
                .fetch_all(&pool, |row| {
                    let val = Destination::from_row_scoped(&row);
                    Ok((row.get::<'_, i64, _>("dest_id"), val))
                })
                .await
                .unwrap();

            *&mut data.1 = vals;
        }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        data.1
            .into_iter()
            .map(|e| SimpleOutput { id: e.0, attr: e.1 })
            .collect()
    }
}

impl<B, T> InsertOneWorker
    for LinkIdWorker<B, T, ManyToMany, Vec<i64>>
where
    B: Collection<Sqlite>,
    T: Collection<Sqlite>,
{
    type Inner = (Option<i64>, Vec<T>);

    type Output = Vec<SimpleOutput<T>>;

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        *&mut data.0 = Some(row.get("id"));
    }

    fn sub_op2<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this {
        async move {
            let local_id = data.0.unwrap();
            let mut st = insert_many(
                self.spec.conjuction_table.to_string(),
            )
            .columns(vec![
                self.spec.base_id.clone(),
                self.spec.destination_id.clone(),
            ])
            .values(
                self.input
                    .iter()
                    .map(|id| (local_id, *id))
                    .collect(),
            );
            st.execute(&pool).await.unwrap();

            let mut st =
                SelectSt::init(T::table_name().to_string());

            st.where_(or(self
                .input
                .iter()
                .cloned()
                .map(|e| col("id").eq(e))
                .collect()));

            T::on_select(&mut st);

            let res = st
                .fetch_all(&pool, |r| Ok(T::from_row_scoped(&r)))
                .await
                .unwrap();

            *&mut data.1 = res;
        }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        // todo!()
        self.input
            .into_iter()
            .zip(data.1)
            .map(|(id, data)| SimpleOutput { id, attr: data })
            .collect()
    }
}

impl<B, T> UpdateOneWorker
    for UpdateIdWorker<B, T, ManyToMany, Vec<UpdateIdInput>>
where
    B: Collection<Sqlite>,
    T: Collection<Sqlite>,
{
    type Inner = (Option<i64>, Vec<i64>);

    type Output = Vec<i64>;

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        *&mut data.0 = Some(row.get("id"));
    }

    fn sub_op2<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this {
        async move {
            let mut new = vec![];
            let mut remove = vec![];

            let base_id = data.0.unwrap();

            for each in self.input.iter() {
                match each {
                    UpdateIdInput::remove_link(id) => {
                        remove.push(id.clone())
                    }
                    UpdateIdInput::set_link(id) => {
                        new.push((base_id, id.clone()));
                    }
                }
            }

            // exec all set_link
            let mut new_st: InsertMany<Sqlite, _> = insert_many(
                self.spec.conjuction_table.to_string(),
            );

            new_st
                .columns(vec![
                    self.spec.base_id.to_string(),
                    self.spec.destination_id.to_string(),
                ])
                .values(new)
                .execute(&pool.clone())
                .await
                .unwrap();

            // exec all remove_id
            let mut rem_st = DeleteSt::init(
                self.spec.conjuction_table.to_string(),
            );

            rem_st.where_(or(remove
                .into_iter()
                .map(|e| {
                    col(self.spec.destination_id.to_string())
                        .eq(e)
                })
                .collect()));

            rem_st.execute(&pool).await.unwrap();

            // populate more data
            let mut st = SelectSt::init(
                self.spec.conjuction_table.to_string(),
            );

            // st.select(col(self.spec.destination_id.to_string()));
            st.select(self.spec.destination_id.to_owned());

            st.where_(
                col(self.spec.base_id.clone()).eq(base_id),
            );

            let res = st
                .fetch_all(&pool, |r| {
                    let r: i64 = r.get(0);
                    Ok(r)
                })
                .await
                .unwrap();

            *&mut data.1 = res
        }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        data.1
    }
}

impl<F, T> GetAllWorker for ManyWorker<F, T, ManyToMany>
where
    F: Collection<Sqlite>,
    T: Collection<Sqlite>,
{
    type Inner = HashMap<i64, Vec<SimpleOutput<T>>>;

    type Output = Vec<SimpleOutput<T>>;

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        let local_id = row.get("local_id");
        data.insert(local_id, vec![]);
    }

    fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this {
        async move {
            let mut st = SelectSt::init(
                self.spec.conjuction_table.to_string(),
            );

            let id_set =
                data.keys().cloned().collect::<Vec<_>>();

            st.select_aliased(
                &self.spec.conjuction_table,
                &self.spec.base_id,
                "from_id",
            );

            st.select_aliased(
                &self.spec.conjuction_table,
                &self.spec.destination_id,
                "dest_id",
            );

            st.where_(or(id_set
                .into_iter()
                .map(|id| {
                    let id = id.clone();
                    col(self.spec.base_id.clone()).eq(id)
                })
                .collect()));

            st.left_join(join {
                on_table: T::table_name().to_string(),
                on_column: "id".to_string(),
                local_column: self.spec.destination_id.clone(),
            });

            T::on_select(&mut st);

            let res = st
                .fetch_all(&pool, |r| {
                    let id = r.get("dest_id");
                    let from_id = r.get("from_id");
                    let attr = T::from_row_scoped(&r);
                    data.get_mut(&from_id)
                        .unwrap()
                        .push(SimpleOutput { id, attr });

                    Ok(())
                })
                .await
                .unwrap();
        }
    }

    fn take(
        &mut self,
        current_id: i64,
        data: &mut Self::Inner,
    ) -> Self::Output {
        data.remove(&current_id).unwrap()
    }
}
