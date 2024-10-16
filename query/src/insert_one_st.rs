use std::marker::PhantomData;

use sqlx::{
    database::HasArguments, Arguments, Database, Encode, Type,
};

use crate::{
    returning::ReturningClause, InitStatement, IntoMutArguments,
    SupportNamedBind, SupportReturning,
};

pub struct InsertStOne<'q, S: Database, R = ()> {
    pub(crate) input: Vec<&'static str>,
    pub(crate) output: Option<Vec<&'static str>>,
    pub(crate) from: &'static str,
    pub(crate) buffer: <S as HasArguments<'q>>::Arguments,
    pub(crate) returning: R,
    pub(crate) _pd: PhantomData<(S, &'q ())>,
}

impl<'q, S> InitStatement for InsertStOne<'_, S, ()>
where
    S: Database,
{
    type Init = &'static str;
    fn init(init: Self::Init) -> Self {
        InsertStOne {
            input: Vec::new(),
            output: None,
            from: init,
            buffer: Default::default(),
            returning: (),
            _pd: PhantomData,
        }
    }
}

impl<'q, S, R> InsertStOne<'q, S, R>
where
    R: ReturningClause,
    S: Database + SupportNamedBind,
{
    pub fn _build(
        self,
    ) -> (String, <S as HasArguments<'q>>::Arguments) {
        let column = self.input.len();
        let str = format!(
            "INSERT INTO {} ({}) VALUES ({}){};",
            self.from,
            self.input.join(", "),
            {
                let mut binds = 1;
                let mut s_inner = Vec::new();
                for _ in 0..column {
                    s_inner.push(format!("${}", binds));
                    binds += 1;
                }

                s_inner.join(", ")
            },
            self.returning.returning(),
        );

        (str, self.buffer)
    }
}

impl<'q, S> InsertStOne<'q, S>
where
    S: Database,
{
    pub fn returning<R>(
        self,
        returning: R,
    ) -> InsertStOne<'q, S, R>
    where
        S: SupportReturning,
    {
        InsertStOne {
            input: self.input,
            output: self.output,
            from: self.from,
            buffer: self.buffer,
            returning,
            _pd: PhantomData,
        }
    }
    pub fn insert_struct<T>(
        &mut self,
        column: &[&'static str],
        value: T,
    ) where
        T: IntoMutArguments<'q, S>,
    {
        self.input.extend_from_slice(column);
        value.into_arguments(&mut self.buffer);
    }
    pub fn insert<T>(&mut self, column: &'static str, value: T)
    where
        T: Type<S> + for<'e> Encode<'e, S> + Send + 'q,
    {
        self.input.push(column);
        self.buffer.add(value);
    }
}
