#![allow(unused)]
use core::fmt;
use internal::{
    better_compare, pretty_compare, str_to_debug, ParsingStream,
};
use std::ops::Not;

// #[cfg(not(feature = "axum"))]
// pub(crate) mod axum;

#[cfg(feature = "axum")]
pub mod axum;

mod internal;
mod test_internals;
#[deprecated]
pub mod unpretty;

pub trait SimilarAsyc {
}

pub struct Expect<T>(pub(crate) T);

pub fn ts<'r, T: ?Sized>(val: &'r T) -> Expect<&'r T> {
    Expect(val)
}
pub fn expect<'r, T: ?Sized>(val: &'r T) -> Expect<&'r T> {
    Expect(val)
}

/// perform comarison, this is the stable API for trait `Comparable`
pub trait ToBe {
    type This;

    #[track_caller]
    fn to_be(&self, other: &Self::This);
}

// impl<'r, T: ToBe> Expect<&'r T> {
//     pub fn to_be(self, other: &T::This) {
//         self.0.to_be(&other);
//     }
// }

pub type SimilarResult = Result<(), String>;

pub trait Similar<With> {
    #[track_caller]
    fn similar(&self, other: With) -> SimilarResult;
}

impl Similar<&'static str> for str {
    fn similar(&self, other: &'static str) -> SimilarResult {
        ().compare(self, other);
        Ok(())
    }
}

impl Similar<&'static str> for String {
    fn similar(&self, other: &'static str) -> SimilarResult {
        ().compare(self, other);
        Ok(())
    }
}

impl<'r, T: ?Sized> Expect<&'r T> {
    pub fn similar<To>(self, to: To) -> Self
    where
        T: Similar<To>,
    {
        match self.0.similar(to) {
            Ok(_) => {}
            Err(err) => panic!("{}", err),
        }
        self
    }
}

#[allow(unused)]
impl<'r, T: std::fmt::Debug + PartialEq> ToBe for T {
    type This = T;
    #[track_caller]
    fn to_be(&self, other: &Self::This) {
        if self.eq(other) {
            return;
        }

        let expect = str_to_debug(
            ParsingStream::new(&mut format!("{:?}", self)), // &mut format!("{:?}", self).chars().peekable(),
        );

        let to_be = str_to_debug(
            ParsingStream::new(&mut format!("{:?}", other)), // &mut format!("{:?}", self).chars().peekable(),
        );

        let (expect, to_be) = match (expect, to_be) {
            (Ok(a1), Ok(a2)) => (a1, a2),
            (Ok(_), Err(err)) => {
                // error!("debug parsing err {err}");
                panic!("expected: {self:?}\n   to be: {other:?}")
            }
            (Err(err), Ok(_)) => {
                // error!("debug parsing err {err}");
                panic!("expected: {self:?}\n   to be: {other:?}")
            }
            (Err(e1), Err(e2)) => {
                // error!("debug parsing err {e1} and {e2}");
                panic!("expected: {self:?}\n   to be: {other:?}")
            }
        };

        internal::value_to_be(&expect, &to_be, "#".to_owned());
    }
}

impl<'r, T: ToBe> ToBe for Expect<&'r T> {
    type This = T::This;
    #[track_caller]
    fn to_be(&self, other: &Self::This) {
        self.0.to_be(other)
    }
}

impl DisplaySpec for ((),) {
    fn compare(&self, expect: &str, to_be: &'static str) {
        better_compare(expect, to_be);
    }
}

impl DisplaySpec for () {
    fn compare(&self, expect: &str, to_be: &'static str) {
        pretty_compare(expect, to_be);
    }
}

