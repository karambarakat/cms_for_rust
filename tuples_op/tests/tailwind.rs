#![allow(unused)]

use core::panic;
use std::marker::PhantomData;

use tuples_op::behaviors::{
    ContextPR, FnOncePartPhantomCtxRef, FnPartPhantomCtxRef, PartPhantomCtxRef, SupportFnOncePhantomCtxRef, SupportFnPhantomCtxRef, TuplePhantomCtxRef
};

struct Bg(String);
impl FromStr for Bg {
    fn from_str(str: &str) -> Self {
        let this = str.split_whitespace().last().unwrap().to_owned();
        Bg(this)
    }
}
struct ThemeColor(String);
impl FromStr for ThemeColor {
    fn from_str(str: &str) -> Self {
        let this =
            str.split_whitespace().find(|_| true).unwrap().to_owned();
        ThemeColor(this)
    }
}

trait FromStr {
    fn from_str(str: &str) -> Self;
}

trait Manual<Args, Return, Marker: ?Sized + ContextPR> {
    fn calling(&self, ctx: &Marker::Context) -> Return;
}

struct Inv;

impl<Return: Into<i32>> SupportFnOncePhantomCtxRef<Return> for Inv {
    type FnOnceOutput = i32;

    fn support_fn_once_part_phantom_ctx_ref(this: Return) -> Self::FnOnceOutput {
        this.into()
    }
}
impl<Return: Into<i32>> SupportFnPhantomCtxRef<Return> for Inv {
    type FnOutput = i32;

    fn support_fn(this: Return) -> Self::FnOutput {
        this.into()
    }
}

impl ContextPR for Inv {
    type Context = str;
}

impl<P: FromStr> PartPhantomCtxRef<P> for Inv {
    type Output = P;
    fn part_phantom_ctx_ref<const I: usize, const L: usize>(
        ctx: &Self::Context,
    ) -> Self::Output {
        P::from_str(ctx)
    }
}

#[test]
fn tailwind() {
    let str = String::from("hello world");

    fn bg(bg: Bg, cp: ThemeColor) -> i8 {
        panic!("it works {}, {}", bg.0, cp.0);
        todo!()
    }

    let string = String::from("hi");

    let hi = move |bg: Bg, cp: ThemeColor| {
        string;
        2
    };

    hi.fn_once_phantom_ctx_ref_using_marker(
        str.as_str(),
        PhantomData::<Inv>,
    );

    let hi = bg.fn_phantom_ctx_ref_using_marker(
        str.as_str(),
        PhantomData::<Inv>,
    );
}
