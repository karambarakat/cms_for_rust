use std::collections::HashMap;

use case::CaseExt;
use queries_for_sqlx::create_table_st::CreateTableHeader;
use serde::de::DeserializeOwned;
use serde_json::from_value;

use crate::{
    dynamic_schema::{
        DynInsertOneWorker, DynamicRelationResult,
    },
    migration2::DynMigration,
    operations::{
        delete_one::DynDeleteWorker,
        insert_one::InsertOneWorker, select_many::GetAllWorker,
        update_one::UpdateOneWorker, IdOutput, SimpleOutput,
    },
    queries_bridge::{SelectSt, UpdateSt},
    relations::{ManyWorker, UpdateId},
};

use super::{
    prelude::*, LinkIdWorker, LinkSpecCanInsert,
    LinkSpecCanUpdate, UpdateIdWorker,
};

#[derive(Clone)]
pub struct OptionalToMany {
    pub foriegn_key: String,
}

impl DynMigration for (&'static str, OptionalToMany) {
    fn migrate(
        &self,
        ctx: &mut crate::migration2::MigrationCtx,
    ) -> Result<(), String> {
        let name = self.0.to_owned();

        todo!();

        Ok(())
    }
}

impl LinkSpec for OptionalToMany {}
impl LinkSpecCanInsert for OptionalToMany {
    type Input = i64;
}

impl LinkSpecCanUpdate for OptionalToMany {
    type Input = Option<i64>;
}

pub struct OptionalToManyDynamic<From, To> {
    pub(crate) list_itself_under: String,
    pub(crate) rel_spec: OptionalToMany,
    pub(crate) key: String,
    pub(crate) _pd: PhantomData<(From, To)>,
}

impl<From, To> OptionalToManyDynamic<From, To> {
    pub fn new() -> Self
    where
        From: Linked<To, Spec = OptionalToMany>,
        From: Collection<Sqlite> + Serialize + 'static,
        To: Collection<Sqlite>,
    {
        Self {
            list_itself_under: From::table_name().to_string(),
            key: To::table_name().to_snake(),
            rel_spec: From::spec(),
            _pd: PhantomData,
        }
    }
}

impl<From, To> CompleteRelationForServer
    for OptionalToManyDynamic<From, To>
where
    From: Collection<Sqlite> + Serialize + 'static,
    To: Collection<Sqlite>
        + Serialize
        + 'static
        + DeserializeOwned,
{
    fn list_iteself_under(&self) -> String {
        self.list_itself_under.to_string()
    }

    fn key(&self) -> String {
        self.key.clone()
    }

    fn init_on_delete(
        self: Arc<Self>,
        to: &str,
    ) -> DynamicRelationResult<Box<dyn DynDeleteWorker>> {
        if to != self.key {
            return DynamicRelationResult::NotFound;
        }

        struct Worker {
            spec: OptionalToMany,
        }

        impl DynDeleteWorker for Worker {
            fn sub_op(
                &mut self,
                db: axum::extract::State<Pool<Sqlite>>,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = ()> + Send>,
            > {
                Box::pin(async move {})
            }

            fn from_row(&mut self, r: &SqliteRow) -> Value {
                let re: Option<i64> =
                    r.get(self.spec.foriegn_key.as_str());

                re.into()
            }
        }

        let ret = {
            // DynamicWorker::new( self.clone(),
            Worker {
                spec: self.rel_spec.clone(),
            }
            // )
        };

        DynamicRelationResult::Ok(Box::new(ret))
    }

    fn init_on_update(
        self: Arc<Self>,
        to: &str,
        input: Value,
    ) -> DynamicRelationResult<
        Box<dyn crate::dynamic_schema::DynUpdateOneWorker>,
    > {
        if to != self.key {
            return DynamicRelationResult::NotFound;
        }

        #[allow(non_camel_case_types)]
        #[derive(Deserialize)]
        enum ValidInput<T> {
            set(Option<i64>),
            create_new(T),
        }

        let input = match from_value::<ValidInput<To>>(input) {
            Ok(ValidInput::set(ok)) => ok,
            Ok(_) => todo!(),
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
                _pd: PhantomData::<(From, To)>,
            },
        ));
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
        enum ValidInput {
            set_id_to_and_populate(i64),
        }

        let input = match from_value::<ValidInput>(input) {
            Ok(ok) => ok,
            Err(err) => {
                return DynamicRelationResult::InvalidInput(
                    err.to_string(),
                )
            }
        };

        let ret = match input {
            ValidInput::set_id_to_and_populate(id) => {
                DynamicWorker::new(
                    self.clone(),
                    LinkIdWorker {
                        input: id,
                        spec: self.rel_spec.clone(),
                        _pd: PhantomData::<(From, To)>,
                    },
                )
            }
        };

        return DynamicRelationResult::Ok(ret);
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
            Ok(ok) => ok,
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
                _pd: PhantomData::<(From, To)>,
            },
        );

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
}

