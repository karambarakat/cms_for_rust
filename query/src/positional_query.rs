use std::ops::IndexMut;
use std::{marker::PhantomData, mem::take};

use sqlx::{
    database::HasArguments, prelude::Type, Arguments, Database,
    Encode,
};

use crate::{
    ident_safety::PanicOnUnsafe, Accept, BindItem, Query,
    QueryHandlers,
};
use crate::{AcceptColIdent, AcceptTableIdent, IdentSafety};

pub struct PositionalQuery<'q, S>(PhantomData<(&'q (), S)>);

pub trait Stored<S> {
    fn bind(
        self: Box<Self>,
        ctx: &mut <S as HasArguments<'static>>::Arguments,
    ) where
        S: Database;
}

impl<S, T> Stored<S> for T
where
    S: Database,
    T: Type<S> + for<'q> Encode<'q, S> + 'static + Send,
{
    fn bind(
        self: Box<Self>,
        ctx: &mut <S as HasArguments<'static>>::Arguments,
    ) where
        S: Database,
    {
        ctx.add(*self);
    }
}

pub struct NoOpIdentSafety;
impl IdentSafety for NoOpIdentSafety {
    type Table = String;

    type Column = String;

    fn check_other<T: AsRef<str>>(any_: T) {}

    fn init<T: AsRef<str>>(on_table: Option<&T>) -> Self {
        NoOpIdentSafety
    }
}

impl AcceptColIdent<&'static str> for NoOpIdentSafety {
    fn into_col(this: &'static str) -> Self::Column {
        this.to_string()
    }
}

impl AcceptTableIdent<&'static str> for NoOpIdentSafety {
    fn into_table(this: &'static str) -> Self::Table {
        this.to_string()
    }
}

impl<S> Query for PositionalQuery<'static, S>
where
    S: Database,
{
    type IdentSafety = NoOpIdentSafety;

    type SqlPart =
        Box<dyn FnOnce(&mut Self::Context2) -> String>;

    type Context1 = Vec<Option<Box<dyn Stored<S>>>>;

    type Context2 = (
        Vec<Option<Box<dyn Stored<S>>>>,
        <S as HasArguments<'static>>::Arguments,
    );

    fn build_sql_part_back(
        ctx: &mut Self::Context2,
        mut from: Self::SqlPart,
    ) -> String {
        from(ctx)
    }

    type Output = <S as HasArguments<'static>>::Arguments;

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        let mut ctx2 = (ctx1, Default::default());
        let str = f(&mut ctx2);
        let output = ctx2.1;
        return (str, output);
    }
}

impl<S> QueryHandlers<S> for PositionalQuery<'static, S>
where
    Self: Query<
        SqlPart = Box<dyn FnOnce(&mut Self::Context2) -> String>,
        Context1 = Vec<Option<Box<dyn Stored<S>>>>,
        Context2 = (
            Vec<Option<Box<dyn Stored<S>>>>,
            <S as HasArguments<'static>>::Arguments,
        ),
    >,
    S: Database,
{
    fn handle_bind_item<T, I>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: BindItem<S, Self, I> + 'static,
    {
        Box::new(move |ctx| {
            let b = t.bind_item(&mut ctx.0);
            b(ctx)
        })
    }

    fn handle_accept<T>(
        t: T,
        ctx: &mut Self::Context1,
    ) -> Self::SqlPart
    where
        T: Send + 'static,
        Self: Accept<T, S>,
    {
        Box::new(move |ctx| {
            let bring_back = Self::accept(t, &mut ctx.0);
            bring_back(ctx)
        })
    }
}

impl<S, T> Accept<T, S> for PositionalQuery<'static, S>
where
    S: Database,
    T: Type<S> + for<'q> Encode<'q, S> + 'static + Send,
{
    fn accept(
        this: T,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        ctx1.push(Some(Box::new(this)));
        let len = ctx1.len();

        move |ctx2| {
            let bring_back = ctx2
                .0
                .get_mut(len - 1)
                .map(|e| e.take())
                .flatten()
                .expect("should be bound and taken only once");

            bring_back.bind(&mut ctx2.1);

            "?".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use sqlx::{
        encode::IsNull,
        sqlite::{
            SqliteArgumentValue, SqliteArguments,
            SqliteTypeInfo, SqliteValue,
        },
        Encode, Sqlite, Type,
    };

    use crate::{
        ident_safety::{define_schema, PanicOnUnsafe},
        positional_query::NoOpIdentSafety,
        prelude::col,
        select_st::SelectSt,
    };

    use super::PositionalQuery;

    struct StringAlias(String);

    static BIND_ORDER: Mutex<Vec<String>> =
        Mutex::new(Vec::new());

    impl<'q> Encode<'q, Sqlite> for StringAlias {
        fn encode_by_ref(
            &self,
            buf: &mut Vec<SqliteArgumentValue<'q>>,
        ) -> IsNull {
            BIND_ORDER.lock().unwrap().push(self.0.clone());

            <String as Encode<'q, Sqlite>>::encode_by_ref(
                &self.0, buf,
            )
        }
        fn encode(
            self,
            buf: &mut <Sqlite as sqlx::database::HasArguments<
                'q,
            >>::ArgumentBuffer,
        ) -> IsNull
        where
            Self: Sized,
        {
            BIND_ORDER.lock().unwrap().push(self.0.clone());

            <String as Encode<'q, Sqlite>>::encode(self.0, buf)
        }
    }

    impl Type<Sqlite> for StringAlias {
        fn type_info() -> SqliteTypeInfo {
            todo!()
        }
    }

    #[test]
    fn positional_query_figure_out_order() {
        let mut st = SelectSt::<
            Sqlite,
            PositionalQuery<'static, Sqlite>,
            NoOpIdentSafety,
        >::init("Todo");

        define_schema(&[("Todo", &["id", "title"])]);

        st.select(col("*"));
        st.where_(col("id").eq(StringAlias("1".to_string())));
        st.offset(StringAlias("2".to_string()));

        let (str, arg) = st.build();

        drop(arg);

        assert_eq!(
            str,
            "SELECT * FROM Todo WHERE id = ? OFFSET ?;"
        );

        let bind_order = BIND_ORDER.lock().unwrap().clone();

        assert_eq!(
            bind_order,
            vec!["1".to_string(), "2".to_string()]
        );

        BIND_ORDER.lock().unwrap().drain(..);

        // even when we call offset before where,
        // PositionalQuery should know to reorder them

        let mut st = SelectSt::<
            Sqlite,
            PositionalQuery<'static, Sqlite>,
            NoOpIdentSafety,
        >::init("Todo");

        define_schema(&[("Todo", &["id", "title"])]);

        st.select(col("*"));
        st.offset(StringAlias("2".to_string()));
        st.where_(col("id").eq(StringAlias("1".to_string())));

        let (str, arg) = st.build();

        drop(arg);

        assert_eq!(
            str,
            "SELECT * FROM Todo WHERE id = ? OFFSET ?;"
        );

        let bind_order = BIND_ORDER.lock().unwrap().clone();

        assert_eq!(
            bind_order,
            vec!["1".to_string(), "2".to_string()]
        );
    }
}
