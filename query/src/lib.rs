// pub mod create_table_st;
pub mod create_table_st;
#[cfg(test)]
pub mod debug_query;
pub mod debug_sql;
pub mod delete_st;
pub mod executable;
pub mod execute_no_cache;
pub mod expressions;
pub mod insert_many_st;
pub mod insert_one_st;
pub mod quick_query;
pub mod returning;
pub mod sanitize;
pub mod select_st;
pub mod string_query;
pub mod update_st;

pub mod macros {
    pub use query_macros::*;
}

pub mod prelude {
    pub use crate::execute_no_cache::ExecuteNoCache;
    pub use crate::execute_no_cache::ExecuteNonSt;

    pub use crate::expressions::exports::*;
    pub use crate::expressions::SelectHelpers2;
    use crate::sanitize::Sanitize;
    pub use crate::select_st::exports::*;
    pub use crate::select_st::joins::join_type;
    pub use crate::select_st::order_by;

    pub fn sanitize<T>(t: T) -> Sanitize<T> {
        Sanitize(t)
    }

    pub mod stmt {
        use crate::create_table_st::CreateTableSt;
        use crate::insert_one_st::InsertStOne;
        use crate::quick_query::QuickQuery;
        use crate::string_query::StringQuery;
        use crate::Query;
        use crate::SupportNamedBind;
        use sqlx::Database;
        use std::marker::PhantomData;

        pub use crate::insert_many_st::insert_many;

        pub fn insert_one<S>(
            from: &'static str,
        ) -> InsertStOne<S>
        where
            S: Database,
        {
            InsertStOne {
                input: Vec::new(),
                output: None,
                from,
                buffer: Default::default(),
                _pd: PhantomData,
                returning: (),
            }
        }

        pub fn create_table_if_not_exists<'q, S>(
            name: &'static str,
        ) -> CreateTableSt<S, QuickQuery<'q>>
        where
            QuickQuery<'q>: Query<S>,
        {
            CreateTableSt {
                header: "CREATE TABLE IF NOT EXISTS".to_string(),
                ident: (None, name.to_string()),
                columns: Vec::new(),
                ctx: Default::default(),
                _sqlx: Default::default(),
                verbatim: Default::default(),
                constraints: Default::default(),
            }
        }

        pub fn string_query<I>(
            query: String,
            input: I,
        ) -> StringQuery<I> {
            StringQuery { sql: query, input }
        }

        pub fn select<'q, S>(
            table: &'static str,
        ) -> crate::select_st::SelectSt<S, QuickQuery<'q>>
        where
            S: Database + SupportNamedBind,
        {
            crate::select_st::SelectSt {
                select_list: Default::default(),
                where_clause: Default::default(),
                joins: Default::default(),
                ctx: Default::default(),
                order_by: Default::default(),
                limit: Default::default(),
                shift: Default::default(),
                from: table,
                _sqlx: Default::default(),
            }
        }

        pub fn update<'q, S>(
            table: &'static str,
        ) -> crate::update_st::UpdateSt<S, QuickQuery<'q>, ()>
        where
            S: Database + SupportNamedBind,
        {
            crate::update_st::UpdateSt {
                sets: Default::default(),
                where_clause: Default::default(),
                ctx: Default::default(),
                table,
                returning: (),
                _sqlx: Default::default(),
            }
        }

        pub fn delete<'q, S>(
            table: &'static str,
        ) -> crate::delete_st::DeleteSt<S, QuickQuery<'q>>
        where
            S: Database + SupportNamedBind,
        {
            crate::delete_st::DeleteSt {
                where_clause: Default::default(),
                ctx: Default::default(),
                table,
                returning: (),
                _sqlx: Default::default(),
            }
        }
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

use sqlx::{database::HasArguments, Database, Postgres, Sqlite};

