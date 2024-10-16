use futures_util::Future;
use sqlx::{database::HasArguments, Database};

use crate::{
    executable::InnerExecutable, insert_many_st::InsertMany,
    insert_one_st::InsertStOne, returning::ReturningClause,
    string_query::StringQuery, IntoMutArguments, Query,
    Statement, SupportNamedBind,
};

pub trait ExecuteNoCache<'q, S: Database, Q>: Sized {
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments);
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
    #[track_caller]
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

pub trait ExecuteNonSt<'q, S: Database>: Sized {
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments);
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

/// this trait force 'ExecuteNoCache' to be implemented using
/// 'Statement' trait, AND remove any conflicting implementation
/// for non-'Statement' types because Statement<LOCAL, ..> can
/// be implemented by downstream crates for any type
pub trait ExecuteNoCacheUsingSelectTrait {}

impl<'q, S, Q, T> ExecuteNoCache<'q, S, Q> for T
where
    S: Database,
    T: Statement<S, Q> + ExecuteNoCacheUsingSelectTrait,
    Q: Query<S, Output = <S as HasArguments<'q>>::Arguments>,
{
    #[track_caller]
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        <Self as Statement<S, Q>>::_build(self)
    }
}

impl<'q, S, R> ExecuteNoCache<'q, S, ()>
    for InsertStOne<'q, S, R>
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

impl<'q, S: Database, R> ExecuteNoCache<'q, S, ()>
    for InsertMany<S, <S as HasArguments<'q>>::Arguments, R>
where
    R: ReturningClause,
    S: Database,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        self._build()
    }
}

impl<'q, S: Database, I> ExecuteNoCache<'q, S, ()>
    for StringQuery<I>
where
    I: IntoMutArguments<'q, S>,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        let mut args = Default::default();
        self.input.into_arguments(&mut args);
        (self.sql, args)
    }
}
