use crate::QueryHandlers;

use std::{marker::PhantomData, ops::Deref, sync::Arc};

use sqlx::{database::HasArguments, Database, Encode, Type};

use crate::{
    ident_safety::PanicOnUnsafe, Accept, BindItem, Query,
    SupportNamedBind,
};

pub struct QuickQuery<S>(PhantomData<S>);

pub struct QuickQueryCtx1<S: Database> {
    size: usize,
    arg: <S as HasArguments<'static>>::Arguments,
    noop: (),
}

impl<S: Database> Default for QuickQueryCtx1<S> {
    fn default() -> Self {
        QuickQueryCtx1 {
            size: 0,
            arg: Default::default(),
            noop: Default::default(),
        }
    }
}

impl<S: Database> From<QuickQueryCtx1<S>> for () {
    fn from(this: QuickQueryCtx1<S>) -> Self {
        this.noop
    }
}

impl<S: Database + SupportNamedBind> Query for QuickQuery<S> {
    type SqlPart = String;
    type Context1 = QuickQueryCtx1<S>;
    type Context2 = ();
    fn build_sql_part_back(
        _: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String {
        from
    }
    type Output = <S as HasArguments<'static>>::Arguments;
    fn build_query(
        mut ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        // I wonder if I can remove this in the future
        let noop = unsafe { &mut *(&mut ctx1.noop as *mut _) };
        let strr = f(noop);
        (strr, ctx1.arg)
    }
}

impl<S> QueryHandlers<S> for QuickQuery<S>
where
    S: Database + SupportNamedBind,
    // needed because the S in this impl may not match the S in Query impl:
    Self: Query<
        SqlPart = String,
        Context1 = QuickQueryCtx1<S>,
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
        let cc = &mut ctx.noop as &mut ();
        let cc2 = unsafe { &mut *(cc as *mut _) };
        Self::accept(t, ctx)(cc2)
    }
    fn handle_bind_item<T, I>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: BindItem<S, Self, I> + 'static,
    {
        let cc = &mut ctx.noop as &mut ();
        let cc2 = unsafe { &mut *(cc as *mut _) };
        t.bind_item(ctx)(cc2)
    }
}

#[cfg(not(feature = "flexible_accept_impl"))]
impl<S, T> Accept<T, S> for QuickQuery<S>
where
    S: Database + SupportNamedBind,
    T: for<'e> Encode<'e, S> + Type<S> + Send + 'static,
{
    fn accept(
        this: T,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + Send + 'static
    {
        use sqlx::Arguments;
        ctx1.arg.add(this);
        ctx1.size += 1;
        let len = ctx1.size;
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
