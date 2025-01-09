#![allow(unused)]
// #[cfg(todo)]
// pub mod concept_prepared_statement;
pub mod create_table_st;
#[cfg(test)]
pub mod debug_query;
pub mod delete_st;
pub mod executable;
pub mod execute_no_cache;
pub mod expressions;
pub mod ident_safety;
pub mod insert_many_st;
pub mod insert_one_st;
pub mod named_buffer;
pub mod positional_buffer;
pub mod quick_query;
pub(crate) mod quick_query_v2;
pub mod returning;
pub mod sanitize;
pub mod select_st;
pub mod string_query;
pub mod update_st;

pub mod prelude {
    pub use crate::execute_no_cache::ExecuteNoCache;
    pub use crate::InitStatement;

    pub use crate::expressions::exports::*;
    pub use crate::expressions::SelectHelpers2;
    pub use crate::select_st::exports::*;
    pub use crate::select_st::order_by;

    pub fn sanitize<T>(t: T) -> crate::sanitize::Sanitize<T> {
        crate::sanitize::Sanitize(t)
    }
    pub mod stmt {
        pub use crate::create_table_st::CreateTableSt;
        pub use crate::delete_st::DeleteSt;
        pub use crate::insert_many_st::InsertMany;
        pub use crate::insert_one_st::InsertStOne;
        pub use crate::select_st::SelectSt;
        pub use crate::string_query::StringQuery;
        pub use crate::update_st::UpdateSt;
    }
}

pub mod from_row {
    use sqlx::{Database, FromRow};

    /// rely on Sqlx's `FromRow` trait to convert a row
    /// I put this on its own method because sometime
    /// I rely on more dynamic ways to convert a row
    /// and I want to keep sqlx's trait separate
    pub fn sqlx_from_row<T, S>(
    ) -> impl FnMut(S::Row) -> Result<T, sqlx::Error>
    where
        S: Database,
        for<'r> T: FromRow<'r, S::Row>,
    {
        |row| T::from_row(&row)
    }

    pub fn reflexive<S>(
    ) -> impl FnMut(S) -> Result<S, sqlx::Error> {
        |row| Ok(row)
    }
}

use std::marker::PhantomData;

use quick_query::QuickQuery;
use sql_part_::inner::{
    AcceptToSqlPart, ToSqlPart, WhereItemToSqlPart,
};
use sqlx::{database::HasArguments, Database, Postgres, Sqlite};

pub trait TakeParts<'q, S>: Query<S> {
    fn handle_where_item<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: WhereItem<S, Self, ()> + 'q;
    fn handle_accept<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        Self: Accept<T, S>;
    fn handle_constraint<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: Constraint<S, Self> + 'q;
    fn handle_column<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: SchemaColumn<S, Self> + 'q;
}

pub trait Query<S>: Sized {
    type SqlPart;
    type Context1: Default;
    type Context2: Default;

    fn handle_where_item<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: WhereItem<S, Self, ()> + 'static,
    {
        todo!()
    }

    fn handle_accept<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        Self: Accept<T, S>,
    {
        todo!()
    }

    fn build_sql_part_back(
        ctx: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String;

    type Output;
    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output);
}

// #[cfg(not(feature = "build_statments"))]
pub(crate) use sql_part_::inner as sql_part;
// #[cfg(feature = "build_statements")]
// pub use sql_part_::inner as sql_part;

mod sql_part_ {
    pub mod inner {
        /// a given type can implement both WhereItemcannd Accept
        /// in order for rust to convert to SqlPart, it has to know
        /// which impl to target, here there are few new-type structs
        /// for each trait
        use crate::Query;

        pub struct WhereItemToSqlPart<T>(pub T);
        pub struct AcceptToSqlPart<T>(pub T);
        pub struct ConstraintToSqlPart<T>(pub T);
        pub struct ColumnToSqlPart<T>(pub T);

        /// a given type can implement both WhereItem and Accept
        /// in order for rust to convert to SqlPart, it has to know
        /// which impl to target, here there are few new-type structs
        /// for each trait
        /// this has blanket implementation for trivial queries:
        /// SqlPart = String, Context2 = ()
        pub trait ToSqlPart<Q: Query<S>, S> {
            fn to_sql_part(
                self,
                ctx: &mut Q::Context1,
            ) -> Q::SqlPart;
        }
    }
}

pub trait InitStatement<Q>: Sized {
    type Init;
    fn init(init: Self::Init) -> Self;
    #[inline(always)]
    fn init_infer(init: Self::Init, _: PhantomData<Q>) -> Self {
        Self::init(init)
    }
}

