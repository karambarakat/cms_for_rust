use std::marker::PhantomData;

use sqlx::{
    database::HasArguments, prelude::Type, Database, Encode,
};

use crate::{Accept, Query, SupportNamedBind, WhereItem};

pub struct QuickQuery<'q>(PhantomData<&'q ()>);

impl<'q, S: Database + SupportNamedBind> Query<S>
    for QuickQuery<'q>
{
    type SqlPart = String;
    type Context1 = (usize, <S as HasArguments<'q>>::Arguments);
    type Context2 = ();
    fn build_sql_part_back(
        _: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String {
        from
    }
    fn handle_where_item(
        item: impl WhereItem<S, Self> + 'static,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart {
        let item = item.where_item(ctx);
        item(&mut ())
    }
    type Output = <S as HasArguments<'q>>::Arguments;
    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        (f(&mut ()), ctx1.1)
    }
}

impl<'q, A, A2, S> Accept<A, S> for QuickQuery<'q>
where
    S: Database,
    Self: Query<
        S,
        Context1 = (usize, <S as HasArguments<'q>>::Arguments),
    >,
    for<'e> A2: Encode<'e, S> + Type<S> + Send + 'q,
    A: FnOnce() -> A2 + 'static,
{
    fn accept(
        this: A,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static
    {
        use sqlx::Arguments;
        ctx1.1.add(this());
        ctx1.0 += 1;
        let len = ctx1.0;
        move |_| format!("${}", len)
    }
}
