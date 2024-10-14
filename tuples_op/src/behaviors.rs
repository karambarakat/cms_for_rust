use std::marker::PhantomData;

pub trait ContextMR<'p, 'c, 'o> {
    type Context: 'c;
}

pub trait PartMutCtxRef<'p, 'c, 'o, E>:
    ContextMR<'p, 'c, 'o>
{
    type Output: 'o;
    fn part<const INDEX: usize, const LEN: usize>(
        part: &'p mut E,
        ctx: &'c Self::Context,
    ) -> Self::Output;
}

pub trait LastMutCtxRef<'p, 'c, 'o, E>:
    ContextMR<'p, 'c, 'o>
{
    type Output: 'o;
    fn last<const INDEX: usize>(
        part: &'p mut E,
        ctx: Self::Context,
    ) -> Self::Output;
}

pub trait TupleMutCtxRef<'t, 'c, 'o, M>
where
    M: ContextMR<'t, 'c, 'o>,
{
    type Output: 'o;
    fn tuple_mut_ctx_ref(
        &'t mut self,
        ctx: &'c M::Context,
    ) -> Self::Output;
}

mod impls_mr {
    use super::*;
    impl<'t, 'c, 'o, M> TupleMutCtxRef<'t, 'c, 'o, M> for ()
    where
        M: ContextMR<'t, 'c, 'o>,
    {
        type Output = ();
        fn tuple_mut_ctx_ref(
            &'t mut self,
            _: &'c M::Context,
        ) -> Self::Output {
            ()
        }
    }

    impl<'t, 'c, 'o, M, P0> TupleMutCtxRef<'t, 'c, 'o, M> for (P0,)
    where
        M: PartMutCtxRef<'t, 'c, 'o, P0>,
    {
        type Output =
            (<M as PartMutCtxRef<'t, 'c, 'o, P0>>::Output,);

        fn tuple_mut_ctx_ref(
            &'t mut self,
            ctx: &'c M::Context,
        ) -> Self::Output {
            (<M as PartMutCtxRef<'t, 'c, 'o, P0>>::part::<0, 1>(
                &mut self.0,
                ctx,
            ),)
        }
    }
    impl<'t, 'c, 'o, M, P0, P1> TupleMutCtxRef<'t, 'c, 'o, M>
        for (P0, P1)
    where
        M: PartMutCtxRef<'t, 'c, 'o, P0>,
        M: PartMutCtxRef<'t, 'c, 'o, P1>,
    {
        type Output = (
            <M as PartMutCtxRef<'t, 'c, 'o, P0>>::Output,
            <M as PartMutCtxRef<'t, 'c, 'o, P1>>::Output,
        );

        fn tuple_mut_ctx_ref(
            &'t mut self,
            ctx: &'c M::Context,
        ) -> Self::Output {
            (
                <M as PartMutCtxRef<'t, 'c, 'o, P0>>::part::<0, 2>(
                    &mut self.0,
                    ctx,
                ),
                <M as PartMutCtxRef<'t, 'c, 'o, P1>>::part::<1, 2>(
                    &mut self.1,
                    ctx,
                ),
            )
        }
    }

    impl<'t, 'c, 'o, M, P0, P1, P2> TupleMutCtxRef<'t, 'c, 'o, M>
        for (P0, P1, P2)
    where
        M: PartMutCtxRef<'t, 'c, 'o, P0>,
        M: PartMutCtxRef<'t, 'c, 'o, P1>,
        M: PartMutCtxRef<'t, 'c, 'o, P2>,
    {
        type Output = (
            <M as PartMutCtxRef<'t, 'c, 'o, P0>>::Output,
            <M as PartMutCtxRef<'t, 'c, 'o, P1>>::Output,
            <M as PartMutCtxRef<'t, 'c, 'o, P2>>::Output,
        );

        fn tuple_mut_ctx_ref(
            &'t mut self,
            ctx: &'c M::Context,
        ) -> Self::Output {
            (
                <M as PartMutCtxRef<'t, 'c, 'o, P0>>::part::<0, 3>(
                    &mut self.0,
                    ctx,
                ),
                <M as PartMutCtxRef<'t, 'c, 'o, P1>>::part::<1, 3>(
                    &mut self.1,
                    ctx,
                ),
                <M as PartMutCtxRef<'t, 'c, 'o, P2>>::part::<2, 3>(
                    &mut self.2,
                    ctx,
                ),
            )
        }
    }

    impl<'t, 'c, 'o, M, P0, P1, P2, P3>
        TupleMutCtxRef<'t, 'c, 'o, M> for (P0, P1, P2, P3)
    where
        M: PartMutCtxRef<'t, 'c, 'o, P0>,
        M: PartMutCtxRef<'t, 'c, 'o, P1>,
        M: PartMutCtxRef<'t, 'c, 'o, P2>,
        M: PartMutCtxRef<'t, 'c, 'o, P3>,
    {
        type Output = (
            <M as PartMutCtxRef<'t, 'c, 'o, P0>>::Output,
            <M as PartMutCtxRef<'t, 'c, 'o, P1>>::Output,
            <M as PartMutCtxRef<'t, 'c, 'o, P2>>::Output,
            <M as PartMutCtxRef<'t, 'c, 'o, P3>>::Output,
        );

        fn tuple_mut_ctx_ref(
            &'t mut self,
            ctx: &'c M::Context,
        ) -> Self::Output {
            (
                <M as PartMutCtxRef<'t, 'c, 'o, P0>>::part::<0, 4>(
                    &mut self.0,
                    ctx,
                ),
                <M as PartMutCtxRef<'t, 'c, 'o, P1>>::part::<1, 4>(
                    &mut self.1,
                    ctx,
                ),
                <M as PartMutCtxRef<'t, 'c, 'o, P2>>::part::<2, 4>(
                    &mut self.2,
                    ctx,
                ),
                <M as PartMutCtxRef<'t, 'c, 'o, P3>>::part::<3, 4>(
                    &mut self.3,
                    ctx,
                ),
            )
        }
    }
}

