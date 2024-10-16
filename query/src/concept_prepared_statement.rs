use core::hash;
use std::any::Any;
use std::marker::PhantomData;
use std::ops::Not;

use futures_util::Future;
use sqlx::database::HasArguments;
use sqlx::{Arguments, Database, Pool, Sqlite};

use crate::create_table_st::constraints::exports::default;
use crate::executable::InnerExecutable;
use crate::select_st::SelectSt;
use crate::{Accept, IntoMutArguments, Query, Statement};

pub trait IntoMutArgumentsMeta<'q, DB>:
    IntoMutArguments<'q, DB>
where
    DB: Database,
{
    type Meta: BindMeta;
}

pub trait BindMeta: Sized {
    fn shift(self, by: usize) -> Self;
    fn names(&self) -> &[&'static str];
}

struct DynamicQuery<S>(PhantomData<S>);

impl<S> DynamicQuery<S> {
    pub fn build<St: Statement<S, Self>>(
        st: St,
    ) -> (CachedExecute<S>, DynamicDeserializer<S>) {
        let str = st._build();
        todo!()
    }
}

impl<S> Query<S> for DynamicQuery<S> {
    type SqlPart = String;

    type Context1 = Vec<Input>;

    type Context2 = ();

    fn build_sql_part_back(
        ctx: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String {
        todo!()
    }

    type Output = ();

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        todo!()
    }
}

struct Input(&'static str, Box<dyn Ty>);

trait Ty {}
impl<T: 'static> Ty for PhantomData<T> {}

fn input<T: 'static>(name: &'static str) -> Input {
    Input(name, Box::new(PhantomData::<T>))
}

impl<S> Accept<Input, S> for DynamicQuery<S> {
    fn accept(
        this: Input,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        ctx1.push(this);
        let len = ctx1.len();
        move |_| format!("${}", len)
    }
}

struct CachedExecute<S>(String, PhantomData<S>);

struct H(std::collections::HashMap<(), ()>);

impl<S: Database> CachedExecute<S> {
    fn execute<E, H: hash::Hash + 'static>(
        self,
        executor: E,
        // 'static restriction here can be lifted if I
        // restrict the output future to execute before
        // an lifetime 'q
        // the lifetime is just for Sqlite value that
        // can be some Cow<'q, str>
        input: (H, <S as HasArguments<'static>>::Arguments),
    ) -> impl Future<Output = Result<S::QueryResult, sqlx::Error>>
           + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
    {
        if <H as Any>::type_id(&input.0).eq(&().type_id()).not()
        {
            todo!("chached statment")
        }
        async move {
            InnerExecutable {
                stmt: self.0.as_str(),
                buffer: input.1,
                persistent: false,
            }
            .execute(executor)
            .await
        }
    }
}

struct DynamicDeserializer<S>(PhantomData<S>);

impl<S: Database> DynamicDeserializer<S> {
    pub fn input(
        &self,
        input: serde_json::Value,
    ) -> Result<
        ((), <S as HasArguments<'static>>::Arguments),
        serde_json::Error,
    > {
        todo!()
    }
}

async fn concept() {
    use serde_json::json;

    use crate::{
        expressions::exports::{all_columns, col},
        from_row,
        prelude::stmt,
    };

    let mut st =
        SelectSt::<Sqlite, DynamicQuery<Sqlite>>::init("Users");

    st.select(all_columns());

    st.where_(col("id").eq(input::<i32>("by_id")));
    st.limit(input::<i32>("limit"));

    let (prepared_st, des) = DynamicQuery::build(st);

    let pool = Pool::<Sqlite>::connect("").await.unwrap();

    let dynamic_value = json!({
        "limit": 3,
        "by_id": 1,
    });

    prepared_st
        .execute(
            &pool,
            des.input(dynamic_value)
                .expect("format of value is invalid"),
        )
        .await
        .unwrap();
}
