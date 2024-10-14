#![allow(unused)]
trait Example<'c> {
    type Output;
    fn trans(ctx: &'c String) -> Self::Output;
}

impl<'c1> Example<'c1> for &'c1 str {
    type Output = &'c1 str;
    fn trans(ctx: &'c1 String) -> Self::Output {
        ctx.as_str()
    }
}

#[test]
fn client_new() {
    trait TupleOp<'c, I: RefContext<'c>> {
        type Output;
        fn op(ctx: <I as RefContext<'c>>::Context) -> Self::Output;
    }

    struct MyI;

    trait Each<'c, E>: RefContext<'c> {
        type Ret;
        fn each(ctx: <Self as RefContext<'c>>::Context) -> Self::Ret;
    }

    trait Context {
        type Context;
    }

    impl Context for MyI {
        type Context = String;
    }

    trait RefContext<'c> {
        type Context;
    }

    impl<'c, T> RefContext<'c> for T
    where
        T: Context,
        T::Context: 'c,
    {
        type Context = &'c T::Context;
    }

    impl<'c, E: Example<'c>> Each<'c, E> for MyI {
        type Ret = E::Output;
        fn each(ctx: &'c String) -> Self::Ret {
            E::trans(ctx)
        }
    }

    impl<'c, I, T0, T1> TupleOp<'c, I> for (T0, T1)
    where
        I: RefContext<'c>,
        <I as RefContext<'c>>::Context: Clone,
        I: Each<'c, T0>,
        I: Each<'c, T1>,
    {
        type Output =
            (<I as Each<'c, T0>>::Ret, <I as Each<'c, T1>>::Ret);
        fn op(ctx: <I as RefContext<'c>>::Context) -> Self::Output {
            (
                <I as Each<'c, T0>>::each(ctx.clone()),
                <I as Each<'c, T1>>::each(ctx.clone()),
            )
        }
    }

    let str = String::from("Hello, World!");

    let output = <&str as Example>::trans(&str);

    assert_eq!(output, "Hello, World!");

    let output = <(&str, &str) as TupleOp<MyI>>::op(&str);

    assert_eq!(output, ("", ""));
}

// ------------------------------------------ IGNORE ---------------
trait PartPhantomCtxMut<E> {
    type Output: 'static;
    fn part<'c1>(ctx: &'c1 mut String) -> Self::Output;
}

struct PPCM;

impl PartPhantomCtxMut<i32> for PPCM {
    type Output = i32;
    fn part<'c1>(ctx: &'c1 mut String) -> Self::Output {
        ctx.push_str("hi");
        3
    }
}

trait TuplePhantomCtxMut<I> {
    type Output: 'static;
    fn tuple(ctx: &mut String) -> Self::Output;
}

impl<'c, I, T0, T1> TuplePhantomCtxMut<I> for (T0, T1)
where
    I: PartPhantomCtxMut<T0>,
    I: PartPhantomCtxMut<T1>,
{
    type Output = (
        <I as PartPhantomCtxMut<T0>>::Output,
        <I as PartPhantomCtxMut<T1>>::Output,
    );

    fn tuple(ctx: &mut String) -> Self::Output {
        (
            <I as PartPhantomCtxMut<T0>>::part(ctx),
            <I as PartPhantomCtxMut<T1>>::part(ctx),
        )
    }
}

fn client() {
    let mut str = String::new();
    let hi =
        <(i32, i32) as TuplePhantomCtxMut<PPCM>>::tuple(&mut str);
}

// ----------------------------------------------------------------
trait ContextMRY<'p, 'c, 'o> {
    type Context: 'c;
}
trait PartMutCtxRefYield<'p, 'c, 'o, E>: ContextMRY<'p, 'c, 'o> {
    type Output: 'o;
    fn part(part: &'p mut E, ctx: &'c Self::Context) -> Self::Output;
}

struct Invaraint3;

impl<'p, 'c> ContextMRY<'p, 'c, 'p> for Invaraint3 {
    type Context = String;
}

trait PartTrait3<'x> {
    type Output: 'x;
    fn part(&'x mut self, ctx: &String) -> Self::Output;
}

impl<'p, 'c, E: PartTrait3<'p>> PartMutCtxRefYield<'p, 'c, 'p, E>
    for Invaraint3
where
    // E::Output: 'p,
{
    type Output = E::Output;
    fn part(part: &'p mut E, ctx: &'c String) -> Self::Output {
        E::part(part, ctx)
    }
}

// Result<(), (True, Option<Span>, String)>

trait TupleMutCtxRefYield<'t, 'c, 'o, I>
where
    I: ContextMRY<'t, 'c, 'o>,
{
    type Output: 'o;
    fn tuple(&'t mut self, ctx: &'c I::Context) -> Self::Output;
}

impl<'t, 'c, 'o, I, T0, T1> TupleMutCtxRefYield<'t, 'c, 'o, I>
    for (T0, T1)
where
    I: PartMutCtxRefYield<'t, 'c, 'o, T0>,
    I: PartMutCtxRefYield<'t, 'c, 'o, T1>,
{
    type Output = (
        <I as PartMutCtxRefYield<'t, 'c, 'o, T0>>::Output,
        <I as PartMutCtxRefYield<'t, 'c, 'o, T1>>::Output,
    );

    fn tuple(
        &'t mut self,
        ctx: &'c <I as ContextMRY<'t, 'c, 'o>>::Context,
    ) -> Self::Output {
        (
            <I as PartMutCtxRefYield<'t, 'c, 'o, T0>>::part(
                &mut self.0,
                ctx,
            ),
            <I as PartMutCtxRefYield<'t, 'c, 'o, T1>>::part(
                &mut self.1,
                ctx,
            ),
        )
    }
}

#[test]
fn client_mut() {
    let mut tuple = (String::from("hi"), 32);

    let str = String::from("hel");

    impl<'x> PartTrait3<'x> for String {
        type Output = &'x str;
        fn part(&'x mut self, ctx: &String) -> Self::Output {
            self.push_str(ctx);
            self.as_str()
        }
    }

    impl PartTrait3<'_> for i32 {
        type Output = i32;
        fn part(&mut self, ctx: &String) -> Self::Output {
            *self += ctx.len() as i32;
            *self
        }
    }

    let output = TupleMutCtxRefYield::<'_, '_, '_, Invaraint3>::tuple(
        &mut tuple, &str,
    );

    assert_eq!(output, ("hihel", 35));

}
