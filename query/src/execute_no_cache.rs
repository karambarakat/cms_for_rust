use futures_util::Future;
use sqlx::{database::HasArguments, Database};

use crate::{executable::InnerExecutable, Query, Statement};

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

/// this is temporary until I figure out what to do with
/// statement that do not contain any Query
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

impl<'q, S, Q, T> ExecuteNoCache<'q, S, Q> for T
where
    S: Database,
    T: Statement<S, Q>,
    Q: Query<S, Output = <S as HasArguments<'q>>::Arguments>,
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        <Self as Statement<S, Q>>::_build(self)
    }
}

// impl<'q, S: Database, R> NoCache<'q, S>
//     for InsertMany<S, <S as HasArguments<'q>>::Arguments, R>
// where
//     R: ReturningClause,
//     S: Database + SupportNamedBind,
// {
//     fn build(
//         self,
//     ) -> (String, <S as HasArguments<'q>>::Arguments) {
//         self._build()
//     }
// }
//
// impl<'q, S, R> NoCache<'q, S> for InsertStOne<'q, S, R>
// where
//     R: ReturningClause,
//     S: Database + SupportNamedBind,
// {
//     fn build(
//         self,
//     ) -> (String, <S as HasArguments<'q>>::Arguments) {
//         self._build()
//     }
// }
