use std::marker::PhantomData;

use sqlx::Database;

use crate::{
    returning::ReturningClause,
    sql_part::{ToSqlPart, WhereItemToSqlPart},
    Query, SupportNamedBind,
};

pub struct DeleteSt<S, Q: Query<S>, R = ()> {
    pub(crate) where_clause: Vec<Q::SqlPart>,
    pub(crate) ctx: Q::Context1,
    pub(crate) table: &'static str,
    pub(crate) returning: R,
    pub(crate) _sqlx: PhantomData<S>,
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
    pub fn _build(self) -> (String, Q::Output) {
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
