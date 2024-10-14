use std::ops::{Deref, DerefMut};

use proc_macro2::TokenStream;

pub trait ErroneusInner {
    type Errors;
    fn new(self) -> Erroneus<Self>
    where
        Self: Sized,
        Self::Errors: Default,
    {
        Erroneus(self, Default::default())
    }
}

#[must_use]
#[derive(Default)]
pub struct Erroneus<T: ErroneusInner>(T, T::Errors);

impl<T: ErroneusInner> Deref for Erroneus<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ErroneusInner> DerefMut for Erroneus<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait ExtendErrors<P: ErroneusInner> {
    fn extend(&mut self, erros: P::Errors);
}

impl<T: ErroneusInner> Erroneus<T> {
    pub fn unwrap<E>(self, into: &mut E) -> T
    where
        E: ExtendErrors<T>, // E: ExtendErrors<T::Errors>,
    {
        into.extend(self.1);
        self.0
    }
}

impl ErroneusInner for () {
    type Errors = Vec<TokenStream>;
}

impl ErroneusInner for TokenStream {
    type Errors = Vec<TokenStream>;
}
