use futures_util::Future;
use sqlx::{database::HasArguments, Database};

use crate::{
    create_table_st::CreateTableSt, delete_st::DeleteSt,
    executable::InnerExecutable, insert_many_st::InsertMany,
    insert_one_st::InsertStOne, returning::ReturningClause,
    select_st::SelectSt, update_st::UpdateSt, Statement, Query,
    SupportNamedBind,
};

pub trait ExecuteNoCache<'q, S: Database>:
    NoCache<'q, S> + Sized
{
    fn execute<E>(
        self,
        executor: E,
    ) -> impl Future<Output = Result<S::QueryResult, sqlx::Error>>
           + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
    {
        let (query, args) = self.build();
        async move {
            InnerExecutable {
                stmt: query.as_str(),
                buffer: args,
                persistent: false,
            }
            .execute(executor)
            .await
        }
    }
    fn fetch_one<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> impl Future<Output = Result<O, sqlx::Error>> + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
        F: FnOnce(S::Row) -> Result<O, sqlx::Error> + Send,
    {
        let (query, args) = self.build();
        async move {
            InnerExecutable {
                stmt: query.as_str(),
                buffer: args,
                persistent: false,
            }
            .fetch_one_with(executor, with)
            .await
        }
    }
    fn fetch_optional<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> impl Future<Output = Result<Option<O>, sqlx::Error>> + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
        F: FnOnce(S::Row) -> Result<O, sqlx::Error> + Send,
    {
        let (query, args) = self.build();
        async move {
            InnerExecutable {
                stmt: query.as_str(),
                buffer: args,
                persistent: false,
            }
            .fetch_optional_with(executor, with)
            .await
        }
    }
    fn fetch_all<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> impl Future<Output = Result<Vec<O>, sqlx::Error>> + Send
    where
        E: for<'e> sqlx::Executor<'e, Database = S>,
        F: FnMut(S::Row) -> Result<O, sqlx::Error> + Send,
    {
        let (query, args) = self.build();
        async move {
            InnerExecutable {
                stmt: query.as_str(),
                buffer: args,
                persistent: false,
            }
            .fetch_all_with(executor, with)
            .await
        }
    }
}

impl<'q, S, T> ExecuteNoCache<'q, S> for T
where
    S: Database,
    T: NoCache<'q, S>,
{
}

pub trait NoCache<'q, S: Database> {
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments);
}

impl<'q, S, Q> NoCache<'q, S> for CreateTableSt<'q, S, Q>
where
    S: Database + SupportNamedBind,
    Q: Query<S, Output = <S as HasArguments<'q>>::Arguments>,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        <Self as Statement<S, Q>>::_build(self)
    }
}

impl<'q, S, Q> NoCache<'q, S> for SelectSt<S, Q>
where
    Q: Query<S, Output = <S as HasArguments<'q>>::Arguments>,
    S: Database + SupportNamedBind,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        self._build()
    }
}

impl<'q, S, Q, R> NoCache<'q, S> for DeleteSt<S, Q, R>
where
    R: ReturningClause,
    S: Database + SupportNamedBind,
    Q: Query<S, Output = <S as HasArguments<'q>>::Arguments>,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        self._build()
    }
}
impl<'q, S, R, Q> NoCache<'q, S> for UpdateSt<S, Q, R>
where
    R: ReturningClause,
    S: Database + SupportNamedBind,
    Q: Query<S, Output = <S as HasArguments<'q>>::Arguments>,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        self._build()
    }
}

impl<'q, S: Database, R> NoCache<'q, S>
    for InsertMany<S, <S as HasArguments<'q>>::Arguments, R>
where
    R: ReturningClause,
    S: Database + SupportNamedBind,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        self._build()
    }
}

impl<'q, S, R> NoCache<'q, S> for InsertStOne<'q, S, R>
where
    R: ReturningClause,
    S: Database + SupportNamedBind,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        self._build()
    }
}