pub trait DisplaySpec {
    fn compare(&self, expect: &str, to_be: &'static str);
}

impl<'r, T: ?Sized> Expect<&'r T> {
    /// do a "pretty" string comparison for types that
    /// implements `Display`, like this:
    ///
    /// ```rust,ignore
    /// // assume this is a very long string or it is obtained
    /// // from external source
    /// expect("create table Todo { id int, title string }")
    ///     .displaying(
    ///         "
    ///         create table Todo {
    ///             id int,
    ///             title string
    ///         }
    ///     ",
    ///     ()
    ///     );
    /// ```
    ///
    /// the default displaying specifier `()` will ignore any
    /// white space between letters and symbols in both expect
    /// and to_be strings, this may result in incorrect
    /// positive comparison in some syntaxes so provide the
    /// correct specifier
    ///
    /// this is a syntax sugar over `better_testing::displaying`
    pub fn displaying(
        self,
        to_be: &'static str,
        spec: impl DisplaySpec,
    ) -> Self
    where
        &'r T: ToString,
    {
        spec.compare(&self.0.to_string(), to_be);
        self
    }
}

pub struct DisplayingImplSimilar<Spec>(&'static str, Spec);

pub fn displaying<T: DisplaySpec>(
    val: &'static str,
    spec: T,
) -> DisplayingImplSimilar<T> {
    DisplayingImplSimilar(val, spec)
}

impl<Spec, This: ?Sized> Similar<DisplayingImplSimilar<Spec>>
    for This
where
    Spec: DisplaySpec,
    for<'r> &'r This: ToString,
{
    fn similar(
        &self,
        other: DisplayingImplSimilar<Spec>,
    ) -> SimilarResult {
        other.1.compare(&self.to_string(), other.0);
        Ok(())
    }
}

pub mod ord_mod {
    use std::{fmt, marker::PhantomData, ops::Not};

    use crate::{Similar, SimilarResult};

    pub struct OrdBuilder<T> {
        to_data: PhantomData<T>,
        less_than: Option<T>,
        greater_than: Option<T>,
        equal: Option<T>,
    }
    pub fn ord<T>() -> OrdBuilder<T> {
        OrdBuilder {
            to_data: PhantomData,
            less_than: None,
            greater_than: None,
            equal: None,
        }
    }

    impl<T> OrdBuilder<T> {
        pub fn less_than(mut self, other: T) -> Self {
            self.less_than = Some(other);
            self
        }

        pub fn greater_than(mut self, other: T) -> Self {
            self.greater_than = Some(other);
            self
        }
        pub fn equal(mut self, other: T) -> Self {
            self.equal = Some(other);
            self
        }
    }

    pub struct Or<Phantom, T, O>(PhantomData<Phantom>, T, O);

    impl<To, C1, C2> Similar<Or<To, C1, C2>> for To
    where
        To: Similar<C1>,
        To: Similar<C2>,
    {
        fn similar(
            &self,
            condition: Or<To, C1, C2>,
        ) -> SimilarResult {
            let c1 = self.similar(condition.1);
            let c2 = self.similar(condition.2);

            match (c1, c2) {
                (Ok(_), Ok(_)) => Ok(()),
                (Err(_), Ok(_)) => Ok(()),
                (Ok(_), Err(_)) => Ok(()),
                (Err(e1), Err(e2)) => Err(format!(
                    "both conditions failed: \n{}\n{}",
                    e1, e2
                )),
            }
        }
    }

    pub fn or<T, C1, C2>(c1: C1, c2: C2) -> Or<T, C1, C2>
    where
        T: Similar<C1>,
        T: Similar<C2>,
    {
        Or(PhantomData, c1, c2)
    }

    impl<T: Ord + fmt::Debug> Similar<OrdBuilder<T>> for T {
        fn similar(
            &self,
            other: OrdBuilder<T>,
        ) -> SimilarResult {
            if let Some(less_than) = other.less_than {
                if (self < &less_than).not() {
                    return Err(format!(
                        "expected: {:?} to be less than {:?}",
                        self, less_than
                    ));
                }
            }
            if let Some(greater_than) = other.greater_than {
                if (self > &greater_than).not() {
                    return Err(format!(
                        "expected: {:?} to be greater than {:?}",
                        self, greater_than
                    ));
                }
            }
            if let Some(equal) = other.equal {
                if (self == &equal).not() {
                    return Err(format!(
                        "expected: {:?} to be equal to {:?}",
                        self, equal
                    ));
                }
            }

            Ok(())
        }
    }
}

pub struct NotToBe<T>(T);

impl<'r, T: std::fmt::Debug + PartialEq> ToBe
    for NotToBe<&'r T>
{
    type This = T;
    #[track_caller]
    fn to_be(&self, other: &Self::This) {
        if self.0.eq(other) {
            panic! {"values are equal, found: \n{:?}", self.0}
        }
    }
}
impl<'r, T: fmt::Debug + PartialEq> Expect<&'r T> {
    pub fn not(self) -> NotToBe<&'r T> {
        NotToBe(self.0)
    }
}

pub struct DefaultToAssert<T>(T);
impl<'r, T: fmt::Debug + PartialEq> Expect<&'r T> {
    /// I put this method just in case there is any bug in my
    /// crate, you can use this method to go back to std's
    /// `assert_eq!` behavior.
    pub fn default(self) -> DefaultToAssert<&'r T> {
        DefaultToAssert(self.0)
    }
}

impl<'r, T: std::fmt::Debug + PartialEq> ToBe
    for DefaultToAssert<&'r T>
{
    type This = T;
    #[track_caller]
    fn to_be(&self, other: &Self::This) {
        if self.0.eq(other).not() {
            panic! {"assertion failed \nexpected: {:?}\n   to be: {:?}", self.0, other}
        }
    }
}

pub struct RelyOnDebug<T>(T);

impl<'r, T: fmt::Debug + PartialEq> Expect<&'r T> {
    /// In the future when rust stebalizes `specialization`
    /// feature, I might make the blanket implementation (`Debug
    /// + PartialEq`) of `ToBe` a default implementation, in that
    /// case you might want to opt out of the specialized impl
    /// by calling this method. this method guerantee you to rely
    /// on `Debug + PartialEq` implementation.
    ///
    /// I don't see a any use case for other crate to specialize
    /// `ToBe` but I will do this for backward compatibility
    pub fn debug(self) -> RelyOnDebug<&'r T> {
        RelyOnDebug(self.0)
    }
}

#[allow(unused)]
impl<'r, T: fmt::Debug + PartialEq> ToBe for RelyOnDebug<&'r T> {
    type This = T;
    #[track_caller]
    fn to_be(&self, other: &Self::This) {
        if self.0.eq(other) {
            return;
        }

        let expect = str_to_debug(
            ParsingStream::new(&mut format!("{:?}", self.0)), // &mut format!("{:?}", self).chars().peekable(),
        );

        let to_be = str_to_debug(
            ParsingStream::new(&mut format!("{:?}", other)), // &mut format!("{:?}", self).chars().peekable(),
        );

        let (expect, to_be) = match (expect, to_be) {
            (Ok(a1), Ok(a2)) => (a1, a2),
            (Ok(_), Err(err)) => {
                // error!("debug parsing err {err}");
                panic!(
                    "expected: {:?}\n   to be: {:?}",
                    self.0, other
                )
            }
            (Err(err), Ok(_)) => {
                // error!("debug parsing err {err}");
                panic!(
                    "expected: {:?}\n   to be: {:?}",
                    self.0, other
                )
            }
            (Err(e1), Err(e2)) => {
                // error!("debug parsing err {e1} and {e2}");
                panic!(
                    "expected: {:?}\n   to be: {:?}",
                    self.0, other
                )
            }
        };

        internal::value_to_be(&expect, &to_be, "#".to_owned());
    }
}