pub trait ContextCM {
    type Context;
}

pub trait PartConsumeCtxMut<E>: ContextCM {
    type Output;
    fn part<const INDEX: usize, const LEN: usize>(
        part: E,
        ctx: &mut Self::Context,
    ) -> Self::Output;
}

pub trait ContextCR<'c, 'o> {
    type Context: 'c;
}

pub trait PartConsumeCtxRef<'c, 'o, E>:
    ContextCR<'c, 'o>
{
    type Output: 'o;
    // type Input;
    fn part<const INDEX: usize, const LEN: usize>(
        // part: Self::Input,
        ctx: &'c Self::Context,
    ) -> Self::Output;
}

pub trait TupleConsumeCtxRef<'c, 'o, I: ContextCR<'c, 'o>> {
    type Output;
    fn tuple(ctx: &'c I::Context) -> Self::Output;
}

mod impl_cr {
    use super::PartConsumeCtxRef;
    use super::TupleConsumeCtxRef;
    impl<'c, 'o, I, E0, E1> TupleConsumeCtxRef<'c, 'o, I>
        for (E0, E1)
    where
        I: PartConsumeCtxRef<'c, 'o, E0>,
        I: PartConsumeCtxRef<'c, 'o, E1>,
    {
        type Output = (
            <I as PartConsumeCtxRef<'c, 'o, E0>>::Output,
            <I as PartConsumeCtxRef<'c, 'o, E1>>::Output,
        );
        fn tuple(
            ctx: &'c <I as super::ContextCR<'c, 'o>>::Context,
        ) -> Self::Output {
            (
                <I as PartConsumeCtxRef<'c, 'o, E0>>::part::<0, 2>(
                    ctx,
                ),
                <I as PartConsumeCtxRef<'c, 'o, E1>>::part::<1, 2>(
                    ctx,
                ),
            )
        }
    }
}

pub trait TupleConsumeCtxMut<M>
where
    M: ContextCM,
{
    type Output;
    fn tuple_consume_ctx_mut(
        self,
        ctx: &mut M::Context,
    ) -> Self::Output;
}

mod impl_cm {
    use super::*;

    impl<M> TupleConsumeCtxMut<M> for ()
    where
        M: ContextCM,
    {
        type Output = ();
        fn tuple_consume_ctx_mut(
            self,
            _: &mut M::Context,
        ) -> Self::Output {
            ()
        }
    }

    impl<M, P0> TupleConsumeCtxMut<M> for (P0,)
    where
        M: PartConsumeCtxMut<P0>,
    {
        type Output = <M as PartConsumeCtxMut<P0>>::Output;
        fn tuple_consume_ctx_mut(
            self,
            ctx: &mut M::Context,
        ) -> Self::Output {
            <M as PartConsumeCtxMut<P0>>::part::<0, 1>(
                self.0, ctx,
            )
        }
    }

