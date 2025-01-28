use crate::QueryHandlers;

use std::{marker::PhantomData, ops::Deref, sync::Arc};

use sqlx::{database::HasArguments, Database, Encode, Type};

use crate::{
    ident_safety::PanicOnUnsafe, Accept, BindItem, Query,
    SupportNamedBind,
};

use sqlx::Arguments;

pub struct ClonablQuery<'q, S>(PhantomData<(&'q (), S)>);

pub trait IntoMutBut<'q, S: Database> {
    fn into_mut(
        &self,
        buf: &mut <S as HasArguments<'q>>::Arguments,
    );
    fn clone_to_box(&self) -> Box<dyn IntoMutBut<'q, S> + 'q>;
}

impl<'q, S: Database, T> IntoMutBut<'q, S> for T
where
    T: for<'e> Encode<'e, S> + Type<S> + Send + 'q + Clone,
{
    fn into_mut(
        &self,
        buf: &mut <S as HasArguments<'q>>::Arguments,
    ) {
        let cloned = self.clone();
        buf.add(cloned);
    }
    fn clone_to_box(&self) -> Box<dyn IntoMutBut<'q, S> + 'q> {
        Box::new(Clone::clone(self))
    }
}

pub struct ClonableCtx1<'q, S: Database> {
    size: usize,
    arg: <S as HasArguments<'q>>::Arguments,
    back: Vec<Box<dyn IntoMutBut<'q, S> + 'q>>,
    noop: (),
}

impl<'q, S: Database> From<ClonableCtx1<'q, S>> for () {
    fn from(this: ClonableCtx1<'q, S>) -> Self {
        this.noop
    }
}

impl<'q, S: Database> Default for ClonableCtx1<'q, S> {
    fn default() -> Self {
        ClonableCtx1 {
            size: 0,
            arg: Default::default(),
            back: Default::default(),
            noop: Default::default(),
        }
    }
}

impl<'q, S: Database> Clone for ClonableCtx1<'q, S> {
    fn clone(&self) -> Self {
        let mut arg = Default::default();
        let back = self
            .back
            .iter()
            .map(|e| {
                e.into_mut(&mut arg);
                e.clone_to_box()
            })
            .collect();
        Self {
            size: self.size.clone(),
            arg,
            back,
            noop: self.noop.clone(),
        }
    }
}

impl<S: Database + SupportNamedBind> Query
    for ClonablQuery<'static, S>
{
    type SqlPart = String;
    type Context1 = ClonableCtx1<'static, S>;
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
        let strr = f(&mut ());
        (strr, ctx1.arg)
    }
}

impl<S> QueryHandlers<S> for ClonablQuery<'static, S>
where
    S: Database + SupportNamedBind,
    // needed because the S in this impl may not match the S in Query impl:
    Self: Query<
        SqlPart = String,
        Context1 = ClonableCtx1<'static, S>,
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
        let noop = &mut ctx.noop as &mut ();
        let noop_ptr = unsafe { &mut *(noop as *mut _) };
        Self::accept(t, ctx)(noop_ptr)
    }
    fn handle_bind_item<T, I>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: BindItem<S, Self, I> + 'static,
    {
        let noop = &mut ctx.noop as &mut ();
        let noop_ptr = unsafe { &mut *(noop as *mut _) };
        t.bind_item(ctx)(noop_ptr)
    }
}

#[cfg(not(feature = "flexible_accept_impl"))]
impl<S, T> Accept<T, S> for ClonablQuery<'static, S>
where
    S: Database + SupportNamedBind,
    T: for<'e> Encode<'e, S> + Type<S> + Send + 'static + Clone,
{
    fn accept(
        this: T,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        use sqlx::Arguments;
        let cloned = this.clone();
        ctx1.back.push(Box::new(cloned));
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
        let cloned = this.clone();
        ctx1.back.push(Box::new(cloned));
        ctx1.arg.add(this);
        ctx1.size += 1;
        let len = ctx1.size;
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
        let cloned = this.clone();
        ctx1.back.push(Box::new(cloned));
        ctx1.arg.add(this);
        ctx1.size += 1;
        let len = ctx1.size;
        move |_| format!("${}", len)
    }
}
