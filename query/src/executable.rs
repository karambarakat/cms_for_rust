use std::mem::take;

use sqlx::{
    database::HasArguments, Database, Execute, Executor,
};
use tracing::debug;

/// like sqlx::Query* but allow binding via &mut self
mod _private {
    use sqlx::{database::HasArguments, Database};

    pub struct InnerExecutable<'s, 'q, DB: Database> {
        pub stmt: &'s str,
        pub buffer: <DB as HasArguments<'q>>::Arguments,
        pub persistent: bool,
    }
}

#[cfg(feature = "export_inner_executable")]
pub use _private::InnerExecutable;
#[cfg(not(feature = "export_inner_executable"))]
pub(crate) use _private::InnerExecutable;

impl<'q, DB: Database> Execute<'q, DB>
    for InnerExecutable<'q, 'q, DB>
{
    fn sql(&self) -> &'q str {
        self.stmt
    }

    fn persistent(&self) -> bool {
        self.persistent
    }

    fn statement(
        &self,
    ) -> Option<
        &<DB as sqlx::database::HasStatement<'q>>::Statement,
    > {
        None
    }

    fn take_arguments(
        &mut self,
    ) -> Option<<DB as HasArguments<'q>>::Arguments> {
        Some(take(&mut self.buffer))
    }
}

impl<'s, 'q, S: Database> InnerExecutable<'s, 'q, S> {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.stmt
    }
    pub async fn execute<E>(
        self,
        executor: E,
    ) -> Result<S::QueryResult, sqlx::Error>
    where
        for<'c> E: Executor<'q, Database = S>,
    {
        debug!("execute: {}", self.stmt);
        executor
            .execute(InnerExecutable {
                // SAFETY: the output of execute is free of
                // any reference of self, which means that
                // self can drop after the await, and the
                // result can live longer
                //
                // I tried Self: 'q, and &'q mut self can
                // be used to solve this issue
                //
                // I saw the same issue before, this is
                // either a problem in sqlx or rust is not
                // advanced enough to catch this pattern, but
                // i'm sure this code is 100% safe
                stmt: unsafe { &*(self.stmt as *const _) },
                ..self
            })
            .await
    }

    pub async fn fetch_one_with<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> Result<O, sqlx::Error>
    where
        F: FnOnce(S::Row) -> Result<O, sqlx::Error>,
        for<'c> E: Executor<'c, Database = S>,
    {
        debug!("fetch one: {}", self.stmt);
        let execute = InnerExecutable {
            // SAFETY: the output of execute is free of
            // any reference of self, which means that
            // self can drop after the await, and the
            // result can live longer
            //
            // I tried Self: 'q, and &'q mut self can
            // be used to solve this issue
            //
            // I saw the same issue before, this is
            // either a problem in sqlx or rust is not
            // advanced enough to catch this pattern, but
            // i'm sure this code is 100% safe
            stmt: unsafe { &*(self.stmt as *const _) },
            ..self
        };

        let res = executor.fetch_one(execute).await;

        match res {
            Ok(r) => Ok(with(r)?),
            Err(e) => Err(e),
        }
    }

    pub async fn fetch_all_with<E, O, F>(
        self,
        executor: E,
        mut with: F,
    ) -> Result<Vec<O>, sqlx::Error>
    where
        F: FnMut(S::Row) -> Result<O, sqlx::Error>,
        for<'c> E: Executor<'c, Database = S>,
    {
        debug!("fetch all: {}", self.stmt);
        let execute = InnerExecutable {
            // SAFETY: the output of execute is free of
            // any reference of self, which means that
            // self can drop after the await, and the
            // result can live longer
            //
            // I tried Self: 'q, and &'q mut self can
            // be used to solve this issue
            //
            // I saw the same issue before, this is
            // either a problem in sqlx or rust is not
            // advanced enough to catch this pattern, but
            // i'm sure this code is 100% safe
            stmt: unsafe { &*(self.stmt as *const _) },
            ..self
        };

        executor.fetch_all(execute).await.map(|r| {
            r.into_iter()
                .map(|r| with(r).expect("failed to decode"))
                .collect::<Vec<_>>()
        })
    }

    pub async fn fetch_optional_with<E, O, F>(
        self,
        executor: E,
        with: F,
    ) -> Result<Option<O>, sqlx::Error>
    where
        F: FnOnce(S::Row) -> Result<O, sqlx::Error>,
        for<'c> E: Executor<'c, Database = S>,
    {
        debug!("fetch optional: {}", self.stmt);
        let execute = InnerExecutable {
            // SAFETY: the output of execute is free of
            // any reference of self, which means that
            // self can drop after the await, and the
            // result can live longer
            //
            // I tried Self: 'q, and &'q mut self can
            // be used to solve this issue
            //
            // I saw the same issue before, this is
            // either a problem in sqlx or rust is not
            // advanced enough to catch this pattern, but
            // i'm sure this code is 100% safe
            stmt: unsafe { &*(self.stmt as *const _) },
            ..self
        };

        let op = executor.fetch_optional(execute).await;

        match op {
            Ok(Some(r)) => Ok(Some(with(r)?)),
            _ => Ok(None),
        }
    }
}
