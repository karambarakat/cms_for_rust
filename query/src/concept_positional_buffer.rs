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
            let found =
                take(ctx2.0.get_mut(len - 1).expect("overflow"))
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
    let part1 =
        Condition::<true, _, _>(|| 3, || "hello".to_string())
            .where_item(ctx_mut);
    let ctx_mut2 = unsafe { &mut *((&mut ctx) as *mut _) };
    let part2 =
        Condition::<false, _, _>(|| 3, || "hello".to_string())
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
        let part =
            WhereItemToSqlPart(item).to_sql_part(&mut self.args);
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

    let res = where_clause
        .columns
        .into_iter()
        .map(|e| MockMySql::build_sql_part_back(&mut ctx2, e));

    let res = res.collect::<Vec<String>>();

    assert_eq!(res, vec!["?", "?"]);
}
