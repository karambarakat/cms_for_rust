use std::{marker::PhantomData, mem::take};

use sqlx::{database::HasArguments, Database};

use crate::{
    delete_st::DeleteSt, returning::ReturningClause,
    IntoMutArguments,
};

pub struct InsertMany<S, B, R = ()> {
    into: String,
    cols: Vec<String>,
    buffer: B,
    argument_count: usize,
    returning: R,
    _db: PhantomData<S>,
}

impl<'q, S, R>
    InsertMany<S, <S as HasArguments<'q>>::Arguments, R>
where
    R: ReturningClause,
    S: Database,
{
    pub fn _build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        let column = self.cols.len();
        let str = format!(
            "INSERT INTO {} ({}) VALUES {} {}",
            self.into,
            self.cols.join(", "),
            {
                let mut binds = 1;
                let mut s_inner = Vec::new();
                // for _ in 0..column {
                let mut s_inner_inner = Vec::new();
                for cc in 0..self.argument_count {
                    s_inner_inner.push(format!("${}", binds));

                    binds += 1;

                    if ((binds - 1) % self.cols.len()) == 0 {
                        let v = take(&mut s_inner_inner);
                        s_inner
                            .push(format!("({})", v.join(", ")));
                    }
                }

                s_inner.join(", ")
            },
            self.returning.returning(),
        );

        tracing::debug!("insert many {str}");

        (str, self.buffer)
    }
}

pub fn insert_many<'q, S: Database>(
    into: String,
) -> InsertMany<S, <S as HasArguments<'q>>::Arguments> {
    InsertMany {
        into,
        cols: Default::default(),
        buffer: Default::default(),
        returning: (),
        _db: PhantomData,
        argument_count: Default::default(),
    }
}

impl<'q, S: Database>
    InsertMany<S, <S as HasArguments<'q>>::Arguments>
{
    pub fn returning(
        self,
        returning: Vec<&'static str>,
    ) -> InsertMany<
        S,
        <S as HasArguments<'q>>::Arguments,
        Vec<&'static str>,
    > {
        InsertMany {
            returning,
            into: self.into,
            cols: self.cols,
            buffer: self.buffer,
            _db: PhantomData,
            argument_count: self.argument_count,
        }
    }
    pub fn columns(
        self,
        cols: Vec<String>,
    ) -> InsertMany<S, <S as HasArguments<'q>>::Arguments> {
        InsertMany { cols, ..self }
    }
    #[track_caller]
    pub fn values<B>(mut self, values: Vec<B>) -> Self
    where
        B: IntoMutArguments<'q, S>,
    {
            if self.cols.len() != B::LEN {
                panic!("col count shoudl be consistant")
            }
        self.argument_count += values.len() * B::LEN;
        for value in values {
            value.into_arguments(&mut self.buffer);
        }
        self
    }
}