    impl<M, P0, P1> TupleConsumeCtxMut<M> for (P0, P1)
    where
        M: PartConsumeCtxMut<P0>,
        M: PartConsumeCtxMut<P1>,
    {
        type Output = (
            <M as PartConsumeCtxMut<P0>>::Output,
            <M as PartConsumeCtxMut<P1>>::Output,
        );
        fn tuple_consume_ctx_mut(
            self,
            ctx: &mut M::Context,
        ) -> Self::Output {
            (
                <M as PartConsumeCtxMut<P0>>::part::<0, 2>(
                    self.0, ctx,
                ),
                <M as PartConsumeCtxMut<P1>>::part::<1, 2>(
                    self.1, ctx,
                ),
            )
        }
    }

    impl<M, P0, P1, P2> TupleConsumeCtxMut<M> for (P0, P1, P2)
    where
        M: PartConsumeCtxMut<P0>,
        M: PartConsumeCtxMut<P1>,
        M: PartConsumeCtxMut<P2>,
    {
        type Output = (
            <M as PartConsumeCtxMut<P0>>::Output,
            <M as PartConsumeCtxMut<P1>>::Output,
            <M as PartConsumeCtxMut<P2>>::Output,
        );
        fn tuple_consume_ctx_mut(
            self,
            ctx: &mut M::Context,
        ) -> Self::Output {
            (
                <M as PartConsumeCtxMut<P0>>::part::<0, 3>(
                    self.0, ctx,
                ),
                <M as PartConsumeCtxMut<P1>>::part::<1, 3>(
                    self.1, ctx,
                ),
                <M as PartConsumeCtxMut<P2>>::part::<2, 3>(
                    self.2, ctx,
                ),
            )
        }
    }

    impl<M, P0, P1, P2, P3> TupleConsumeCtxMut<M>
        for (P0, P1, P2, P3)
    where
        M: PartConsumeCtxMut<P0>,
        M: PartConsumeCtxMut<P1>,
        M: PartConsumeCtxMut<P2>,
        M: PartConsumeCtxMut<P3>,
    {
        type Output = (
            <M as PartConsumeCtxMut<P0>>::Output,
            <M as PartConsumeCtxMut<P1>>::Output,
            <M as PartConsumeCtxMut<P2>>::Output,
            <M as PartConsumeCtxMut<P3>>::Output,
        );
        fn tuple_consume_ctx_mut(
            self,
            ctx: &mut M::Context,
        ) -> Self::Output {
            (
                <M as PartConsumeCtxMut<P0>>::part::<0, 4>(
                    self.0, ctx,
                ),
                <M as PartConsumeCtxMut<P1>>::part::<1, 4>(
                    self.1, ctx,
                ),
                <M as PartConsumeCtxMut<P2>>::part::<2, 4>(
                    self.2, ctx,
                ),
                <M as PartConsumeCtxMut<P3>>::part::<3, 4>(
                    self.3, ctx,
                ),
            )
        }
    }
}

pub trait ContextPR {
    type Context: ?Sized;
}

pub trait PartPhantomCtxRef<P>: ContextPR {
    type Output;
    fn part_phantom_ctx_ref<
        const INDEX: usize,
        const LEN: usize,
    >(
        ctx: &Self::Context,
    ) -> Self::Output;
}

// pub trait LastPhantomCtxRef<P>
// where
//     Self: ContextPR,
//     Self::Context: Sized,
// {
// }

pub trait TuplePhantomCtxRef<M: ContextPR> {
    type Output;
    fn tuple_phantom_ctx_ref(
        ctx: &<M as ContextPR>::Context,
    ) -> Self::Output;
}

pub trait TuplePhantomCtxRefWithLast<L, M>
where
    M: ContextPR + LastPhantomCtxRef<L, M>,
    M::Context: Sized,
{
    type OutputLast;
    fn tuple_phantom_ctx_ref_with_last(
        ctx: <M as ContextPR>::Context,
    ) -> Self::OutputLast;
}

pub trait FnPartPhantomCtxRef<M, Args, Return>
where
    M: ContextPR,
    M: SupportFnPhantomCtxRef<Return>,
{
    fn fn_phantom_ctx_ref(
        &self,
        ctx: &M::Context,
    ) -> M::FnOutput;
    fn fn_phantom_ctx_ref_using_marker(
        &self,
        ctx: &M::Context,
        _marker: PhantomData<M>,
    ) -> M::FnOutput {
        self.fn_phantom_ctx_ref(ctx)
    }
}

pub trait ContextRR<'p, 'c, 'o> {
    type Context: 'c;
}

