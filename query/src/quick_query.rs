use crate::InitStatement;

#[cfg(not(feature = "query_v2"))]
pub use v1::QuickQuery;
#[cfg(feature = "query_v2")]
pub use crate::quick_query_v2::QuickQuery;


pub trait InitQuickQuery<'q>: Sized {
    type Init2;
    fn quick_query(init: Self::Init2) -> Self;
}

impl<'q, T> InitQuickQuery<'q> for T
where
    T: InitStatement<QuickQuery<'q>>,
{
    type Init2 = T::Init;
    fn quick_query(init: Self::Init2) -> Self {
        T::init(init)
    }
}

pub mod v1 {
    use std::marker::PhantomData;

    use sqlx::{database::HasArguments, Database, Encode, Type};

    use crate::{
        ident_safety::PanicOnUnsafe, sql_part::{
            AcceptToSqlPart, ColumnToSqlPart,
            ConstraintToSqlPart, ToSqlPart, WhereItemToSqlPart,
        }, Accept, Constraint, Query, SchemaColumn, SupportNamedBind, WhereItem
    };

    pub struct QuickQuery<'q>(PhantomData<&'q ()>);

    impl<'q, S: Database + SupportNamedBind> Query<S>
        for QuickQuery<'q>
    {
        type SqlPart = String;
        type Context1 =
            (usize, <S as HasArguments<'q>>::Arguments);
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

    impl<'q, S, T> ToSqlPart<QuickQuery<'q>, S>
        for WhereItemToSqlPart<T>
    where
        S: Database + SupportNamedBind,
        T: WhereItem<S, QuickQuery<'q>, PanicOnUnsafe>,
        QuickQuery<'q>:
            Query<S, SqlPart = String, Context2 = ()>,
    {
        fn to_sql_part(
            self,
            ctx: &mut <QuickQuery<'q> as Query<S>>::Context1,
        ) -> <QuickQuery<'q> as Query<S>>::SqlPart {
            self.0.where_item(ctx)(&mut ())
        }
    }

    impl<'q, S, T> ToSqlPart<QuickQuery<'q>, S>
        for AcceptToSqlPart<T>
    where
        S: Database + SupportNamedBind,
        QuickQuery<'q>: Accept<T, S>,
        QuickQuery<'q>:
            Query<S, SqlPart = String, Context2 = ()>,
    {
        fn to_sql_part(
            self,
            ctx: &mut <QuickQuery<'q> as Query<S>>::Context1,
        ) -> <QuickQuery<'q> as Query<S>>::SqlPart {
            <QuickQuery<'q> as Accept<T, S>>::accept(self.0, ctx)(
                &mut (),
            )
        }
    }

    impl<'q, S, ToBeAccepted, T> Accept<ToBeAccepted, S>
        for QuickQuery<'q>
    where
        S: Database + SupportNamedBind,
        ToBeAccepted: FnOnce() -> T,
        T: for<'e> Encode<'e, S> + Type<S> + Send + 'q,
    {
        fn accept(
            this: ToBeAccepted,
            ctx1: &mut <Self as Query<S>>::Context1,
        ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
        {
            use sqlx::Arguments;
            ctx1.1.add(this());
            ctx1.0 += 1;
            let len = ctx1.0;
            move |_| format!("${}", len)
        }
    }

    impl<'q, S, T> ToSqlPart<QuickQuery<'q>, S>
        for ConstraintToSqlPart<T>
    where
        S: Database + SupportNamedBind,
        T: Constraint<S, QuickQuery<'q>>,
        QuickQuery<'q>:
            Query<S, SqlPart = String, Context2 = ()>,
    {
        fn to_sql_part(
            self,
            ctx: &mut <QuickQuery<'q> as Query<S>>::Context1,
        ) -> <QuickQuery<'q> as Query<S>>::SqlPart {
            self.0.constraint(ctx)(&mut ())
        }
    }

    impl<'q, S, T> ToSqlPart<QuickQuery<'q>, S>
        for ColumnToSqlPart<T>
    where
        S: Database + SupportNamedBind,
        T: SchemaColumn<S, QuickQuery<'q>>,
        QuickQuery<'q>:
            Query<S, SqlPart = String, Context2 = ()>,
    {
        fn to_sql_part(
            self,
            ctx: &mut <QuickQuery<'q> as Query<S>>::Context1,
        ) -> <QuickQuery<'q> as Query<S>>::SqlPart {
            self.0.column(ctx)(&mut ())
        }
    }
}
