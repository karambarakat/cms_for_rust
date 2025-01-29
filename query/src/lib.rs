#![allow(unused)]
#[cfg(feature = "flexible_accept_impl")]
pub mod accept_extra;
pub mod clonable_query;
pub mod create_table_st;
#[cfg(test)]
pub mod debug_query;
pub mod delete_st;
pub mod executable;
pub mod execute_no_cache;
pub mod expressions_2;
pub mod ident_safety;
pub mod impls;
pub mod insert_many_st;
pub mod insert_one_st;
#[cfg(feature = "todo")]
pub mod positional_query;
pub mod prepared_statement;
pub mod quick_query;
pub mod returning;
pub mod select_st;
pub mod string_query;
#[cfg(feature = "support_non_static_args")]
pub mod support_non_static_args_mod;
#[cfg(feature = "support_non_static_args")]
pub use support_non_static_args_mod::*;
pub mod update_st;

pub mod prelude {
    pub use crate::execute_no_cache::ExecuteNoCache;
    pub use crate::expressions_2::exports::*;
    // pub use crate::expressions::SelectHelpers2;
    pub use crate::select_st::join;
    pub use crate::select_st::order_by;

    #[cfg(feature = "flexible_accept_impl")]
    pub use crate::accept_extra::exports::*;

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
    ///
    /// rust fn_impl will make this obselete
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

use std::{
    fmt::{format, Display},
    marker::PhantomData,
};

use sqlx::{database::HasArguments, Database, Postgres, Sqlite};

#[cfg(not(feature = "support_non_static_args"))]
pub trait Query: Sized {
    type SqlPart;
    type Context1: Default + 'static;
    type Context2: From<Self::Context1>;

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

#[cfg(not(feature = "support_non_static_args"))]
pub trait QueryHandlers<S>: Query {
    fn handle_bind_item<T, I>(
        t: T,
        ctx: &mut <Self as Query>::Context1,
    ) -> Self::SqlPart
    where
        T: BindItem<S, Self, I> + 'static;

    fn handle_accept<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: 'static + Send,
        Self: Accept<T, S>;
}

#[allow(non_camel_case_types)]
pub struct bind<T>(pub T);

#[cfg(not(feature = "support_non_static_args"))]
pub trait Statement<S, Q: Query> {
    fn deref_ctx(&self) -> &Q::Context1;
    fn deref_mut_ctx(&mut self) -> &mut Q::Context1;
    #[track_caller]
    fn _build(self) -> (String, Q::Output);
}

pub trait SupportNamedBind {}
pub trait SupportReturning {}

pub trait IdentSafety: 'static {
    type Table: AsRef<str>;
    type Column: AsRef<str>;
    #[track_caller]
    fn check_other<T: AsRef<str>>(any_: T);
    fn init<T: AsRef<str>>(on_table: Option<&T>) -> Self;
}

pub trait AcceptTableIdent<T>: IdentSafety {
    #[track_caller]
    fn into_table(this: T) -> Self::Table;
}
pub trait AcceptColIdent<T>: IdentSafety {
    #[track_caller]
    fn into_col(this: T) -> Self::Column;
}

#[cfg(not(feature = "support_non_static_args"))]
pub trait Accept<This, S>: QueryHandlers<S> + Send {
    fn accept(
        this: This,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send;
}

#[cfg(not(feature = "support_non_static_args"))]
pub trait BindItem<S, Q, I>
where
    Q::Context1: 'static,
    Q: Query,
{
    fn bind_item(
        self,
        ctx: &mut Q::Context1,
    ) -> impl FnOnce(&mut Q::Context2) -> String + 'static;
}

pub trait NonBindItem: Display + 'static {
    type I: IdentSafety;
}

// impl<S, Q: Query, I, T> BindItem<S, Q, I> for T
// where
//     T: NonBindItem,
// {
//     fn bind_item(
//         self,
//         ctx: &mut <Q as Query>::Context1,
//     ) -> impl FnOnce(&mut <Q as Query>::Context2) -> String + 'static
//     {
//         move |_| format!("{}", self)
//     }
// }

pub trait WhereItem {}
pub trait Constraint {}
pub trait SchemaColumn<S> {
    fn display(&self) -> String;
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

mod impls_for_sqlx {
    use super::*;
    impl SupportReturning for Sqlite {}
    impl SupportNamedBind for Sqlite {}

    impl SupportReturning for Postgres {}
    impl SupportNamedBind for Postgres {}
}
