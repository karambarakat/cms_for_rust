use core::fmt;
use sqlx::{
    database::HasArguments, prelude::Type, Database, Encode,
};

use crate::{Accept, Query, Statement};

pub struct DebugQuery;

pub trait Value<S: Database> {
    fn debug(&self) -> String;
    fn clone_to_box(&self) -> Box<dyn Value<S>>;
    fn encode(
        &self,
        buf: &mut <S as HasArguments<'static>>::Arguments,
    );
}

impl<S: Database> Clone for Box<dyn Value<S>> {
    fn clone(&self) -> Self {
        self.clone_to_box()
    }
}

impl<'q, S: Database> Query<S> for DebugQuery {
    type SqlPart = String;
    type Context1 = (usize, Vec<Box<dyn Value<S>>>);
    type Context2 = ();
    fn build_sql_part_back(
        _: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String {
        from
    }
    type Output = <S as HasArguments<'static>>::Arguments;
    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        let mut args = Default::default();
        for value in ctx1.1 {
            value.encode(&mut args);
        }
        (f(&mut ()), args)
    }
}

pub trait DebugQueryMethods<S> {
    fn buffer_display_as(&self) -> Vec<String>;
    fn build_statement(&self) -> String
    where
        Self: Clone;
}

impl<S, T> DebugQueryMethods<S> for T
where
    S: Database,
    T: Statement<S, DebugQuery> + Clone,
{
    fn buffer_display_as(&self) -> Vec<String> {
        let mut str = Vec::default();
        for value in self.deref_ctx().1.iter() {
            str.push(value.debug());
        }
        str
    }
    fn build_statement(&self) -> String {
        let cloned = self.clone();
        cloned._build().0
    }
}

impl<A, A2, S> Accept<A, S> for DebugQuery
where
    S: Database,
    Self:
        Query<S, Context1 = (usize, Vec<Box<dyn fmt::Display>>)>,
    for<'e> A2: Encode<'e, S>
        + Type<S>
        + Send
        + fmt::Display
        + 'static
        + Clone,
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
