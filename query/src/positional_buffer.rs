use sealing::{SealedS, SealedT};
use sqlx::{
    database::HasArguments, Arguments, Database, Encode, Type,
};

use crate::{
    ident_safety::PanicOnUnsafe, sql_part::{
        AcceptToSqlPart, ColumnToSqlPart, ConstraintToSqlPart,
        ToSqlPart, WhereItemToSqlPart,
    }, Accept, Constraint, Query, SchemaColumn, WhereItem
};

mod sealing {
    pub trait SealedT {}
    pub struct SealedS;
}

pub trait BindLater<S: Database>: SealedT {
    fn bind_from_heap(
        self: Box<Self>,
        buffer: &mut <S as HasArguments<'static>>::Arguments,
        _private_to_call: SealedS,
    );
}

pub struct Container<T>(T);

impl<T> SealedT for Container<T> {}

impl<S, T> BindLater<S> for Container<T>
where
    T: for<'q> Encode<'q, S> + Type<S> + Send + 'static,
    S: Database,
{
    fn bind_from_heap(
        self: Box<Self>,
        buffer: &mut <S as HasArguments<'static>>::Arguments,
        _private_to_call: SealedS,
    ) {
        let Container(val) = *self;
        buffer.add(val);
    }
}

pub struct PositionalStaticBuffer;
impl<S> Query<S> for PositionalStaticBuffer
where
    S: Database,
{
    type SqlPart =
        Box<dyn FnOnce(&mut Self::Context2) -> String>;
    type Context1 = Vec<Option<Box<dyn BindLater<S>>>>;
    type Context2 = (
        Vec<Option<Box<dyn BindLater<S>>>>,
        Vec<Box<dyn BindLater<S>>>,
    );
    type Output = <S as HasArguments<'static>>::Arguments;

    fn build_sql_part_back(
        ctx: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String {
        from(ctx)
    }

    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        let mut ctx2 = (ctx1, Default::default());
        let str = f(&mut ctx2);
        let bind = ctx2.1;
        let mut args = Default::default();
        for each in bind {
            each.bind_from_heap(&mut args, SealedS);
        }
        (str, args)
    }
}

impl<A, T, S> Accept<A, S> for PositionalStaticBuffer
where
    A: FnOnce() -> T + 'static,
    T: for<'q> Encode<'q, S> + Type<S> + Send + 'static,
    S: Database,
{
    fn accept(
        this: A,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        let container = Box::new(Container(this()));
        let index = ctx1.len();
        ctx1.push(Some(container));
        move |ctx2| {
            let taken = ctx2
                .0
                .get_mut(index)
                .expect("out of index")
                .take()
                .expect("should only take once");

            ctx2.1.push(taken);
            "?".to_string()
        }
    }
}

impl<T, S> ToSqlPart<PositionalStaticBuffer, S>
    for WhereItemToSqlPart<T>
where
    S: Database,
    T: WhereItem<S, PositionalStaticBuffer, PanicOnUnsafe> + 'static,
{
    fn to_sql_part(
        self,
        ctx: &mut <PositionalStaticBuffer as Query<S>>::Context1,
    ) -> <PositionalStaticBuffer as Query<S>>::SqlPart {
        let ctx2 = unsafe { &mut *(ctx as *mut _) };
        let item = self.0.where_item(ctx2);
        Box::new(move |ctx2| item(ctx2))
    }
}

impl<S, T> ToSqlPart<PositionalStaticBuffer, S> for AcceptToSqlPart<T>
where
    PositionalStaticBuffer: Accept<T, S> + 'static,
    PositionalStaticBuffer: Query<
        S,
        SqlPart = Box<
            dyn FnOnce(
                &mut <PositionalStaticBuffer as Query<S>>::Context2,
            ) -> String,
        >,
    >,
    S: Database,
{
    fn to_sql_part(
        self,
        ctx: &mut <PositionalStaticBuffer as Query<S>>::Context1,
    ) -> <PositionalStaticBuffer as Query<S>>::SqlPart {
        let first = self.0;
        let first = PositionalStaticBuffer::accept(first, ctx);
        Box::new(move |ctx2| first(ctx2))
    }
}

impl<S, T> ToSqlPart<PositionalStaticBuffer, S>
    for ConstraintToSqlPart<T>
where
    S: Database,
    T: Constraint<S, PositionalStaticBuffer> + 'static,
{
    fn to_sql_part(
        self,
        ctx: &mut <PositionalStaticBuffer as Query<S>>::Context1,
    ) -> <PositionalStaticBuffer as Query<S>>::SqlPart {
        let ctx2 = unsafe { &mut *(ctx as *mut _) };
        let item = self.0.constraint(ctx2);
        Box::new(move |ctx2| item(ctx2))
    }
}

impl<S, T> ToSqlPart<PositionalStaticBuffer, S>
    for ColumnToSqlPart<T>
