use std::marker::PhantomData;

use sqlx::Database;

use crate::{
    execute_no_cache::ExecuteNoCacheUsingSelectTrait,
    returning::ReturningClause,
    sql_part::{ToSqlPart, WhereItemToSqlPart},
    InitStatement, Query, Statement, SupportNamedBind,
};

pub struct DeleteSt<S, Q: Query<S>, R = ()> {
    pub(crate) where_clause: Vec<Q::SqlPart>,
    pub(crate) ctx: Q::Context1,
    pub(crate) table: &'static str,
    pub(crate) returning: R,
    pub(crate) _sqlx: PhantomData<S>,
}

impl<S, Q, R> ExecuteNoCacheUsingSelectTrait for DeleteSt<S, Q, R> where
    Q: Query<S>
{
}

impl<S, Q> InitStatement for DeleteSt<S, Q, ()>
where
    Q: Query<S>,
{
    type Init = &'static str;
    fn init(init: Self::Init) -> Self {
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
    Q: Query<S>,
{
    fn deref_ctx(&self) -> &<Q as Query<S>>::Context1 {
        &self.ctx
    }
    fn deref_mut_ctx(
        &mut self,
    ) -> &mut <Q as Query<S>>::Context1 {
        &mut self.ctx
    }
    fn _build(self) -> (String, <Q as Query<S>>::Output) {
        self.build()
    }
}

impl<S, Q: Query<S>> DeleteSt<S, Q> {
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

impl<S, Q: Query<S>> DeleteSt<S, Q> {
    pub fn where_<W>(&mut self, item: W)
    where
        W: crate::WhereItem<S, Q> + 'static,
        WhereItemToSqlPart<W>: ToSqlPart<Q, S>,
    {
        let item =
            WhereItemToSqlPart(item).to_sql_part(&mut self.ctx);
        self.where_clause.push(item);
    }
}

impl<'q, S, Q, R> DeleteSt<S, Q, R>
where
    S: SupportNamedBind,
    S: Database,
    R: ReturningClause,
    Q: Query<S>,
{
    pub fn build(self) -> (String, Q::Output) {
        <Q as Query<S>>::build_query(self.ctx, |ctx| {
            let mut str = String::from("DELETE FROM ");

            str.push_str(self.table);

            for (index, where_item) in
                self.where_clause.into_iter().enumerate()
            {
                if index == 0 {
                    str.push_str(" WHERE ");
                } else {
                    str.push_str(" AND ");
                }
                str.push_str(
                    &<Q as Query<S>>::build_sql_part_back(
                        ctx, where_item,
                    ),
                );
            }

            str.push_str(&self.returning.returning());

            str
        })
    }
}