pub trait Query<S>: Sized {
    type SqlPart;
    type Context1: Default;
    type Context2: Default;
    #[deprecated = "in favor of ToSqlPart"]
    fn handle_where_item(
        _: impl WhereItem<S, Self> + 'static,
        _: &mut Self::Context1,
    ) -> Self::SqlPart {
        panic!("depricate in favor of ToSqlPart")
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

#[cfg(not(feature = "build_statments"))]
pub(crate) use sql_part_::inner as sql_part;
#[cfg(feature = "build_statements")]
pub use sql_part_::inner as sql_part;

mod sql_part_ {
    pub mod inner {
        use std::marker::PhantomData;

        use sqlx::Database;

        /// a given type can implement both WhereItem and Accept
        /// in order for rust to convert to SqlPart, it has to know
        /// which impl to target, here there are few new-type structs
        /// for each trait
        use crate::{
            Accept, Constraint, Query, SchemaColumn, WhereItem,
        };

        pub struct WhereItemToSqlPart<T>(pub T);
        pub struct AcceptToSqlPart<T>(pub T);
        pub struct ConstraintToSqlPart<T, Q>(
            pub T,
            pub PhantomData<Q>,
        );
        pub struct ColumnToSqlPart<T, Q>(
            pub T,
            pub PhantomData<Q>,
        );

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

        impl<Q, S, T> ToSqlPart<Q, S> for WhereItemToSqlPart<T>
        where
            S: Database,
            T: WhereItem<S, Q>,
            Q: Query<S, SqlPart = String, Context2 = ()>,
        {
            fn to_sql_part(
                self,
                ctx: &mut <Q as Query<S>>::Context1,
            ) -> <Q as Query<S>>::SqlPart
            where
                Q: Query<S>,
            {
                let item = self.0.where_item(ctx);
                item(&mut ())
            }
        }

        impl<'q, Q, S, T> ToSqlPart<Q, S> for ConstraintToSqlPart<T, Q>
        where
            S: Database,
            T: Constraint<S, Q>,
            Q: Query<S, SqlPart = String, Context2 = ()>,
        {
            fn to_sql_part(
                self,
                ctx: &mut <Q as Query<S>>::Context1,
            ) -> <Q as Query<S>>::SqlPart
            where
                Q: Query<S>,
            {
                self.0.constraint(ctx)(&mut ())
            }
        }

        impl<'q, Q, S, T> ToSqlPart<Q, S>
            for crate::sql_part::ColumnToSqlPart<T, Q>
        where
            S: Database,
            T: SchemaColumn<S, Q>,
            Q: Query<S, SqlPart = String, Context2 = ()>,
        {
            fn to_sql_part(
                self,
                ctx: &mut <Q as Query<S>>::Context1,
            ) -> <Q as Query<S>>::SqlPart
            where
                Q: Query<S>,
            {
                self.0.column(ctx)(&mut ())
            }
        }
        impl<Q, S, T> ToSqlPart<Q, S> for AcceptToSqlPart<T>
        where
            Q: Accept<T, S>,
            Q: Query<S, SqlPart = String, Context2 = ()>,
        {
            fn to_sql_part(
                self,
                ctx: &mut <Q as Query<S>>::Context1,
            ) -> <Q as Query<S>>::SqlPart
            where
                Q: Query<S>,
            {
                <Q as Accept<T, S>>::accept(self.0, ctx)(&mut ())
            }
        }
    }
}

pub trait Statement<S, Q: Query<S>> {
    type Init;
    fn init(init: Self::Init) -> Self;
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

pub trait SelectItem<S> {
    fn select_item(self) -> String;
}

pub trait WhereItem<S, Q: Query<S>> {
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
    fn into_arguments(
        self,
        argument: &mut <DB as HasArguments<'q>>::Arguments,
    );
}

#[cfg(todo)]
mod into_mut_generalization {
    use core::hash;
    use std::any::Any;
    use std::marker::PhantomData;
    use std::ops::Not;

    use futures_util::Future;
    use sqlx::database::HasArguments;
    use sqlx::{Arguments, Database, Pool, Sqlite};

    use crate::create_table_st::constraints::exports::default;
    use crate::executable::InnerExecutable;
    use crate::select_st::SelectSt;
    use crate::{Accept, IntoMutArguments, Query, Statement};

    pub trait IntoMutArgumentsMeta<'q, DB>:
        IntoMutArguments<'q, DB>
    where
        DB: Database,
    {
        type Meta: BindMeta;
    }

    pub trait BindMeta: Sized {
        fn shift(self, by: usize) -> Self;
        fn names(&self) -> &[&'static str];
    }

    struct DynamicQuery<S>(PhantomData<S>);

    impl<S> DynamicQuery<S> {
        pub fn build<St: Statement<S, Self>>(
            st: St,
        ) -> (CachedExecute<S>, DynamicDeserializer<S>) {
            let str = st._build();
            todo!()
        }
    }

    impl<S> Query<S> for DynamicQuery<S> {
        type SqlPart = String;

        type Context1 = Vec<Input>;

        type Context2 = ();

        fn build_sql_part_back(
            ctx: &mut Self::Context2,
            from: Self::SqlPart,
        ) -> String {
            todo!()
        }

        type Output = ();

        fn build_query(
            ctx1: Self::Context1,
            f: impl FnOnce(&mut Self::Context2) -> String,
        ) -> (String, Self::Output) {
            todo!()
        }
    }

    struct Input(&'static str, Box<dyn Ty>);

    trait Ty {}
    impl<T: 'static> Ty for PhantomData<T> {}

    fn input<T: 'static>(name: &'static str) -> Input {
        Input(name, Box::new(PhantomData::<T>))
    }

    impl<S> Accept<Input, S> for DynamicQuery<S> {
        fn accept(
            this: Input,
            ctx1: &mut Self::Context1,
        ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
        {
            ctx1.push(this);
            let len = ctx1.len();
            move |_| format!("${}", len)
        }
    }

    struct CachedExecute<S>(String, PhantomData<S>);

    struct H(std::collections::HashMap<(), ()>);

    impl<S: Database> CachedExecute<S> {
        fn execute<E, H: hash::Hash + 'static>(
            self,
            executor: E,
            // 'static restriction here can be lifted if I
            // restrict the output future to execute before
            // an lifetime 'q
            // the lifetime is just for Sqlite value that
            // can be some Cow<'q, str>
            input: (H, <S as HasArguments<'static>>::Arguments),
        ) -> impl Future<
            Output = Result<S::QueryResult, sqlx::Error>,
        > + Send
        where
            E: for<'e> sqlx::Executor<'e, Database = S>,
        {
            if <H as Any>::type_id(&input.0)
                .eq(&().type_id())
                .not()
            {
                todo!("chached statment")
            }
            async move {
                InnerExecutable {
                    stmt: self.0.as_str(),
                    buffer: input.1,
                    persistent: false,
                }
                .execute(executor)
                .await
            }
        }
    }

    struct DynamicDeserializer<S>(PhantomData<S>);

    impl<S: Database> DynamicDeserializer<S> {
        pub fn input(
            &self,
            input: serde_json::Value,
        ) -> Result<
            ((), <S as HasArguments<'static>>::Arguments),
            serde_json::Error,
        > {
            todo!()
        }
    }

    async fn concept() {
        use serde_json::json;

        use crate::{
            expressions::exports::{all_columns, col},
            from_row,
            prelude::stmt,
        };

        let mut st =
            SelectSt::<Sqlite, DynamicQuery<Sqlite>>::init(
                "Users",
            );

        st.select(all_columns());

        st.where_(col("id").eq(input::<i32>("by_id")));
        st.limit(input::<i32>("limit"));

        let (prepared_st, des) = DynamicQuery::build(st);

        let pool = Pool::<Sqlite>::connect("").await.unwrap();

        let dynamic_value = json!({
            "limit": 3,
            "by_id": 1,
        });

        prepared_st
            .execute(
                &pool,
                des.input(dynamic_value)
                    .expect("format of value is invalid"),
            )
            .await
            .unwrap();
    }
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
        ($([$ident:ident, $part:literal])*) => {
            #[allow(unused)]
            impl<'q, DB, $($ident,)*> IntoMutArguments<'q, DB> for ($($ident,)*)
            where
                DB: Database,
                $($ident: Encode<'q, DB> + Type<DB> + Send + 'q,)*
            {
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

    impls!();
    impls!([T0, 0]);
    impls!([T0, 0] [T1, 1]);
    impls!([T0, 0] [T1, 1] [T2, 2]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9] [T10, 10]);
    impls!([T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9] [T10, 10] [T11, 11]);

}

mod impls {
    use super::*;
    impl SupportReturning for Sqlite {}
    impl SupportNamedBind for Sqlite {}

    impl SupportReturning for Postgres {}
    impl SupportNamedBind for Postgres {}
}

#[cfg(test)]
mod positional_buffer_concept {
    use crate::{
        sql_part::{ToSqlPart, WhereItemToSqlPart},
        Accept, Query, WhereItem,
    };
    use sqlx::MySql;
    use std::{marker::PhantomData, mem::take};

    pub trait DebugAny
    where
        Self: std::fmt::Debug,
    {
        fn ty(&self) -> &'static str;
        fn debug(&self) -> String {
            format!("{}: {:?}", self.ty(), self)
        }
    }

    #[derive(Debug, Default)]
    pub struct MockMySql;
    impl<T> ToSqlPart<MockMySql, MySql> for WhereItemToSqlPart<T>
    where
        T: WhereItem<MySql, MockMySql> + 'static,
    {
        fn to_sql_part(
            self,
            ctx: &mut <MockMySql as Query<MySql>>::Context1,
        ) -> <MockMySql as Query<MySql>>::SqlPart {
            let ctx2 = unsafe { &mut *(ctx as *mut _) };
            let item = self.0.where_item(ctx2);
            Box::new(move |ctx2| item(ctx2))
        }
    }
    impl Query<MySql> for MockMySql {
        type SqlPart =
            Box<dyn FnOnce(&mut Self::Context2) -> String>;

        type Context1 =
            Vec<Option<Box<dyn FnOnce() -> Box<dyn DebugAny>>>>;

        type Context2 = (
            Vec<Option<Box<dyn FnOnce() -> Box<dyn DebugAny>>>>,
            Vec<Box<dyn DebugAny>>,
        );

        fn build_sql_part_back(
            ctx: &mut Self::Context2,
            from: Self::SqlPart,
        ) -> String {
            from(ctx)
        }

        type Output = ();
        fn build_query(
            _: Self::Context1,
            _: impl FnOnce(&mut Self::Context2) -> String,
        ) -> (String, Self::Output) {
            panic!("just an example")
        }
    }

    macro_rules! debug_any {
    ($($ident:ident), *) => {
        $(impl DebugAny for $ident
        where $ident: std::fmt::Debug
        {
            fn ty(&self) -> &'static str {
                stringify!($ident)
            }
        })*
    };
}

    debug_any!(String, i8, i16, i32, u8, u16, u32, u64, bool);

    impl<A, T> Accept<A, MySql> for MockMySql
    where
        A: FnOnce() -> T + 'static,
        T: DebugAny + 'static,
    {
        fn accept(
            this: A,
            ctx1: &mut Self::Context1,
        ) -> impl FnOnce(&mut Self::Context2) -> String + 'static
        {
            ctx1.push(Some(Box::new(|| Box::new(this()))));
            let len = ctx1.len();

            move |ctx2| {
                let found = take(
                    ctx2.0.get_mut(len - 1).expect("overflow"),
                )
                .unwrap();
                let found = found();
                ctx2.1.push(found);
                "?".to_string()
            }
        }
    }

    struct Condition<const B: bool, A1, A2>(A1, A2);

    impl<const B: bool, A1, A2> WhereItem<MySql, MockMySql>
        for Condition<B, A1, A2>
    where
        MockMySql: Accept<A1, MySql>,
        MockMySql: Accept<A2, MySql>,
    {
        fn where_item(
            self,
            ctx: &mut <MockMySql as Query<MySql>>::Context1,
        ) -> impl FnOnce(
            &mut <MockMySql as Query<MySql>>::Context2,
        ) -> String {
            let ctx1 = unsafe { &mut *(ctx as *mut _) };
            let s1 = <MockMySql as Accept<A1, MySql>>::accept(
                self.0, ctx1,
            );
            let ctx2 = unsafe { &mut *(ctx as *mut _) };
            let s2 = <MockMySql as Accept<A2, MySql>>::accept(
                self.1, ctx2,
            );

            |ctx2| {
                if B {
                    format!("{} AND {}", s1(ctx2), s2(ctx2))
                } else {
                    format!("{} AND {}", s2(ctx2), s1(ctx2))
                }
            }
        }
    }

    #[test]
    fn test() {
        let mut ctx = Default::default();
        let ctx_mut = unsafe { &mut *((&mut ctx) as *mut _) };
        let part1 = Condition::<true, _, _>(
            || 3,
            || "hello".to_string(),
        )
        .where_item(ctx_mut);
        let ctx_mut2 = unsafe { &mut *((&mut ctx) as *mut _) };
        let part2 = Condition::<false, _, _>(
            || 3,
            || "hello".to_string(),
        )
        .where_item(ctx_mut2);

        let mut ctx2: <MockMySql as Query<MySql>>::Context2 =
            (ctx, Default::default());

        let res_str1 = part1(&mut ctx2);
        let _ = part2(&mut ctx2);

        let res_val = ctx2
            .1
            .into_iter()
            .map(|e| e.debug())
            .collect::<Vec<String>>();

        assert_eq!(res_str1, "? AND ?");

        assert_eq!(
            res_val,
            vec![
                "i32: 3".to_string(),
                "String: \"hello\"".to_string(),
                "String: \"hello\"".to_string(),
                "i32: 3".to_string(),
            ]
        );
    }

    struct WhereClause<S, Q: Query<S>> {
        columns: Vec<Q::SqlPart>,
        args: Q::Context1,
        _pd: PhantomData<S>,
    }

    impl<S, Q: Query<S>> Default for WhereClause<S, Q> {
        fn default() -> Self {
            Self {
                columns: Default::default(),
                args: Default::default(),
                _pd: PhantomData,
            }
        }
    }

    impl<S> WhereClause<S, MockMySql>
    where
        S: 'static,
        MockMySql: Query<
            S,
            Context2: 'static,
            Context1: 'static,
            SqlPart = Box<
                dyn FnOnce(
                    &mut <MockMySql as Query<S>>::Context2,
                ) -> String,
            >,
        >,
    {
        fn item<T>(&mut self, item: T)
        where
            T: WhereItem<S, MockMySql> + 'static,
            WhereItemToSqlPart<T>: ToSqlPart<MockMySql, S>,
        {
            let part = WhereItemToSqlPart(item)
                .to_sql_part(&mut self.args);
            self.columns.push(part);
        }
    }

    struct WhereEx<T>(T);

    impl<S, Q, T> WhereItem<S, Q> for WhereEx<T>
    where
        Q: Query<S>,
        Q: Accept<T, S>,
    {
        fn where_item(
            self,
            ctx: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2) -> String {
            let ctx1 = unsafe { &mut *(ctx as *mut _) };
            let s1 = <Q as Accept<T, S>>::accept(self.0, ctx1);

            move |ctx2| s1(ctx2)
        }
    }

    #[test]
    fn test_where_clause() {
        let mut where_clause = WhereClause::default();

        where_clause.item(WhereEx(|| 3));
        where_clause.item(WhereEx(|| "hello".to_string()));

        let mut ctx2: <MockMySql as Query<MySql>>::Context2 =
            (where_clause.args, Default::default());

        let res = where_clause.columns.into_iter().map(|e| {
            MockMySql::build_sql_part_back(&mut ctx2, e)
        });

        let res = res.collect::<Vec<String>>();

        assert_eq!(res, vec!["?", "?"]);
    }
}
