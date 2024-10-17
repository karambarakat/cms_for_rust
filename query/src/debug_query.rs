use std::{any::Any, marker::PhantomData};

use sqlx::{database::HasArguments, Database};

use crate::positional_buffer::PositionalStaticBuffer;
use crate::{
    execute_no_cache::ExecuteNoCache, Query, Statement,
};

pub struct DebugOutput<S: Database> {
    string: String,
    output: <S as HasArguments<'static>>::Arguments,
}

impl<S: Database> ExecuteNoCache<'static, S, ()>
    for DebugOutput<S>
{
    fn build(
        self,
    ) -> (String, <S as HasArguments<'static>>::Arguments) {
        (self.string, self.output)
    }
}

impl<S: Database> DebugOutput<S> {
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
    pub fn clone_buffer(&self) -> Vec<Box<dyn Any>> {
        todo!()
    }
}

pub trait DebugQueries<Q: Query<S>, S: Database> {
    fn debug(self, infer_db: PhantomData<S>) -> DebugOutput<S>;
}

impl<S, T> DebugQueries<PositionalStaticBuffer, S> for T
where
    T: Statement<S, PositionalStaticBuffer>,
    S: Database,
{
    fn debug(self, _infer_db: PhantomData<S>) -> DebugOutput<S> {
        let built = self._build();
        DebugOutput {
            string: built.0,
            output: built.1,
        }
    }
}