impl<From, To> GetOneWorker
    for RelationWorker<OptionalToMany, From, To>
where
    From: Send + Collection<Sqlite>,
    To: Send + Collection<Sqlite>,
{
    type Output = Option<SimpleOutput<To>>;
    type Inner = Option<(i64, To)>;

    fn on_select(
        &self,
        data: &mut Self::Inner,
        st: &mut SelectSt<Sqlite>,
    ) {
        st.left_join(join {
            on_table: To::table_name().to_string(),
            on_column: "id".to_string(),
            local_column: self.rel_spec.foriegn_key.to_string(),
        });
        st.select_scoped(
            From::table_name().to_string(),
            self.rel_spec.foriegn_key.clone(),
        );
        To::on_select(st);
    }

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        let id: Option<i64> =
            row.get(self.rel_spec.foriegn_key.as_str());
        if let Some(id) = id {
            let value = To::from_row_scoped(row);
            *data = Some((id, value));
        }
    }

    async fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) {
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        data.map(|(id, attr)| SimpleOutput { id, attr })
    }
}

impl<B, T> InsertOneWorker
    for LinkIdWorker<B, T, OptionalToMany, i64>
where
    B: Collection<Sqlite>,
    T: Collection<Sqlite>,
{
    type Inner = Option<T>;

    type Output = SimpleOutput<T>;

    fn on_insert(
        &self,
        data: &mut Self::Inner,
        st: &mut stmt::InsertStOne<'_, Sqlite, ()>,
    ) {
        st.insert(self.spec.foriegn_key.clone(), self.input);
    }

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
    }

    fn sub_op2<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this {
        async move {
            let mut st = stmt::SelectSt::init(
                T::table_name().to_string(),
            );

            T::on_select(&mut st);

            let id = self.input;
            st.where_(col("id").eq(id));

            st.fetch_one(&pool, |row| {
                *data = Some(T::from_row_scoped(&row));
                Ok(())
            })
            .await;
        }
    }

    fn sub_op1<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this {
        async {}
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        SimpleOutput {
            id: self.input,
            attr: data.unwrap(),
        }
    }
}

impl<B, T> UpdateOneWorker
    for UpdateIdWorker<B, T, OptionalToMany, Option<i64>>
where
    B: Collection<Sqlite>,
    T: Collection<Sqlite>,
{
    type Inner = Option<i64>;

    type Output = Option<IdOutput<T>>;

    fn on_update(
        &self,
        data: &mut Self::Inner,
        st: &mut UpdateSt<Sqlite>,
    ) {
        let id = self.input.clone();
        st.set(self.spec.foriegn_key.to_string(), id);
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        self.input.map(|id| IdOutput {
            id,
            _pd: PhantomData,
        })
    }
}

impl<F, T> GetAllWorker for ManyWorker<F, T, OptionalToMany>
where
    F: Collection<Sqlite>,
    T: Collection<Sqlite>,
{
    type Inner = HashMap<i64, SimpleOutput<T>>;

    type Output = Option<SimpleOutput<T>>;

    fn on_select(
        &self,
        data: &mut Self::Inner,
        st: &mut SelectSt<Sqlite>,
    ) {
        st.left_join(join {
            on_table: T::table_name().to_string(),
            on_column: "id".to_string(),
            local_column: self.spec.foriegn_key.to_string(),
        });

        st.select(self.spec.foriegn_key.clone());
        T::on_select(st);
    }

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        let id: Option<i64> =
            row.get(self.spec.foriegn_key.as_str());
        if let Some(id) = id {
            let value = T::from_row_scoped(row);
            let local_id = row.get("local_id");
            data.insert(
                local_id,
                SimpleOutput { id, attr: value },
            );
        }
    }

    fn take(
        &mut self,
        current_id: i64,
        data: &mut Self::Inner,
    ) -> Self::Output {
        data.remove(&current_id)
    }
}
