use std::marker::PhantomData;

pub trait ContextPR<'c, 'o> {
    type Context: 'c;
}

pub trait PartPR<'c, 'o, P>: ContextPR<'c, 'o> {
    type Output: 'o;

    fn part<const INDEX: usize, const LEN: usize>(
        part: PhantomData<P>, //nl
        ctx: &'c Self::Context,
    ) -> Self::Output;
}

pub trait TuplePR<'c, 'o, M: ContextPR<'c, 'o>> {
    type Output: 'o;
    fn tuple(self, ctx: &'c M::Context) -> Self::Output;
    #[cfg(feature = "try")]
    fn try_tuple(self, ctx: &'c M::Context) -> Self::Output::Output
    where
        Self::Output: std::ops::Try;
}

mod pr_nl_impls {
    use std::marker::PhantomData;

    use super::{PartPR, TuplePR};

    impl<'c, 'o, M, P0, P1> TuplePR<'c, 'o, M> for PhantomData<(P0, P1)>
    where
        M: PartPR<'c, 'o, P0>,
        M: PartPR<'c, 'o, P1>,
    {
        type Output = (
            <M as PartPR<'c, 'o, P0>>::Output, //nl
            <M as PartPR<'c, 'o, P1>>::Output, //nl
        );
        fn tuple(self, ctx: &'c <M as super::ContextPR<'c, 'o>>::Context) -> Self::Output {
            (
                <M as PartPR<'c, 'o, P0>>::part::<0, 2>(PhantomData, ctx), //nl
                <M as PartPR<'c, 'o, P1>>::part::<1, 2>(PhantomData, ctx), //nl
            )
        }
    }
}
