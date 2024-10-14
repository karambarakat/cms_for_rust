use futures_util::{future::BoxFuture, stream::BoxStream};
use sqlx::{
    database::HasStatement, Database, Describe, Either, Execute,
    Executor,
};

pub struct DebugSql<T, C>(pub T, pub C);

impl<T, C> std::fmt::Debug for DebugSql<T, C>
where
    T: std::fmt::Debug,
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_tuple("DebugStatement").field(&self.0).finish()
    }
}

impl<'c, T, C> Executor<'c> for DebugSql<T, C>
where
    T: Executor<'c>,
    C: FnOnce(String) + Send,
{
    type Database = T::Database;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<
        'e,
        Result<
            Either<
                <Self::Database as Database>::QueryResult,
                <Self::Database as Database>::Row,
            >,
            sqlx::Error,
        >,
    >
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        self.1(query.sql().to_owned());
        self.0.fetch_many(query)
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<
        'e,
        Result<
            Option<<Self::Database as Database>::Row>,
            sqlx::Error,
        >,
    >
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        self.1(query.sql().to_owned());
        self.0.fetch_optional(query)
    }
    fn execute<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<
        'e,
        Result<
            <Self::Database as Database>::QueryResult,
            sqlx::Error,
        >,
    >
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        self.1(query.sql().to_owned());
        self.0.execute(query)
    }
    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) -> BoxFuture<
        'e,
        Result<
            <Self::Database as HasStatement<'q>>::Statement,
            sqlx::Error,
        >,
    >
    where
        'c: 'e,
    {
        self.0.prepare_with(sql, parameters)
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<
        'e,
        Result<Describe<Self::Database>, sqlx::Error>,
    >
    where
        'c: 'e,
    {
        self.0.describe(sql)
    }
}
