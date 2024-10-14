// use crate::behaviors::*;
// // mod phantom_op {
// //     use crate::behaviors::*;
// //
// //     impl<I> PhantomOp<I> for ()
// //     where
// //         I: Context<0>,
// //     {
// //         fn tuple_phantom_ctx_mut(_: &mut I::Context) {}
// //     }
// //
// //     impl<I, T0> PhantomOp<I> for (T0,)
// //     where
// //         I: Phantom<T0>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 1>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1> PhantomOp<I> for (T0, T1)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 2>(ctx);
// //             <I as Phantom<T1>>::each::<1, 2>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2> PhantomOp<I> for (T0, T1, T2)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 3>(ctx);
// //             <I as Phantom<T1>>::each::<1, 3>(ctx);
// //             <I as Phantom<T2>>::each::<2, 3>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3> PhantomOp<I> for (T0, T1, T2, T3)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 4>(ctx);
// //             <I as Phantom<T1>>::each::<1, 4>(ctx);
// //             <I as Phantom<T2>>::each::<2, 4>(ctx);
// //             <I as Phantom<T3>>::each::<3, 4>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4> PhantomOp<I> for (T0, T1, T2, T3, T4)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //         I: Phantom<T4>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 5>(ctx);
// //             <I as Phantom<T1>>::each::<1, 5>(ctx);
// //             <I as Phantom<T2>>::each::<2, 5>(ctx);
// //             <I as Phantom<T3>>::each::<3, 5>(ctx);
// //             <I as Phantom<T4>>::each::<4, 5>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5> PhantomOp<I>
// //         for (T0, T1, T2, T3, T4, T5)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //         I: Phantom<T4>,
// //         I: Phantom<T5>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 6>(ctx);
// //             <I as Phantom<T1>>::each::<1, 6>(ctx);
// //             <I as Phantom<T2>>::each::<2, 6>(ctx);
// //             <I as Phantom<T3>>::each::<3, 6>(ctx);
// //             <I as Phantom<T4>>::each::<4, 6>(ctx);
// //             <I as Phantom<T5>>::each::<5, 6>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6> PhantomOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //         I: Phantom<T4>,
// //         I: Phantom<T5>,
// //         I: Phantom<T6>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 7>(ctx);
// //             <I as Phantom<T1>>::each::<1, 7>(ctx);
// //             <I as Phantom<T2>>::each::<2, 7>(ctx);
// //             <I as Phantom<T3>>::each::<3, 7>(ctx);
// //             <I as Phantom<T4>>::each::<4, 7>(ctx);
// //             <I as Phantom<T5>>::each::<5, 7>(ctx);
// //             <I as Phantom<T6>>::each::<6, 7>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7> PhantomOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //         I: Phantom<T4>,
// //         I: Phantom<T5>,
// //         I: Phantom<T6>,
// //         I: Phantom<T7>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 8>(ctx);
// //             <I as Phantom<T1>>::each::<1, 8>(ctx);
// //             <I as Phantom<T2>>::each::<2, 8>(ctx);
// //             <I as Phantom<T3>>::each::<3, 8>(ctx);
// //             <I as Phantom<T4>>::each::<4, 8>(ctx);
// //             <I as Phantom<T5>>::each::<5, 8>(ctx);
// //             <I as Phantom<T6>>::each::<6, 8>(ctx);
// //             <I as Phantom<T7>>::each::<7, 8>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7, T8> PhantomOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7, T8)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //         I: Phantom<T4>,
// //         I: Phantom<T5>,
// //         I: Phantom<T6>,
// //         I: Phantom<T7>,
// //         I: Phantom<T8>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 9>(ctx);
// //             <I as Phantom<T1>>::each::<1, 9>(ctx);
// //             <I as Phantom<T2>>::each::<2, 9>(ctx);
// //             <I as Phantom<T3>>::each::<3, 9>(ctx);
// //             <I as Phantom<T4>>::each::<4, 9>(ctx);
// //             <I as Phantom<T5>>::each::<5, 9>(ctx);
// //             <I as Phantom<T6>>::each::<6, 9>(ctx);
// //             <I as Phantom<T7>>::each::<7, 9>(ctx);
// //             <I as Phantom<T8>>::each::<8, 9>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9> PhantomOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //         I: Phantom<T4>,
// //         I: Phantom<T5>,
// //         I: Phantom<T6>,
// //         I: Phantom<T7>,
// //         I: Phantom<T8>,
// //         I: Phantom<T9>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 10>(ctx);
// //             <I as Phantom<T1>>::each::<1, 10>(ctx);
// //             <I as Phantom<T2>>::each::<2, 10>(ctx);
// //             <I as Phantom<T3>>::each::<3, 10>(ctx);
// //             <I as Phantom<T4>>::each::<4, 10>(ctx);
// //             <I as Phantom<T5>>::each::<5, 10>(ctx);
// //             <I as Phantom<T6>>::each::<6, 10>(ctx);
// //             <I as Phantom<T7>>::each::<7, 10>(ctx);
// //             <I as Phantom<T8>>::each::<8, 10>(ctx);
// //             <I as Phantom<T9>>::each::<9, 10>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> PhantomOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //         I: Phantom<T4>,
// //         I: Phantom<T5>,
// //         I: Phantom<T6>,
// //         I: Phantom<T7>,
// //         I: Phantom<T8>,
// //         I: Phantom<T9>,
// //         I: Phantom<T10>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 11>(ctx);
// //             <I as Phantom<T1>>::each::<1, 11>(ctx);
// //             <I as Phantom<T2>>::each::<2, 11>(ctx);
// //             <I as Phantom<T3>>::each::<3, 11>(ctx);
// //             <I as Phantom<T4>>::each::<4, 11>(ctx);
// //             <I as Phantom<T5>>::each::<5, 11>(ctx);
// //             <I as Phantom<T6>>::each::<6, 11>(ctx);
// //             <I as Phantom<T7>>::each::<7, 11>(ctx);
// //             <I as Phantom<T8>>::each::<8, 11>(ctx);
// //             <I as Phantom<T9>>::each::<9, 11>(ctx);
// //             <I as Phantom<T10>>::each::<10, 11>(ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>
// //         PhantomOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)
// //     where
// //         I: Phantom<T0>,
// //         I: Phantom<T1>,
// //         I: Phantom<T2>,
// //         I: Phantom<T3>,
// //         I: Phantom<T4>,
// //         I: Phantom<T5>,
// //         I: Phantom<T6>,
// //         I: Phantom<T7>,
// //         I: Phantom<T8>,
// //         I: Phantom<T9>,
// //         I: Phantom<T10>,
// //         I: Phantom<T11>,
// //     {
// //         fn tuple_phantom_ctx_mut(ctx: &mut I::Context) {
// //             <I as Phantom<T0>>::each::<0, 12>(ctx);
// //             <I as Phantom<T1>>::each::<1, 12>(ctx);
// //             <I as Phantom<T2>>::each::<2, 12>(ctx);
// //             <I as Phantom<T3>>::each::<3, 12>(ctx);
// //             <I as Phantom<T4>>::each::<4, 12>(ctx);
// //             <I as Phantom<T5>>::each::<5, 12>(ctx);
// //             <I as Phantom<T6>>::each::<6, 12>(ctx);
// //             <I as Phantom<T7>>::each::<7, 12>(ctx);
// //             <I as Phantom<T8>>::each::<8, 12>(ctx);
// //             <I as Phantom<T9>>::each::<9, 12>(ctx);
// //             <I as Phantom<T10>>::each::<10, 12>(ctx);
// //             <I as Phantom<T11>>::each::<11, 12>(ctx);
// //         }
// //     }
// // }
// //
// // mod mut_op {
// // use crate::behaviors::*;
// //
// //     impl<I> MutOp<I> for ()
// //     where
// //         I: Context<2>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, _: &mut I::Context) {}
// //     }
// //
// //     impl<I, T0> MutOp<I> for (T0,)
// //     where
// //         I: Mut<T0>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 1>(&mut self.0, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1> MutOp<I> for (T0, T1)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 2>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 2>(&mut self.1, ctx);
// //         }
// //     }
// //
// impl<I, T0, T1, T2> MutOp<I> for (T0, T1, T2)
// where
//     I: Mut<T0>,
//     I: Mut<T1>,
//     I: Mut<T2>,
// {
//     fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
//         <I as Mut<T0>>::each::<0, 3>(&mut self.0, ctx);
//         <I as Mut<T1>>::each::<1, 3>(&mut self.1, ctx);
//         <I as Mut<T2>>::each::<2, 3>(&mut self.2, ctx);
//     }
// }
// impl<'c, I, T0, T1, T2> YieldRefCtxOp<'c, I> for (T0, T1, T2)
// where
//     for<'a> I: YieldRefCtx<'a, T0>,
//     for<'a> I: YieldRefCtx<'a, T1>,
//     for<'a> I: YieldRefCtx<'a, T2>,
// {
//     type Yielding = (
//         <I as YieldRefCtx<'c, T0>>::Yielding,
//         <I as YieldRefCtx<'c, T1>>::Yielding,
//         <I as YieldRefCtx<'c, T2>>::Yielding,
//     );
//
//     fn tuple_yield_ref_ctx(
//         ctx: &'c I::Context,
//     ) -> Self::Yielding {
//         (
//             <I as YieldRefCtx<T0>>::each::<0, 3>(ctx),
//             <I as YieldRefCtx<T1>>::each::<1, 3>(ctx),
//             <I as YieldRefCtx<T2>>::each::<2, 3>(ctx),
//         )
//     }
// }
//
// //
// //     impl<I, T0, T1, T2, T3> MutOp<I> for (T0, T1, T2, T3)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 4>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 4>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 4>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 4>(&mut self.3, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4> MutOp<I> for (T0, T1, T2, T3, T4)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //         I: Mut<T4>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 5>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 5>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 5>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 5>(&mut self.3, ctx);
// //             <I as Mut<T4>>::each::<4, 5>(&mut self.4, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5> MutOp<I> for (T0, T1, T2, T3, T4, T5)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //         I: Mut<T4>,
// //         I: Mut<T5>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 6>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 6>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 6>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 6>(&mut self.3, ctx);
// //             <I as Mut<T4>>::each::<4, 6>(&mut self.4, ctx);
// //             <I as Mut<T5>>::each::<5, 6>(&mut self.5, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6> MutOp<I> for (T0, T1, T2, T3, T4, T5, T6)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //         I: Mut<T4>,
// //         I: Mut<T5>,
// //         I: Mut<T6>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 7>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 7>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 7>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 7>(&mut self.3, ctx);
// //             <I as Mut<T4>>::each::<4, 7>(&mut self.4, ctx);
// //             <I as Mut<T5>>::each::<5, 7>(&mut self.5, ctx);
// //             <I as Mut<T6>>::each::<6, 7>(&mut self.6, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7> MutOp<I> for (T0, T1, T2, T3, T4, T5, T6, T7)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //         I: Mut<T4>,
// //         I: Mut<T5>,
// //         I: Mut<T6>,
// //         I: Mut<T7>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 8>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 8>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 8>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 8>(&mut self.3, ctx);
// //             <I as Mut<T4>>::each::<4, 8>(&mut self.4, ctx);
// //             <I as Mut<T5>>::each::<5, 8>(&mut self.5, ctx);
// //             <I as Mut<T6>>::each::<6, 8>(&mut self.6, ctx);
// //             <I as Mut<T7>>::each::<7, 8>(&mut self.7, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7, T8> MutOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7, T8)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //         I: Mut<T4>,
// //         I: Mut<T5>,
// //         I: Mut<T6>,
// //         I: Mut<T7>,
// //         I: Mut<T8>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 9>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 9>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 9>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 9>(&mut self.3, ctx);
// //             <I as Mut<T4>>::each::<4, 9>(&mut self.4, ctx);
// //             <I as Mut<T5>>::each::<5, 9>(&mut self.5, ctx);
// //             <I as Mut<T6>>::each::<6, 9>(&mut self.6, ctx);
// //             <I as Mut<T7>>::each::<7, 9>(&mut self.7, ctx);
// //             <I as Mut<T8>>::each::<8, 9>(&mut self.8, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9> MutOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //         I: Mut<T4>,
// //         I: Mut<T5>,
// //         I: Mut<T6>,
// //         I: Mut<T7>,
// //         I: Mut<T8>,
// //         I: Mut<T9>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 10>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 10>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 10>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 10>(&mut self.3, ctx);
// //             <I as Mut<T4>>::each::<4, 10>(&mut self.4, ctx);
// //             <I as Mut<T5>>::each::<5, 10>(&mut self.5, ctx);
// //             <I as Mut<T6>>::each::<6, 10>(&mut self.6, ctx);
// //             <I as Mut<T7>>::each::<7, 10>(&mut self.7, ctx);
// //             <I as Mut<T8>>::each::<8, 10>(&mut self.8, ctx);
// //             <I as Mut<T9>>::each::<9, 10>(&mut self.9, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> MutOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //         I: Mut<T4>,
// //         I: Mut<T5>,
// //         I: Mut<T6>,
// //         I: Mut<T7>,
// //         I: Mut<T8>,
// //         I: Mut<T9>,
// //         I: Mut<T10>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 11>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 11>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 11>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 11>(&mut self.3, ctx);
// //             <I as Mut<T4>>::each::<4, 11>(&mut self.4, ctx);
// //             <I as Mut<T5>>::each::<5, 11>(&mut self.5, ctx);
// //             <I as Mut<T6>>::each::<6, 11>(&mut self.6, ctx);
// //             <I as Mut<T7>>::each::<7, 11>(&mut self.7, ctx);
// //             <I as Mut<T8>>::each::<8, 11>(&mut self.8, ctx);
// //             <I as Mut<T9>>::each::<9, 11>(&mut self.9, ctx);
// //             <I as Mut<T10>>::each::<10, 11>(&mut self.10, ctx);
// //         }
// //     }
// //
// //     impl<I, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> MutOp<I>
// //         for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)
// //     where
// //         I: Mut<T0>,
// //         I: Mut<T1>,
// //         I: Mut<T2>,
// //         I: Mut<T3>,
// //         I: Mut<T4>,
// //         I: Mut<T5>,
// //         I: Mut<T6>,
// //         I: Mut<T7>,
// //         I: Mut<T8>,
// //         I: Mut<T9>,
// //         I: Mut<T10>,
// //         I: Mut<T11>,
// //     {
// //         fn tuple_mut_ctx_mut(&mut self, ctx: &mut I::Context) {
// //             <I as Mut<T0>>::each::<0, 12>(&mut self.0, ctx);
// //             <I as Mut<T1>>::each::<1, 12>(&mut self.1, ctx);
// //             <I as Mut<T2>>::each::<2, 12>(&mut self.2, ctx);
// //             <I as Mut<T3>>::each::<3, 12>(&mut self.3, ctx);
// //             <I as Mut<T4>>::each::<4, 12>(&mut self.4, ctx);
// //             <I as Mut<T5>>::each::<5, 12>(&mut self.5, ctx);
// //             <I as Mut<T6>>::each::<6, 12>(&mut self.6, ctx);
// //             <I as Mut<T7>>::each::<7, 12>(&mut self.7, ctx);
// //             <I as Mut<T8>>::each::<8, 12>(&mut self.8, ctx);
// //             <I as Mut<T9>>::each::<9, 12>(&mut self.9, ctx);
// //             <I as Mut<T10>>::each::<10, 12>(&mut self.10, ctx);
// //             <I as Mut<T11>>::each::<11, 12>(&mut self.11, ctx);
// //         }
// //     }
// // }