pub trait PartRefCtxRef<'p, 'c, 'o, E>:
    ContextRR<'p, 'c, 'o>
{
    type Output: 'o;
    fn part_ref_ctx_ref(
        part: &'p E,
        ctx: &'c Self::Context,
    ) -> Self::Output;
}

pub trait TupleRefCtxRef<'t, 'c, 'o, M>
where
    M: ContextRR<'t, 'c, 'o>,
{
    type Output: 'o;
    fn tuple_ref_ctx_ref(
        &'t self,
        ctx: &'c M::Context,
    ) -> Self::Output;
}

impl<'t, 'c, 'o, M, P0, P1> TupleRefCtxRef<'t, 'c, 'o, M>
    for (P0, P1)
where
    M: PartRefCtxRef<'t, 'c, 'o, P0>,
    M: PartRefCtxRef<'t, 'c, 'o, P1>,
{
    type Output = (
        <M as PartRefCtxRef<'t, 'c, 'o, P0>>::Output,
        <M as PartRefCtxRef<'t, 'c, 'o, P1>>::Output,
    );
    fn tuple_ref_ctx_ref(
        &'t self,
        ctx: &'c <M as ContextRR<'t, 'c, 'o>>::Context,
    ) -> Self::Output {
        (
            <M as PartRefCtxRef<'t, 'c, 'o, P0>>::part_ref_ctx_ref(
                &self.0, ctx,
            ),
            <M as PartRefCtxRef<'t, 'c, 'o, P1>>::part_ref_ctx_ref(
                &self.1, ctx,
            ),
        )
    }
}

pub trait SupportFnPartRefCtxRef<Return> {
    type FnOutput;
    fn support_fn_part_ref_ctx_ref(
        this: Return,
    ) -> Self::FnOutput;
}

pub trait FnPartRefCtxRef<'t, 'c, 'o, M, Tuple, Return>
where
    Self: Sized,
    M: ContextRR<'t, 'c, 'o>,
    M: SupportFnPartRefCtxRef<Return>,
{
    fn fn_part_ref_ctx_ref(
        &self,
        tuple: &'t Tuple,
        ctx: &'c <M as ContextRR<'t, 'c, 'o>>::Context,
    ) -> M::FnOutput;
}

impl<'t, 'c, 'o, M, P0, P1, R, F>
    FnPartRefCtxRef<'t, 'c, 'o, M, (P0, P1), R> for F
where
    M: PartRefCtxRef<'t, 'c, 'o, P0>,
    M: PartRefCtxRef<'t, 'c, 'o, P1>,
    M: SupportFnPartRefCtxRef<R>,

    F: Fn(
        <M as PartRefCtxRef<'t, 'c, 'o, P0>>::Output,
        <M as PartRefCtxRef<'t, 'c, 'o, P1>>::Output,
    ) -> R,
    M: ContextRR<'t, 'c, 'o>,
{
    fn fn_part_ref_ctx_ref(
        &self,
        tuple: &'t (P0, P1),
        ctx: &'c <M as ContextRR<'t, 'c, 'o>>::Context,
    ) -> <M as SupportFnPartRefCtxRef<R>>::FnOutput {
        let part0 = <M as PartRefCtxRef<'t, 'c, 'o, P0>>::part_ref_ctx_ref( &tuple.0, ctx,);
        let part1 = <M as PartRefCtxRef<'t, 'c, 'o, P1>>::part_ref_ctx_ref( &tuple.1, ctx,);

        let result = self(part0, part1);

        <M as SupportFnPartRefCtxRef<R>>::support_fn_part_ref_ctx_ref(result)
    }
}

pub trait SupportFnPhantomCtxRef<Return> {
    type FnOutput;
    fn support_fn(this: Return) -> Self::FnOutput;
}

pub trait SupportFnOncePhantomCtxRef<Return> {
    type FnOnceOutput;
    fn support_fn_once_part_phantom_ctx_ref(
        this: Return,
    ) -> Self::FnOnceOutput;
}

