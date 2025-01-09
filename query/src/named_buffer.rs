use std::marker::PhantomData;

use sqlx::{
    database::HasArguments, prelude::Type, Database, Encode,
};

use crate::{
    ident_safety::PanicOnUnsafe, sql_part::{
        AcceptToSqlPart, ColumnToSqlPart, ConstraintToSqlPart,
        ToSqlPart, WhereItemToSqlPart,
    }, Accept, Constraint, Query, SchemaColumn, SupportNamedBind, WhereItem
};

pub struct NamedBorrowedBuffer<'q>(PhantomData<&'q ()>);

// 'SupportNamedBind' requiredment is important because I don't want to
// to use NamedBorrowedBuffer with databases that don't support
// $1 syntax
impl<'q, S: Database + SupportNamedBind> Query<S>
    for NamedBorrowedBuffer<'q>
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
    type Output = <S as HasArguments<'q>>::Arguments;
    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        (f(&mut ()), ctx1.1)
    }
}

impl<'q, A, A2, S> Accept<A, S> for NamedBorrowedBuffer<'q>
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

impl<'q, T, S> ToSqlPart<NamedBorrowedBuffer<'q>, S>
    for WhereItemToSqlPart<T>
where
    S: Database + SupportNamedBind,
    T: WhereItem<S, NamedBorrowedBuffer<'q>, PanicOnUnsafe> + 'static,
{
    fn to_sql_part(
        self,
        ctx: &mut <NamedBorrowedBuffer<'q> as Query<S>>::Context1,
    ) -> <NamedBorrowedBuffer<'q> as Query<S>>::SqlPart {
        self.0.where_item(ctx)(&mut ())
    }
}

impl<'q, S, T> ToSqlPart<NamedBorrowedBuffer<'q>, S>
    for AcceptToSqlPart<T>
where
    S: Database + SupportNamedBind,
    NamedBorrowedBuffer<'q>: Accept<T, S> + 'static,
    NamedBorrowedBuffer<'q>: Query<
        S,
        SqlPart = String,
        Context1 = (usize, <S as HasArguments<'q>>::Arguments),
        Context2 = (),
    >,
{
    fn to_sql_part(
        self,
        ctx: &mut <NamedBorrowedBuffer<'q> as Query<S>>::Context1,
    ) -> <NamedBorrowedBuffer<'q> as Query<S>>::SqlPart {
        <NamedBorrowedBuffer<'q> as Accept<T, S>>::accept(
            self.0, ctx,
        )(&mut ())
    }
}

impl<'q, S, T> ToSqlPart<NamedBorrowedBuffer<'q>, S>
    for ConstraintToSqlPart<T>
where
    S: Database + SupportNamedBind,
    T: Constraint<S, NamedBorrowedBuffer<'q>> + 'static,
{
    fn to_sql_part(
        self,
        ctx: &mut <NamedBorrowedBuffer<'q> as Query<S>>::Context1,
    ) -> <NamedBorrowedBuffer<'q> as Query<S>>::SqlPart {
        self.0.constraint(ctx)(&mut ())
    }
}

impl<'q, S, T> ToSqlPart<NamedBorrowedBuffer<'q>, S>
    for ColumnToSqlPart<T>
where
    S: Database + SupportNamedBind,
    T: SchemaColumn<S, NamedBorrowedBuffer<'q>> + 'static,
{
    fn to_sql_part(
        self,
        ctx: &mut <NamedBorrowedBuffer<'q> as Query<S>>::Context1,
    ) -> <NamedBorrowedBuffer<'q> as Query<S>>::SqlPart {
        self.0.column(ctx)(&mut ())
    }
}
