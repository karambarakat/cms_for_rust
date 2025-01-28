use std::marker::PhantomData;

use sqlx::Database;

use crate::{
    execute_no_cache::ExecuteNoCacheUsingSelectTrait,
    ident_safety::PanicOnUnsafe, returning::ReturningClause,
    AcceptTableIdent, BindItem, IdentSafety, Query,
    QueryHandlers, Statement, SupportNamedBind,
};

pub struct DeleteSt<S, Q: Query, I: IdentSafety, R = ()> {
    pub(crate) where_clause: Vec<Q::SqlPart>,
    pub(crate) ctx: Q::Context1,
    pub(crate) table: I::Table,
    pub(crate) returning: R,
    pub(crate) _sqlx: PhantomData<S>,
}

impl<'q, S, Q, I: IdentSafety, R> ExecuteNoCacheUsingSelectTrait
    for DeleteSt<S, Q, I, R>
where
    Q: Query,
{
}

impl<'q, S, Q, I> DeleteSt<S, Q, I>
where
    Q: Query,
    I: IdentSafety,
{
    pub fn init<M>(init: M) -> Self
    where
        I: AcceptTableIdent<M>,
    {
        DeleteSt {
            where_clause: Vec::new(),
            ctx: Default::default(),
            table: I::into_table(init),
            returning: (),
            _sqlx: PhantomData,
        }
    }
}
impl<'q, S, Q, I, R> Statement<S, Q> for DeleteSt<S, Q, I, R>
where
    R: ReturningClause,
    S: Database + SupportNamedBind,
    Q: Query,
    I: IdentSafety,
{
    fn deref_ctx(&self) -> &Q::Context1 {
        &self.ctx
    }
    fn deref_mut_ctx(&mut self) -> &mut Q::Context1 {
        &mut self.ctx
    }
    fn _build(self) -> (String, Q::Output) {
        self.build()
    }
}

impl< S, Q: Query, I: IdentSafety> DeleteSt< S, Q, I> {
    pub fn returning(
        self,
        returning: Vec<&'static str>,
    ) -> DeleteSt< S, Q, I, Vec<&'static str>> {
        DeleteSt {
            returning,
            where_clause: self.where_clause,
            ctx: self.ctx,
            table: self.table,
            _sqlx: PhantomData,
        }
    }
}

impl<S, Q: Query, I: IdentSafety>
    DeleteSt<S, Q, I>
{
    pub fn where_<W>(&mut self, item: W)
    where
        W: BindItem<S, Q, I> + 'static,
        Q: QueryHandlers< S>,
    {
        let item = Q::handle_bind_item(item, &mut self.ctx);
        self.where_clause.push(item);
    }
}

impl< S, Q, R, I> DeleteSt<S, Q, I, R>
where
    S: SupportNamedBind,
    S: Database,
    R: ReturningClause,
    I: IdentSafety,
    Q: Query,
{
    pub fn build(self) -> (String, Q::Output) {
        <Q as Query>::build_query(self.ctx, |ctx| {
            let mut str = String::from("DELETE FROM ");

            str.push_str(self.table.as_ref());

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