pub trait FnOncePartPhantomCtxRef<M, Args, Return>
where
    Self: Sized,
    M: ContextPR,
    M: SupportFnOncePhantomCtxRef<Return>,
{
    fn fn_once_phantom_ctx_ref(
        self,
        ctx: &M::Context,
    ) -> M::FnOnceOutput;
    fn fn_once_phantom_ctx_ref_using_marker(
        self,
        ctx: &M::Context,
        _marker: PhantomData<M>,
    ) -> M::FnOnceOutput {
        self.fn_once_phantom_ctx_ref(ctx)
    }
}

impl<M, P0, P1, R, F> FnOncePartPhantomCtxRef<M, (P0, P1), R>
    for F
where
    M: PartPhantomCtxRef<P0>,
    M: PartPhantomCtxRef<P1>,
    M: SupportFnOncePhantomCtxRef<R>,
    F: FnOnce(
        <M as PartPhantomCtxRef<P0>>::Output,
        <M as PartPhantomCtxRef<P1>>::Output,
    ) -> R,
    M: ContextPR,
{
    fn fn_once_phantom_ctx_ref(
        self,
        ctx: &M::Context,
    ) -> M::FnOnceOutput {
        let part0 =
            <M as PartPhantomCtxRef<P0>>::part_phantom_ctx_ref::<
                0,
                2,
            >(ctx);
        let part1 =
            <M as PartPhantomCtxRef<P1>>::part_phantom_ctx_ref::<
                0,
                2,
            >(ctx);

        let result = self(part0, part1);

        <M as SupportFnOncePhantomCtxRef<R>>::support_fn_once_part_phantom_ctx_ref(result)
    }
}

impl<M, P0, P1, R, F> FnPartPhantomCtxRef<M, (P0, P1), R> for F
where
    M: PartPhantomCtxRef<P0>,
    M: PartPhantomCtxRef<P1>,
    M: SupportFnPhantomCtxRef<R>,
    F: Fn(
        <M as PartPhantomCtxRef<P0>>::Output,
        <M as PartPhantomCtxRef<P1>>::Output,
    ) -> R,
    M: ContextPR,
{
    fn fn_phantom_ctx_ref(
        &self,
        ctx: &M::Context,
    ) -> M::FnOutput {
        let part0 =
            <M as PartPhantomCtxRef<P0>>::part_phantom_ctx_ref::<
                0,
                2,
            >(ctx);
        let part1 =
            <M as PartPhantomCtxRef<P1>>::part_phantom_ctx_ref::<
                0,
                2,
            >(ctx);

        let result = self(part0, part1);

        <M as SupportFnPhantomCtxRef<R>>::support_fn(result)
    }
}

pub trait LastPhantomCtxRef<L, M>
where
    Self: TuplePhantomCtxRef<M>,
    M: ContextPR,
    <M as ContextPR>::Context: Sized,
{
    type LastOutput;
    fn last_phantom_ctx_ref<
        const INDEX: usize,
        const LEN: usize,
    >(
        ctx: M::Context,
    ) -> Self::LastOutput;
}

impl<T0, T1, M> TuplePhantomCtxRef<M> for (T0, T1)
where
    M: PartPhantomCtxRef<T0>,
    M: PartPhantomCtxRef<T1>,
{
    type Output = (
        <M as PartPhantomCtxRef<T0>>::Output,
        <M as PartPhantomCtxRef<T1>>::Output,
    );

    fn tuple_phantom_ctx_ref(
        ctx: &<M as ContextPR>::Context,
    ) -> Self::Output {
        (
            <M as PartPhantomCtxRef<T0>>::part_phantom_ctx_ref::<
                0,
                2,
            >(ctx),
            <M as PartPhantomCtxRef<T1>>::part_phantom_ctx_ref::<
                0,
                2,
            >(ctx),
        )
    }
}

impl<T0, T1, M> TuplePhantomCtxRefWithLast<T1, M> for (T0, T1)
where
    M: PartPhantomCtxRef<T0>,
    M: LastPhantomCtxRef<T1, M>,
    M::Context: Sized,
{
    type OutputLast = (
        <M as PartPhantomCtxRef<T0>>::Output,
        <M as LastPhantomCtxRef<T1, M>>::LastOutput,
    );

    fn tuple_phantom_ctx_ref_with_last(
        ctx: <M as ContextPR>::Context,
    ) -> Self::OutputLast {
        (
            <M as PartPhantomCtxRef<T0>>::part_phantom_ctx_ref::<0, 2>(
                &ctx,
            ),
            <M as LastPhantomCtxRef<T1, M>>::last_phantom_ctx_ref::<
                1,
                2,
            >(ctx),
        )
    }
}
