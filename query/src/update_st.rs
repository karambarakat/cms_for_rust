use std::marker::PhantomData;

use sqlx::Database;

use crate::{
    execute_no_cache::ExecuteNoCacheUsingSelectTrait,
    ident_safety::PanicOnUnsafe, returning::ReturningClause,
    Accept, BindItem, Query, QueryHandlers, Statement,
    SupportNamedBind, SupportReturning,
};

pub struct UpdateSt<S, Q: Query, R = ()> {
    pub(crate) sets: Vec<(String, Q::SqlPart)>,
    pub(crate) where_clause: Vec<Q::SqlPart>,
    pub(crate) ctx: Q::Context1,
    pub(crate) table: String,
    pub(crate) returning: R,
    pub(crate) _sqlx: PhantomData<S>,
}

impl<S, Q: Query, R> UpdateSt<S, Q, R> {
    pub fn set_len(&self) -> usize {
        self.sets.len()
    }
}

impl<S, Q: Query, R> ExecuteNoCacheUsingSelectTrait
    for UpdateSt<S, Q, R>
{
}

impl<S, Q> UpdateSt<S, Q, ()>
where
    Q: Query,
{
    pub fn init(init: String) -> Self {
        UpdateSt {
            sets: Vec::new(),
            where_clause: Vec::new(),
            ctx: Default::default(),
            table: init.to_string(),
            returning: (),
            _sqlx: PhantomData,
        }
    }
}
impl<S, Q, R> Statement<S, Q> for UpdateSt<S, Q, R>
where
    S: Database + SupportNamedBind,
    Q: Query,
    R: ReturningClause,
{
    fn deref_ctx(&self) -> &Q::Context1 {
        &self.ctx
    }
    fn deref_mut_ctx(&mut self) -> &mut Q::Context1 {
        &mut self.ctx
    }

    fn _build(self) -> (String, <Q as Query>::Output) {
        self.build()
    }
}

impl<'q, S, R, Q> UpdateSt<S, Q, R>
where
    S: SupportNamedBind,
    S: Database,
    Q: Query,
{
    pub fn build(self) -> (String, Q::Output)
    where
        R: ReturningClause,
        S: Database + SupportNamedBind,
    {
        <Q as Query>::build_query(self.ctx, |ctx| {
            let mut str = String::from("UPDATE ");

            str.push_str(&self.table);

            str.push_str(" SET ");

            if self.sets.is_empty() {
                panic!("empty set on update")
            }

            for (index, (column, value)) in
                self.sets.into_iter().enumerate()
            {
                if index != 0 {
                    str.push_str(", ");
                }
                str.push_str(&column);
                str.push_str(" = ");
                str.push_str(
                    &<Q as Query>::build_sql_part_back(
                        ctx, value,
                    ),
                );
            }

            for (index, where_item) in
                self.where_clause.into_iter().enumerate()
            {
                if index == 0 {
                    str.push_str(" WHERE ");
                } else {
                    str.push_str(" AND ");
                }
                str.push_str(
                    &<Q as Query>::build_sql_part_back(
                        ctx, where_item,
                    ),
                );
            }

            str.push_str(&self.returning.returning());

            str
        })
    }
}

impl<S, Q: Query> UpdateSt<S, Q, ()> {
    pub fn returning_<R>(
        self,
        cols: Vec<R>,
    ) -> UpdateSt<S, Q, Vec<R>>
    where
        S: SupportReturning,
    {
        UpdateSt {
            sets: self.sets,
            where_clause: self.where_clause,
            ctx: self.ctx,
            table: self.table,
            returning: cols,
            _sqlx: PhantomData,
        }
    }
    #[deprecated]
    pub fn returning(
        self,
        cols: Vec<&'static str>,
    ) -> UpdateSt<S, Q, Vec<&'static str>> {
        UpdateSt {
            sets: self.sets,
            where_clause: self.where_clause,
            ctx: self.ctx,
            table: self.table,
            returning: cols,
            _sqlx: PhantomData,
        }
    }
}

impl<S, Q: QueryHandlers<S>, R> UpdateSt<S, Q, R> {
    pub fn set<T>(&mut self, column: String, value: T)
    where
        Q: Accept<T, S>,
    {
        let part = Q::handle_accept(value, &mut self.ctx);
        self.sets.push((column, part));
    }

    pub fn where_<I>(&mut self, item: I)
    where
        I: BindItem<S, Q> + 'static,
        
    {
        let item = Q::handle_bind_item(item, &mut self.ctx);
        self.where_clause.push(item);
    }
}