pub trait Statement<S, Q: Query<S>> {
    fn deref_ctx(&self) -> &Q::Context1;
    fn deref_mut_ctx(&mut self) -> &mut Q::Context1;
    #[track_caller]
    fn _build(self) -> (String, Q::Output);
}

pub trait SupportNamedBind {}
pub trait SupportReturning {}

pub trait Accept<This, S>: Query<S> {
    fn accept(
        this: This,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send;
}

pub trait SelectItem<S, I> {
    fn select_item(self) -> String;
}

pub trait WhereItem<S, Q: Query<S>, I> {
    fn where_item(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String;
}

pub trait Constraint<S, Q: Query<S>>: Sized {
    fn constraint(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String;
}

pub trait SchemaColumn<S, Q: Query<S>>: Sized {
    fn column(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String;
}

#[rustfmt::skip]
mod impl_schema_cols_for_tuples {
    use crate::SchemaColumn;
    use crate::Query;

    macro_rules! impls {
        ($([$ident:ident, $part:literal]),*) => {
            impl<S, Q: Query<S>, $($ident,)*> SchemaColumn<S, Q> for ($($ident,)*)
            where
            $(
                $ident: SchemaColumn<S, Q>,
            )*
            {
                fn column(
                    self,
                    ctx: &mut Q::Context1,
                ) -> impl FnOnce(&mut Q::Context2) -> String
                {
                    let ptr = ctx as *mut _;
                    let first = (  $( paste::paste! {
                        self.$part.column(unsafe { &mut *ptr })
                    },)* );
                    move |ctx2| {
                        let mut str = Vec::new();

                        paste::paste! { $(
                            str.push(first.$part(ctx2));
                        )* }

                        str.join(" ")
                    }
                }
            }
        };
    }

    impls!([T0, 0]);
    impls!([T0, 0], [T1, 1]);
    impls!([T0, 0], [T1, 1], [T2, 2]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10], [T11, 11]);
}

pub trait IntoMutArguments<'q, DB>
where
    Self: Sized,
    DB: Database,
{
    const LEN: usize;
    fn into_arguments(
        self,
        argument: &mut <DB as HasArguments<'q>>::Arguments,
    );
}

pub mod impl_into_mut_arguments_prelude {
    pub use super::IntoMutArguments;
    pub use sqlx::{
        database::HasArguments, Arguments, Database, Encode,
        Type,
    };
}

#[rustfmt::skip]
mod impl_consume_into_args_for_encode_types {
    use super::impl_into_mut_arguments_prelude::*;
    macro_rules! impls {
        ($len:literal $([$ident:ident, $part:literal])*) => {
            #[allow(unused)]
            impl<'q, DB, $($ident,)*> IntoMutArguments<'q, DB> for ($($ident,)*)
            where
                DB: Database,
                $($ident: Encode<'q, DB> + Type<DB> + Send + 'q,)*
            {
            const LEN : usize = $len;
                fn into_arguments(
                    self,
                    argument: &mut <DB as HasArguments<'q>>::Arguments,
                ) {
                    paste::paste! { $(
                        argument.add(self.$part);
                    )* }
                }
            }
        };
    }

    impls!(0);
    impls!(1 [T0, 0]);
    impls!(2 [T0, 0] [T1, 1]);
    impls!(3 [T0, 0] [T1, 1] [T2, 2]);
    impls!(4 [T0, 0] [T1, 1] [T2, 2] [T3, 3]);
    impls!(5 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4]);
    impls!(6 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5]);
    impls!(7 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6]);
    impls!(8 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7]);
    impls!(9 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8]);
    impls!(10 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9]);
    impls!(11 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9] [T10, 10]);
    impls!(12 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9] [T10, 10] [T11, 11]);

}

mod impls {
    use super::*;
    impl SupportReturning for Sqlite {}
    impl SupportNamedBind for Sqlite {}

    impl SupportReturning for Postgres {}
    impl SupportNamedBind for Postgres {}
}

pub mod connect_in_memory {
    use futures_util::Future;
    use sqlx::{Database, Pool, Sqlite};

    pub trait ConnectInMemory: Send {
        fn connect_in_memory(
        ) -> impl Future<Output = Self> + Send;
    }

    impl ConnectInMemory for Pool<Sqlite> {
        fn connect_in_memory(
        ) -> impl Future<Output = Self> + Send {
            async {
                let pool =
                    sqlx::Pool::connect("sqlite::memory:")
                        .await
                        .unwrap();
                pool
            }
        }
    }
}
