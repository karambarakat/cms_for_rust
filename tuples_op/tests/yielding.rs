// use std::{marker::PhantomData, ops::Deref};
//
// use tuples_op::behaviors::{Context, Phantom, Yield, YieldOp, YieldRefCtxOp, YieldRefCtx};
//
// pub trait YieldExample<'c> {
//     type Returning;
//     fn my_behavior(ctx: &'c String) -> Self::Returning;
// }
//
// impl<'c> YieldExample<'c> for i32 {
//     type Returning = Result<Self, ()>;
//     fn my_behavior(ctx: &String) -> Self::Returning {
//         ctx.len().try_into().map_err(|_| ())
//     }
// }
//
// impl<'c> YieldExample<'c> for bool {
//     type Returning = Self;
//     fn my_behavior(ctx: &String) -> Self::Returning {
//         ctx.is_empty()
//     }
// }
//
// impl<'c, 'c2> YieldExample<'c> for &'c2 str
//     where 'c: 'c2
// {
//     type Returning = &'c str;
//     fn my_behavior(ctx: &'c String) -> Self::Returning {
// ctx.as_str()
//     }}
//
// struct Invariant<'c> (PhantomData<(&'c (),)>);
// impl<'c> Context<'r', 30> for Invariant<'c> {
//     type Context = String;
// }
// impl<'c, 'c2, E> YieldRefCtx<'c, E> for Invariant<'c2> 
//     where for<'a> E: YieldExample<'a>
// {
//     type Yielding = <E as YieldExample<'c>>::Returning;
//     fn each<const INDEX: usize, const LEN: usize>(
//         ctx: &'c  Self::Context,
//     ) -> Self::Yielding {
//         E::my_behavior(ctx)
//     }
// }
//
// trait TargetExample<'c> {
//     type Output;
//     fn tuple(ctx: &'c String) -> Self::Output;}
//
// #[test]
// fn client() {
//     let str = String::from("Hello, World!");
//
//     // let output = <(&str, &str, &str) as TargetExample>::tuple(&str);
//
//
//
//     
//
// }
//
//
//
