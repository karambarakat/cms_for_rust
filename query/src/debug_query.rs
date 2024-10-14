use core::fmt;

use sqlx::{prelude::Type, Database, Encode};

use crate::{Accept, Query};

pub struct DebugQuery;

impl<'q, S: Database> Query<S> for DebugQuery {
    type SqlPart = String;
    type Context1 = (usize, Vec<Box<dyn fmt::Display>>);
    type Context2 = ();
    fn build_sql_part_back(
        _: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String {
        from
    }
    type Output = Vec<Box<dyn fmt::Display>>;
    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        (f(&mut ()), ctx1.1)
    }
}

impl<A, A2, S> Accept<A, S> for DebugQuery
where
    S: Database,
    Self:
        Query<S, Context1 = (usize, Vec<Box<dyn fmt::Display>>)>,
    for<'e> A2:
        Encode<'e, S> + Type<S> + Send + fmt::Display + 'static,
    A: FnOnce() -> A2 + 'static,
{
    fn accept(
        this: A,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static
    {
        ctx1.1.push(Box::new(this()));
        ctx1.0 += 1;
        let len = ctx1.0;
        move |_| format!("${}", len)
    }
}