where
    S: Database,
    T: SchemaColumn<S, PositionalStaticBuffer> + 'static,
{
    fn to_sql_part(
        self,
        ctx: &mut <PositionalStaticBuffer as Query<S>>::Context1,
    ) -> <PositionalStaticBuffer as Query<S>>::SqlPart {
        let ctx2 = unsafe { &mut *(ctx as *mut _) };
        let item = self.0.column(ctx2);
        Box::new(move |ctx2| item(ctx2))
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_positional() {
    use std::marker::PhantomData;

    use sqlx::Sqlite;

    use crate::debug_query::DebugQueries;
    use crate::execute_no_cache::ExecuteNoCache;
    use crate::{
        expressions::exports::col, from_row::sqlx_from_row,
        prelude::stmt, InitStatement,
    };

    let pool =
        sqlx::Pool::<sqlx::Sqlite>::connect("sqlite::memory:")
            .await
            .unwrap();

    let mut st =
        stmt::SelectSt::<_, PositionalStaticBuffer>::init(
            "Todos",
        );

    sqlx::query("
CREATE TABLE Todos (
    id INTEGER PRIMARY KEY, 
    title TEXT
);

INSERT INTO Todos (title) VALUES ('hello'), ('hello'), ('hello'), ('hello');
")
            .execute(&pool)
            .await
            .unwrap();

    st.select(col("id"));
    // the value here will bound second, PositionalBuffer knows this apprears second in the query
    // PositionalBuffer will save this value in the heap, remember its position, and when building the query, it will bind it with the correct order
    st.limit(|| 2);
    // the value here will bound first, PositionalBuffer knows this apprears first in the query
    st.where_(col("title").eq(|| "hello".to_string()));

    let debug = st.debug(PhantomData::<Sqlite>);

    assert_eq!(
        debug.as_str(),
        "SELECT id FROM Todos WHERE title = ? LIMIT ?;"
    );

    let row = debug
        .fetch_all(&pool, sqlx_from_row::<(i32,), Sqlite>())
        .await
        .unwrap();

    // this is proof that limit is 2, and title is "hello"
    assert_eq!(row, vec![(1,), (2,)]);
}
// }
//
// pub trait DebugAny
// where
//     Self: std::fmt::Debug,
// {
//     fn ty(&self) -> &'static str;
//     fn debug(&self) -> String {
//         format!("{}: {:?}", self.ty(), self)
//     }
// }
//
// #[derive(Debug, Default)]
// pub struct MockMySql;
// impl<T> ToSqlPart<MockMySql, MySql> for WhereItemToSqlPart<T>
// where
//     T: WhereItem<MySql, MockMySql> + 'static,
// {
//     fn to_sql_part(
//         self,
//         ctx: &mut <MockMySql as Query<MySql>>::Context1,
//     ) -> <MockMySql as Query<MySql>>::SqlPart {
//         let ctx2 = unsafe { &mut *(ctx as *mut _) };
//         let item = self.0.where_item(ctx2);
//         Box::new(move |ctx2| item(ctx2))
//     }
// }
// impl Query<MySql> for MockMySql {
//     type SqlPart =
//         Box<dyn FnOnce(&mut Self::Context2) -> String>;
//
//     type Context1 =
//         Vec<Option<Box<dyn FnOnce() -> Box<dyn DebugAny>>>>;
//
//     type Context2 = (
//         Vec<Option<Box<dyn FnOnce() -> Box<dyn DebugAny>>>>,
//         Vec<Box<dyn DebugAny>>,
//     );
//
//     fn build_sql_part_back(
//         ctx: &mut Self::Context2,
//         from: Self::SqlPart,
//     ) -> String {
//         from(ctx)
//     }
//
//     type Output = ();
//     fn build_query(
//         _: Self::Context1,
//         _: impl FnOnce(&mut Self::Context2) -> String,
//     ) -> (String, Self::Output) {
//         panic!("just an example")
//     }
// }
//
// macro_rules! debug_any {
//     ($($ident:ident), *) => {
//         $(impl DebugAny for $ident
//         where $ident: std::fmt::Debug
//         {
//             fn ty(&self) -> &'static str {
//                 stringify!($ident)
//             }
//         })*
//     };
// }
//
// debug_any!(String, i8, i16, i32, u8, u16, u32, u64, bool);
//
// impl<A, T> Accept<A, MySql> for MockMySql
// where
//     A: FnOnce() -> T + 'static,
//     T: DebugAny + 'static,
// {
//     fn accept(
//         this: A,
//         ctx1: &mut Self::Context1,
//     ) -> impl FnOnce(&mut Self::Context2) -> String + 'static
//     {
//         ctx1.push(Some(Box::new(|| Box::new(this()))));
//         let len = ctx1.len();
//
//         move |ctx2| {
//             let found =
//                 take(ctx2.0.get_mut(len - 1).expect("overflow"))
//                     .unwrap();
//             let found = found();
//             ctx2.1.push(found);
//             "?".to_string()
//         }
//     }
// }
//
// struct Condition<const B: bool, A1, A2>(A1, A2);
//
// impl<const B: bool, A1, A2> WhereItem<MySql, MockMySql>
//     for Condition<B, A1, A2>
// where
//     MockMySql: Accept<A1, MySql>,
//     MockMySql: Accept<A2, MySql>,
// {
//     fn where_item(
//         self,
//         ctx: &mut <MockMySql as Query<MySql>>::Context1,
//     ) -> impl FnOnce(
//         &mut <MockMySql as Query<MySql>>::Context2,
//     ) -> String {
//         let ctx1 = unsafe { &mut *(ctx as *mut _) };
//         let s1 = <MockMySql as Accept<A1, MySql>>::accept(
//             self.0, ctx1,
//         );
//         let ctx2 = unsafe { &mut *(ctx as *mut _) };
//         let s2 = <MockMySql as Accept<A2, MySql>>::accept(
//             self.1, ctx2,
//         );
//
//         |ctx2| {
//             if B {
//                 format!("{} AND {}", s1(ctx2), s2(ctx2))
//             } else {
//                 format!("{} AND {}", s2(ctx2), s1(ctx2))
//             }
//         }
//     }
// }
//
// #[test]
// fn test() {
//     let mut ctx = Default::default();
//     let ctx_mut = unsafe { &mut *((&mut ctx) as *mut _) };
//     let part1 =
//         Condition::<true, _, _>(|| 3, || "hello".to_string())
//             .where_item(ctx_mut);
//     let ctx_mut2 = unsafe { &mut *((&mut ctx) as *mut _) };
//     let part2 =
//         Condition::<false, _, _>(|| 3, || "hello".to_string())
//             .where_item(ctx_mut2);
//
//     let mut ctx2: <MockMySql as Query<MySql>>::Context2 =
//         (ctx, Default::default());
//
//     let res_str1 = part1(&mut ctx2);
//     let _ = part2(&mut ctx2);
//
//     let res_val = ctx2
//         .1
//         .into_iter()
//         .map(|e| e.debug())
//         .collect::<Vec<String>>();
//
//     assert_eq!(res_str1, "? AND ?");
//
//     assert_eq!(
//         res_val,
//         vec![
//             "i32: 3".to_string(),
//             "String: \"hello\"".to_string(),
//             "String: \"hello\"".to_string(),
//             "i32: 3".to_string(),
//         ]
//     );
// }
//
// struct WhereClause<S, Q: Query<S>> {
//     columns: Vec<Q::SqlPart>,
//     args: Q::Context1,
//     _pd: PhantomData<S>,
// }
//
// impl<S, Q: Query<S>> Default for WhereClause<S, Q> {
//     fn default() -> Self {
//         Self {
//             columns: Default::default(),
//             args: Default::default(),
//             _pd: PhantomData,
//         }
//     }
// }
//
// impl<S> WhereClause<S, MockMySql>
// where
//     S: 'static,
//     MockMySql: Query<
//         S,
//         Context2: 'static,
//         Context1: 'static,
//         SqlPart = Box<
//             dyn FnOnce(
//                 &mut <MockMySql as Query<S>>::Context2,
//             ) -> String,
//         >,
//     >,
// {
//     fn item<T>(&mut self, item: T)
//     where
//         T: WhereItem<S, MockMySql> + 'static,
//         WhereItemToSqlPart<T>: ToSqlPart<MockMySql, S>,
//     {
//         let part =
//             WhereItemToSqlPart(item).to_sql_part(&mut self.args);
//         self.columns.push(part);
//     }
// }
//
// struct WhereEx<T>(T);
//
// impl<S, Q, T> WhereItem<S, Q> for WhereEx<T>
// where
//     Q: Query<S>,
//     Q: Accept<T, S>,
// {
//     fn where_item(
//         self,
//         ctx: &mut Q::Context1,
//     ) -> impl FnOnce(&mut Q::Context2) -> String {
//         let ctx1 = unsafe { &mut *(ctx as *mut _) };
//         let s1 = <Q as Accept<T, S>>::accept(self.0, ctx1);
//
//         move |ctx2| s1(ctx2)
//     }
// }
//
// #[test]
// fn test_where_clause() {
//     let mut where_clause = WhereClause::default();
//
//     where_clause.item(WhereEx(|| 3));
//     where_clause.item(WhereEx(|| "hello".to_string()));
//
//     let mut ctx2: <MockMySql as Query<MySql>>::Context2 =
//         (where_clause.args, Default::default());
//
//     let res = where_clause
//         .columns
//         .into_iter()
//         .map(|e| MockMySql::build_sql_part_back(&mut ctx2, e));
//
//     let res = res.collect::<Vec<String>>();
//
//     assert_eq!(res, vec!["?", "?"]);
// }
