use std::{convert::Infallible, fmt::Display, ops::Not};

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub struct MacroError<const CAN_BE_IGNORED: bool> {
    span: Option<Span>,
    message: String,
}

pub union DeferedResult<Ok: Copy, Err: Copy> {
    ok: Ok,
    err: Err,
} 

impl<Ok: Copy, Err: Copy> DeferedResult<Ok, Err> {
    pub fn handle(self, ts: &mut TokenStream) -> Option<Ok> {
        Some(unsafe { self.ok })
    }
}

pub trait HandleError: Sized {
    type Handled;
    type HandledDefault;
    fn handle_and_default(
        self,
        ts: &mut TokenStream,
    ) -> Self::HandledDefault
    where
        Self::HandledDefault: Default,
    {
        self.handle(ts);
        Self::HandledDefault::default()
    }
    fn handle_or<F, R>(self, ts: &mut TokenStream, f: F) -> R
    where
        F: FnOnce(Self::Handled) -> R,
    {
        f(self.handle(ts))
    }

    fn handle(self, ts: &mut TokenStream) -> Self::Handled;
}

pub trait ToUnwrap {
    type Into;
    fn to_unwrap<M: Display, S: Spanned>(
        self,
        msg: M,
        span: Option<S>,
    ) -> Result<Self::Into, MacroError<false>>;
    // like Option::unwrap, used in sitiuation where you don't have msg and span, at witch you should panic
    fn should_work(self) -> Self::Into;
}

impl<T> ToUnwrap for Option<T> {
    type Into = T;
    fn to_unwrap<M: Display, S: Spanned>(
        self,
        msg: M,
        span: Option<S>,
    ) -> Result<Self::Into, MacroError<false>> {
        match self {
            Some(this) => Ok(this),
            None => Err(MacroError {
                span: span.map(|s| s.span()),
                message: msg.to_string(),
            }),
        }
    }
    fn should_work(self) -> Self::Into {
        match self {
            Some(this) => this,
            None => panic!("this should work"),
        }
    }
}

macro_rules! unwrap_handled {
    ($expr:expr, $return_expr:expr) => {{
        match $expr {
            Ok(ok) => ok,
            Err(err) => {
                let err: Result<_, MacroError<HANDLED>> = Err(err);
                return $return_expr;
            }
        }
    }};
}

pub(crate) use unwrap_handled;

macro_rules! unwrap_error {
    ($expr:expr) => {{
        match $expr {
            Ok(ok) => ok,
            Err(err) => {
                return Err(err);
            }
        }
    }};
}

pub(crate) use unwrap_error;

impl<Ok: Default> HandleError for Result<Ok, MacroError<false>> {
    type Handled = Result<Ok, MacroError<true>>;
    type HandledDefault = Ok;
    fn handle(self, ts: &mut TokenStream) -> Self::Handled {
        match self {
            Ok(ok) => Ok(ok),
            Err(this) => match (this.span, this.message.as_str()) {
                (Some(spanned), "") => {
                    quote_spanned!(spanned=> compile_error!("internal error"));
                    Err(MacroError::default())
                }
                (Some(spanned), displaying) => {
                    quote_spanned!(spanned=> compile_error!(#displaying));
                    Err(MacroError::default())
                }
                (None, "") => {
                    quote!(compile_error!("internal error"));
                    Err(MacroError::default())
                }
                (None, displaying) => {
                    quote!(compile_error!(#displaying));
                    Err(MacroError::default())
                }
            },
        }
    }
}

impl MacroError<true> {
    pub fn default() -> Self {
        Self {
            span: None,
            message: String::new(),
        }
    }
}

impl<const CAN_BE_IGNORED: bool> MacroError<CAN_BE_IGNORED> {
    pub fn set_span<S: Spanned>(
        mut self,
        span: &S,
    ) -> MacroError<false> {
        self.span = Some(span.span());

        MacroError {
            span: self.span,
            message: self.message,
        }
    }
    pub fn set_span_if_none<S: Spanned>(
        mut self,
        span: &S,
    ) -> MacroError<false> {
        if self.span.is_none() {
            self.span = Some(span.span());
        }

        MacroError {
            span: self.span,
            message: self.message,
        }
    }
    pub fn add_message<M: Display>(
        mut self,
        msg: M,
    ) -> MacroError<false> {
        if self.message.is_empty().not() {
            self.message.push('\n');
        }

        self.message.push_str(&msg.to_string());

        MacroError {
            span: self.span,
            message: self.message,
        }
    }
}

pub mod exports {
    pub const TO_BE_HANDLED_LATER: bool = false;
    pub const HANDLED: bool = true;
    pub use super::{HandleError, MacroError, ToUnwrap};
}
