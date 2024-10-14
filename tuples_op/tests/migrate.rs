#![allow(unused)]

use std::{marker::PhantomData, vec::IntoIter};

use tuples_op::behaviors::{
    ContextCM, PartConsumeCtxMut, TupleConsumeCtxMut,
};

trait Type {
    fn type_fn() -> String;
}

impl Type for i32 {
    fn type_fn() -> String {
        "I32".to_string()
    }
}

impl Type for String {
    fn type_fn() -> String {
        "STRING".to_string()
    }
}

trait ConstraintTuple<S, Q: Query<S>, Ty: Type> {
    fn main_fn(self, col: &mut Vec<String>) -> Q::SqlPart;
}

struct Inv<S, Q, Ty>(PhantomData<(S, Q, Ty)>);

impl<S, Q, T> ContextCM for Inv<S, Q, T> {
    type Context = String;
}

impl<E, Ty, S, Q: Query<S>> PartConsumeCtxMut<E> for Inv<S, Q, Ty>
where
    E: ConstraintPart<S, Q, Ty>,
{
    type Output = ();
    fn part<const I: usize, const N: usize>(
        part: E,
        ctx: &mut String,
    ) -> Q::SqlPart {
        // Q::handle(|ctx| {
        let output = part.part_fn(&mut ctx.1);
        match output {
            Some(part) => {
                ctx.0.push(' ');
                ctx.0.push_str(&part);
            }
            None => {}
        }
    }
}

impl<S, Q: Query<S>, T, Tyy: Type> ConstraintTuple<S, Q, Tyy> for T
where
    T: TupleConsumeCtxMut<Inv<S, Q, Tyy>>,
{
    fn main_fn(self, col: &mut Vec<String>) -> Q::SqlPart {
        self.tuple_consume_ctx_mut(col);
    }
}

trait Infer<S, Q, T> {}
impl<T> Infer<(), (), T> for PhantomData<T> {}

fn column<Tyy: Type, S, Q: Query<S>>(
    name: &str,
    infer: impl Infer<S, Q, Tyy>,
    constraints: impl ConstraintTuple<S, Q, Tyy>,
) -> String {
    let mut col = String::from(name);
    col.push(' ');
    col.push_str(&Tyy::type_fn());
    let mut ctx = (col, Vec::new());
    let hi: Q::SqlPart = constraints.main_fn(&mut ctx);
    ctx.0
}

#[test]
fn migrate_1() {
    let col = column("id", PhantomData::<i32>, ());

    assert_eq!(col, "id I32");

    // let col = column("id", PhantomData::<String>, (not_null(),));
    //
    // assert_eq!(col, "id STRING NOT NULL");
    //
    // let col =
    //     column("id", PhantomData::<i32>, (not_null(), default(0)));
    //
    // assert_eq!(col, "id I32 NOT NULL DEFAULT $1");
}

struct NotNull;
struct Default<T>(T);
struct CheckIfNull;

trait Query<S> {
    type SqlPart;
    type StrContext;
    type Context;
    fn handle(
        t: impl FnOnce(&mut Self::StrContext, &mut String),
    ) -> Self::SqlPart;
}

trait ConstraintPart<S, Q: Query<S>, Ty> {
    fn part_fn(self, ctx: &mut Q::StrContext) -> Option<String>;
}

impl<S, Q: Query<S>, T> ConstraintPart<S, Q, T> for NotNull {
    fn part_fn(self, ctx: &mut Q::StrContext) -> Option<String> {
        Some("NOT NULL".to_string())
    }
    // fn to_sql_string
}

trait Accept<S, T>: Query<S> {
    fn to_sql_string(
        value: T,
        ctx: &mut <Self as Query<S>>::StrContext,
    ) -> String;
}

impl<T: ToString, S, Q: Query<S>> Accept<S, T> for Q {
    fn to_sql_string(
        value: T,
        ctx: &mut <Self as Query<S>>::StrContext,
    ) -> String {
        format!("${}", {
            let val = value.to_string();
            ctx.push(val);
            ctx.len()
        })
    }
}

impl<S, Q, T> ConstraintPart<S, Q, T> for Default<T>
where
    Q: Accept<S, T>,
{
    fn part_fn(self, ctx: &mut Q::StrContext) -> Option<String> {
        Some(format!("DEFAULT {}", Q::to_sql_string(self.0, ctx)))
    }
}

impl<S, Q: Query<S>, T> ConstraintPart<S, Q, T> for CheckIfNull {
    fn part_fn(self, ctx: &mut Q::StrContext) -> Option<String> {
        None
    }
}

fn not_null() -> NotNull {
    NotNull
}

fn default<Ty>(value: Ty) -> Default<Ty> {
    Default(value)
}

fn check_if_null() -> CheckIfNull {
    CheckIfNull
}
