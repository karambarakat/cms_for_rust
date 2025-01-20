use crate::QueryHandlers;

use std::{marker::PhantomData, ops::Deref, sync::Arc};

use sqlx::{database::HasArguments, Database, Encode, Type};

use crate::{
    ident_safety::PanicOnUnsafe, Accept, BindItem, Query,
    SupportNamedBind,
};

pub struct QuickQuery<'q, S>(PhantomData<(&'q (), S)>);

impl<'q, S: Database + SupportNamedBind> Query
    for QuickQuery<'q, S>
{
    type IdentSafety = PanicOnUnsafe;
    type SqlPart = String;
    type Context1 = (usize, <S as HasArguments<'q>>::Arguments);
    type Context2 = ();
    fn build_sql_part_back(
        _: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String {
        from
    }
    type Output = <S as HasArguments<'q>>::Arguments;
    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        (f(&mut ()), ctx1.1)
    }
}

impl<'q, S> QueryHandlers<S> for QuickQuery<'q, S>
where
    S: Database + SupportNamedBind,
    // needed because the S in this impl may not match the S in Query impl:
    Self: Query<
        SqlPart = String,
        Context1 = (usize, <S as HasArguments<'q>>::Arguments),
        Context2 = (),
    >,
{
    fn handle_accept<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        Self: Accept<T, S>,
    {
        Self::accept(t, ctx)(&mut ())
    }
    fn handle_bind_item<T, I>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: BindItem<S, Self, I> + 'static,
    {
        t.bind_item(ctx)(&mut ())
    }
}

#[cfg(not(feature = "flexible_accept_impl"))]
impl<'q, S, T> Accept<T, S> for QuickQuery<'q, S>
where
    S: Database + SupportNamedBind,
    T: for<'e> Encode<'e, S> + Type<S> + Send + 'q,
{
    fn accept(
        this: T,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        use sqlx::Arguments;
        ctx1.1.add(this);
        ctx1.0 += 1;
        let len = ctx1.0;
        move |_| format!("${}", len)
    }
}

#[cfg(feature = "flexible_accept_impl")]
impl<'q, S, T> Accept<bind<T>, S> for QuickQuery<'q, S>
where
    S: Database + SupportNamedBind,
    T: for<'e> Encode<'e, S> + Type<S> + Send + 'q,
{
    fn accept(
        this: ToBeAccepted,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        use sqlx::Arguments;
        ctx1.1.add(this.0);
        ctx1.0 += 1;
        let len = ctx1.0;
        move |_| format!("${}", len)
    }
}

#[cfg(feature = "flexible_accept_impl")]
impl<'q, S, ToBeAccepted, T> Accept<ToBeAccepted, S>
    for QuickQuery<'q, S>
where
    S: Database + SupportNamedBind,
    ToBeAccepted: FnOnce() -> T,
    T: for<'e> Encode<'e, S> + Type<S> + Send + 'q,
{
    fn accept(
        this: ToBeAccepted,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        use sqlx::Arguments;
        ctx1.1.add(this());
        ctx1.0 += 1;
        let len = ctx1.0;
        move |_| format!("${}", len)
    }
}
