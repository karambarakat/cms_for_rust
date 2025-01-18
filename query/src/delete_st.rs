use std::marker::PhantomData;

use sqlx::Database;

use crate::{
    execute_no_cache::ExecuteNoCacheUsingSelectTrait,
    ident_safety::PanicOnUnsafe, returning::ReturningClause,
    BindItem, Query, QueryHandlers, Statement, SupportNamedBind,
};

pub struct DeleteSt<S, Q: Query, R = ()> {
    pub(crate) where_clause: Vec<Q::SqlPart>,
    pub(crate) ctx: Q::Context1,
    pub(crate) table: String,
    pub(crate) returning: R,
    pub(crate) _sqlx: PhantomData<S>,
}

impl<S, Q, R> ExecuteNoCacheUsingSelectTrait
    for DeleteSt<S, Q, R>
where
    Q: Query,
{
}

impl<S, Q> DeleteSt<S, Q, ()>
where
    Q: Query,
{
    pub fn init(init: String) -> Self {
        DeleteSt {
            where_clause: Vec::new(),
            ctx: Default::default(),
            table: init,
            returning: (),
            _sqlx: PhantomData,
        }
    }
}
impl<S, Q, R> Statement<S, Q> for DeleteSt<S, Q, R>
where
    R: ReturningClause,
    S: Database + SupportNamedBind,
    Q: Query,
{
    fn deref_ctx(&self) -> &<Q as Query>::Context1 {
        &self.ctx
    }
    fn deref_mut_ctx(&mut self) -> &mut <Q as Query>::Context1 {
        &mut self.ctx
    }
    fn _build(self) -> (String, <Q as Query>::Output) {
        self.build()
    }
}

impl<S, Q: Query> DeleteSt<S, Q> {
    pub fn returning(
        self,
        returning: Vec<&'static str>,
    ) -> DeleteSt<S, Q, Vec<&'static str>> {
        DeleteSt {
            returning,
            where_clause: self.where_clause,
            ctx: self.ctx,
            table: self.table,
            _sqlx: PhantomData,
        }
    }
}

impl<S, Q: Query> DeleteSt<S, Q> {
    pub fn where_<W>(&mut self, item: W)
    where
        W: BindItem<S, Q> + 'static,
        Q: QueryHandlers<S>,
    {
        let item = Q::handle_bind_item(item, &mut self.ctx);
        self.where_clause.push(item);
    }
}

impl<'q, S, Q, R> DeleteSt<S, Q, R>
where
    S: SupportNamedBind,
    S: Database,
    R: ReturningClause,
    Q: Query,
{
    pub fn build(self) -> (String, Q::Output) {
        <Q as Query>::build_query(self.ctx, |ctx| {
            let mut str = String::from("DELETE FROM ");

            str.push_str(&self.table);

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
