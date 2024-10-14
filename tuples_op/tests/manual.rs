// #![allow(unused)]
// use std::{fmt::Display, ops::Not};
//
// use tuples_op::{
//     behaviors::{Context, Mut, MutOp, PhantomOp},
//     invariant,
// };
//
// pub trait Mutate {
//     fn my_behavior(&mut self, ctx: &mut String);
// }
//
// impl Mutate for i32 {
//     fn my_behavior(&mut self, ctx: &mut String) {
//         ctx.push_str("i32");
//         *self += 1;
//     }
// }
//
// impl Mutate for bool {
//     fn my_behavior(&mut self, ctx: &mut String) {
//         ctx.push_str("bool");
//         *self = !*self;
//     }
// }
//
// impl Mutate for f32 {
//     fn my_behavior(&mut self, ctx: &mut String) {
//         ctx.push_str("f32");
//         *self += 0.1;
//     }
// }
//
// // invariant! {
// //     trait MutateExt: Mutate {
// //         fn part_mut_ctx_mut<E: Mutate>(part: &mut E, ctx: &mut String) {
// //             if INDEX != 0 {
// //                 ctx.push_str(", ");
// //                 part.my_behavior(ctx);
// //             }else {
// //                 part.my_behavior(ctx);
// //             }
// //         }
// //         fn my_behavior(&mut self) -> String {
// //             let mut output = String::new();
// //             self.tuple_mut_ctx_mut(&mut output);
// //             output
// //         }
// //     }
// // }
//
// struct Invariant;
//
// trait MutateTuple {
//     fn my_behavior(&mut self) -> String;
// }
//
// impl Context<'m', 2> for Invariant {
//     type Context = String;
// }
//
// impl<E: Mutate> Mut<E> for Invariant {
//     fn each<const INDEX: usize, const LEN: usize>(
//         part: &mut E,
//         ctx: &mut Self::Context,
//     ) {
//         if INDEX != 0 {
//             ctx.push_str(", ");
//             part.my_behavior(ctx);
//         } else {
//             part.my_behavior(ctx);
//         }
//     }
// }
//
// impl<T> MutateTuple for T
// where
//     T: MutOp<Invariant>,
// {
//     fn my_behavior(&mut self) -> String {
//         let mut output = String::new();
//         self.tuple_mut_ctx_mut(&mut output);
//         output
//     }
// }
//
// #[test]
// fn with_lifetimes() {
//     let mut tuple = (32, true, 3.14);
//
//     let output = tuple.my_behavior();
//
//     assert_eq!(output, "i32, bool, f32");
//     assert_eq!(tuple, (33, false, 3.24));
// }
